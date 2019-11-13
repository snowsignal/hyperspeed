use std::marker::PhantomData;
use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use std::collections::HashMap;
use tokio::prelude::{Stream, AsyncRead, Future, Async, Poll};
use tokio::prelude::stream::ForEach;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use std::fmt::Debug;

pub type NetworkResult<T> =  Result<T, Box<dyn std::error::Error>>;

pub struct ClientMessage {
    //inner_bytes: Bytes
}

pub type ClientID = u32;

#[derive(Debug)]
pub struct Client {
    socket: TcpStream
}
#[derive(Debug)]
pub struct ClientHandle<M: 'static + Send + From<ClientMessage> + Into<ClientMessage> + Debug>(pub Sender<M>, pub Receiver<M>);

pub struct ServerBuilder<M: 'static + Send + From<ClientMessage> + Into<ClientMessage> + Debug> {
    _pd: PhantomData<M>,
    thread_cap: u16,
    addr: &'static str,
    port: u16
}

pub struct Server<M: 'static + Send + From<ClientMessage> + Into<ClientMessage> + Debug> {
    thread_cap: u16,
    address: SocketAddr,

    listener: TcpListener,
    clients: HashMap<ClientID, ClientHandle<M>>,
    connection_channel: Receiver<ClientHandle<M>>,
}

impl<M: 'static + Send + From<ClientMessage> + Into<ClientMessage> + Debug> ServerBuilder<M> {
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
            clients: HashMap::new(),
            connection_channel: channel(1).1,
        }
    }
}

impl<M: 'static + Send + From<ClientMessage> + Into<ClientMessage> + Debug> Server<M> {
    pub fn new() -> ServerBuilder<M> {
        // This is the default Server configuration
        ServerBuilder {
            _pd: PhantomData,
            thread_cap: 5,
            addr: "0.0.0.0",
            port: 4343,
        }
    }
    pub fn run(mut self) -> NetworkResult<()> {
        Ok(())
    }
}

impl Future for Client {
    type Item = ();
    type Error = std::io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::NotReady)
    }
}