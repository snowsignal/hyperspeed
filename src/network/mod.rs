use std::marker::PhantomData;
use tokio::net::{TcpListener, TcpStream};
use std::net::{SocketAddr, Shutdown};
use std::collections::HashMap;
use tokio::prelude::*;
use tokio::prelude::stream::ForEach;
use bytes::{Bytes, BytesMut};
use futures::sync::mpsc::{channel, Receiver, Sender, unbounded, UnboundedReceiver, UnboundedSender};
use std::fmt::Debug;
use std::thread;
use std::thread::JoinHandle;
use crate::ecs::GameUpdate;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

/// A wrapper for a binary packet sent to or from the server socket.
pub struct ClientMessage {
    pub bytes: BytesMut
}

/// A wrapper type that maps clients to their address and the channel
/// to communicate with them.
pub type ClientMap<M> = HashMap<ClientID, (SocketAddr, UnboundedSender<M>)>;

/// A wrapper type that puts a client map in an arc mutex pointer so
/// it can be accessed and modified by multiple threads.
pub type SharedClientMap<M> = Arc<Mutex<ClientMap<M>>>;

/// A client identifier number, used to represent the UID (Unique Identifier) for each client.
pub type ClientID = u32;

/// A set of traits required for any message type.
pub trait Message = 'static + Send + TryFrom<ClientMessage, Error=()> + Into<ClientMessage> + Debug;

/// A TCP socket that serializes and deserializes messages automatically
pub struct MessageSocket<M: Message> {
    _pd: PhantomData<M>,
    socket: TcpStream,
    buffer: BytesMut
}

/// A client future that processes a client connection and
/// communicates with a server.
pub struct Client<M: Message> {
    socket: MessageSocket<M>,
    id: ClientID,
    server_tx: UnboundedSender<M>,
    server_rx: UnboundedReceiver<M>,
    shared_client_map: SharedClientMap<M>,
}

/// A map of clients to messages from that client.
#[derive(Debug)]
pub struct ClientInput<M: Message> {
    input: HashMap<ClientID, Vec<M>>
}

/// A communication channel with the server.
pub struct ServerHandle<M: Message>(pub UnboundedReceiver<ClientInput<M>>, pub UnboundedSender<GameUpdate>, pub JoinHandle<()>);

/// A helper class for server generation
pub struct ServerBuilder<M: Message> {
    _pd: PhantomData<M>,
    thread_cap: u16,
    addr: &'static str,
    port: u16
}

/// A struct that handles multi-client networking.
pub struct Server<M: Message> {
    thread_cap: u16,
    address: SocketAddr,

    listener: TcpListener,
    clients: SharedClientMap<M>,
    messages: UnboundedReceiver<M>
}

static mut CLIENT_ID_COUNTER: ClientID = 0;

fn get_id(socket: &mut TcpStream) -> ClientID {
    // This only gets called from one thread, so there won't be any data racing.
    // In other words this (shouldn't) ever panic.
    unsafe {
        CLIENT_ID_COUNTER += 1;
        CLIENT_ID_COUNTER - 1
    }
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
            messages: unbounded().1,
        }
    }
}

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
    /// communication interface with the server, `ServerHandle`
    pub fn run(mut self) -> ServerHandle<M> {
        let (client_input_tx, client_input_rx) = unbounded::<ClientInput<M>>();
        let (server_tx, server_rx) = unbounded::<M>();
        let (game_tx, game_rx) = unbounded::<GameUpdate>();
        let shared_client_map = self.clients.clone();
        let hc_client_map = shared_client_map.clone();
        thread::spawn(|| Server::handle_channels(server_rx, hc_client_map));
        let server_process = self.listener.incoming().for_each(move |mut socket| {
            let id = get_id(&mut socket);
            let (tx, rx) = unbounded();
            {
                let mut shared_client_map = shared_client_map.lock().unwrap();
                shared_client_map.insert(id, (socket.local_addr().unwrap(), tx));
            }
            tokio::spawn(Client::new(socket, id, shared_client_map.clone(),rx, server_tx.clone()));
             Ok(())
        }).map_err(|e| ()); //TODO: Do something with error
        let handle = thread::spawn(move || tokio::run(server_process));
        ServerHandle(client_input_rx, game_tx, handle)
    }

    fn handle_channels(server_rx: UnboundedReceiver<M>, client_map: SharedClientMap<M>) {
        loop {
            thread::park()
        }
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
    pub fn new(socket: TcpStream, id: ClientID, shared_client_map: SharedClientMap<M>, server_rx: UnboundedReceiver<M>, server_tx: UnboundedSender<M>) -> Client<M> {
        Client {
            socket: MessageSocket::new(socket),
            id,
            server_rx,
            server_tx,
            shared_client_map
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
                match self.server_tx.unbounded_send(msg) {
                    Ok(_) => (),
                    Err(e) => println!("{}", e)
                }
            }
        }
        Ok(Async::NotReady)
    }
}

impl<M: Message> Drop for Client<M> {
    fn drop(&mut self) {
        self.socket.close().unwrap();
        self.shared_client_map.lock().unwrap().remove(&self.id);
    }
}