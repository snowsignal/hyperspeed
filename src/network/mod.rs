use std::marker::PhantomData;
use tokio::net::{TcpListener, TcpStream};
use std::net::{SocketAddr, Shutdown};
use std::collections::HashMap;
use tokio::prelude::*;
use tokio::prelude::stream::ForEach;
use bytes::{Bytes, BytesMut};
use tokio::sync::mpsc::{channel, Receiver, Sender, unbounded_channel, UnboundedReceiver, UnboundedSender};
use std::fmt::Debug;
use std::thread;
use std::thread::JoinHandle;
use crate::ecs::GameUpdate;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};


pub struct ClientMessage {
    pub bytes: BytesMut
}

pub type ClientMap<M> = HashMap<ClientID, (SocketAddr, UnboundedSender<M>)>;

pub type SharedClientMap<M> = Arc<Mutex<ClientMap<M>>>;

pub type ClientID = u32;

pub trait Message = 'static + Send + TryFrom<ClientMessage, Error=()> + Into<ClientMessage> + Debug;

pub struct MessageSocket<M: Message> {
    _pd: PhantomData<M>,
    socket: TcpStream,
    buffer: BytesMut
}

pub struct Client<M: Message> {
    socket: MessageSocket<M>,
    server_tx: UnboundedSender<M>,
    server_rx: UnboundedReceiver<M>
}

#[derive(Debug)]
pub struct ClientInput<M: Message> {
    input: HashMap<ClientID, Vec<M>>
}

pub struct ServerHandle<M: Message>(pub UnboundedReceiver<ClientInput<M>>, pub UnboundedSender<GameUpdate>, pub JoinHandle<()>);

pub struct ServerBuilder<M: Message> {
    _pd: PhantomData<M>,
    thread_cap: u16,
    addr: &'static str,
    port: u16
}

pub struct Server<M: Message> {
    thread_cap: u16,
    address: SocketAddr,

    listener: TcpListener,
    clients: SharedClientMap<M>,
    messages: UnboundedReceiver<M>,
}

impl<M: Message> ServerBuilder<M> {
    pub fn maximum_threads(mut self, thread_max: u16) -> ServerBuilder<M> {
        self.thread_cap = thread_max;
        self
    }
    pub fn address(mut self, address: &'static str) -> ServerBuilder<M> {
        self.addr = address;
        self
    }
    pub fn port(mut self, port: u16) -> ServerBuilder<M> {
        self.port = port;
        self
    }
    pub fn build(self) -> Server<M> {
        let socket_addr = format!("{}:{}", self.addr, self.port).parse().unwrap();
        Server {
            thread_cap: self.thread_cap,
            address: socket_addr,
            listener: TcpListener::bind(&socket_addr).unwrap(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            messages: unbounded_channel().1,
        }
    }
}

fn get_id(socket: &mut TcpStream) -> ClientID { 0 }

impl<M: Message> Server<M> {
    pub fn new() -> ServerBuilder<M> {
        // This is the default Server configuration
        ServerBuilder {
            _pd: PhantomData,
            thread_cap: 5,
            addr: "0.0.0.0",
            port: 4343,
        }
    }

    /// Starts a new thread with Tokio running the server processes. Returns a
    /// communication interface with the server
    pub fn run(mut self) -> ServerHandle<M> {
        let (client_input_tx, client_input_rx) = unbounded_channel::<ClientInput<M>>();
        let (server_tx, server_rx) = unbounded_channel::<M>();
        let (server2_tx, server2_rx) = unbounded_channel::<M>();
        let (game_tx, game_rx) = unbounded_channel::<GameUpdate>();
        let shared_client_map = self.clients.clone();
        let server_process = self.listener.incoming().for_each(move |mut socket| {
            let id = get_id(&mut socket);
            let mut shared_client_map = shared_client_map.lock().unwrap();
            let (tx, rx) = unbounded_channel();
            shared_client_map.insert(id, (socket.local_addr().unwrap(), tx));
            tokio::spawn(Client::new(socket, rx, server2_tx.clone()));
             Ok(())
        }).map_err(|e| ()); //TODO: Do something with error
        let handle = thread::spawn(move || tokio::run(server_process));
        ServerHandle(client_input_rx, game_tx, handle)
    }
}

impl<M: Message> MessageSocket<M> {
    pub fn new(socket: TcpStream) -> MessageSocket<M> {
        const MSG_SOCKET_BUF_CAP: usize = 4096;
        MessageSocket {
            _pd: PhantomData,
            socket: socket,
            buffer: BytesMut::with_capacity(MSG_SOCKET_BUF_CAP)
        }
    }
}

impl<M: Message> Stream for MessageSocket<M> {
    type Item = M;
    type Error = std::io::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        // Attempt to read from the socket
        match self.socket.poll_read(&mut self.buffer) {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(_)) => {
                // Attempt to convert to message
                let buffer = self.buffer.clone();
                let client_m = ClientMessage { bytes: buffer };
                match M::try_from(client_m) {
                    Ok(msg) => {
                        // Flush buffer
                        self.buffer.clear();
                        Ok(Async::Ready(Some(msg)))
                    },
                    Err(_) => {
                        Ok(Async::Ready(None))
                    }
                }
            },
            Err(e) => Err(e)
        }
    }
}

impl<M: Message> Sink for MessageSocket<M> {
    type SinkItem = M;
    type SinkError = std::io::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        // Extract bytes from ClientMessage
        let ClientMessage { bytes: bytes } = M::into(item);
        // Begin sending over self.socket
        self.socket.write(bytes.as_ref());
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.socket.flush()?;
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(self.socket.shutdown(Shutdown::Write)?))
    }
}

impl<M: Message> Client<M> {
    pub fn new(socket: TcpStream, server_rx: UnboundedReceiver<M>, server_tx: UnboundedSender<M>) -> Client<M> {
        Client {
            socket: MessageSocket::new(socket),
            server_rx,
            server_tx
        }
    }
}

impl<M: Message> Future for Client<M> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Each client sends, at most, 10 messages per tick.
        const MESSAGE_LIMIT: u32 = 10;
        // First, get messages from the server
        for i in 0..MESSAGE_LIMIT {
            match self.server_rx.poll() {
                Ok(Async::Ready(Some(msg))) => {
                    self.socket.start_send(msg);

                    if i + 1 == MESSAGE_LIMIT {
                        task::current().notify();
                    }
                },
                _ => break
            }
        }

        match self.socket.poll_complete() {
            Ok(_) => (),
            Err(_) => return Err(()) //TODO: Fix error coercion
        };

        while let Ok(Async::Ready(msg)) = self.socket.poll() {
            if let Some(msg) = msg {
                self.server_tx.send(msg);
            }
        }
        Ok(Async::NotReady)
    }
}