use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{JoinHandle, spawn, sleep};
use super::world::{Input, Connection, ClientView};
use std::collections::{HashMap, VecDeque};
use bytes::{BytesMut, BufMut};
use std::ops::{Deref, DerefMut};
use std::io::{Read, Write};
use std::sync::mpsc::{Sender, channel, Receiver};
use std::time::Duration;
use crate::utils::server::*;
use crate::utils::StreamHandler;

pub(crate) type InputBufferMutex = Arc<Mutex<PlayerInputBuffer>>;

pub(crate) struct PlayerInputBuffer {
    inner: HashMap<String, VecDeque<Input>>
}

#[derive(Clone)]
pub struct StreamData {
    login_key: String,
    should_connect: bool
}

impl StreamData {
    pub fn should_connect(&self) -> bool {
        self.should_connect
    }

    pub fn login_key(&self) -> String {
        self.login_key.clone()
    }

    pub fn do_connect(login_key: String) -> Self {
        StreamData {
            login_key,
            should_connect: true
        }
    }

    pub fn do_connect_str(login_key: &str) -> Self {
        StreamData {
            login_key: login_key.to_string(),
            should_connect: true
        }
    }

    pub fn dont_connect() -> Self {
        StreamData {
            login_key: "".to_string(),
            should_connect: false
        }
    }
}


pub(crate) struct Server {
    stream_handle: StreamHandler,
    tcp_listener: TcpListener,
    input_stream: InputBufferMutex,
    connection_channel: Sender<(Connection, Sender<ClientView>)>,
}

#[derive(Clone)]
pub(crate) struct ServerConfig {
    pub port: u16,
    pub server_name: String
}

impl PlayerInputBuffer {
    pub fn new() -> Self {
        PlayerInputBuffer {
            inner: HashMap::new()
        }
    }

    pub fn push_input(&mut self, player: String, input: Input) {
        if let Some(mut input_v) = self.inner.get_mut(&player) {
            input_v.push_back(input);
        } else {
            self.inner.insert(player, cascade::cascade! {
                VecDeque::new();
                ..push_back(input);
            });
        }
    }

    pub fn pop_input(&mut self, player: String) -> Option<Input> {
        if let Some(mut input_v) = self.inner.get_mut(&player) {
            input_v.pop_front()
        } else {
            None
        }
    }
}

impl ServerConfig {
    pub fn new() -> Self {
        ServerConfig {
            port: 1212, // the default port for Hyperspeed
            server_name: "default_name".to_string()
        }
    }
}

impl Server {
    pub(crate) fn new(s: ServerConfig, c_sender: Sender<(Connection, Sender<ClientView>)>, stream_handler: StreamHandler) -> Server {
        Server {
            tcp_listener: TcpListener::bind(format!("0.0.0.0:{}", s.port)).unwrap(),
            input_stream: Arc::new(Mutex::new(PlayerInputBuffer::new())),
            connection_channel: c_sender,
            stream_handle: stream_handler
        }
    }
    pub(crate) fn main_loop(&mut self) {
        for stream in self.tcp_listener.incoming() {
            let mut stream = stream.unwrap();
            let data = (self.stream_handle)(&mut stream);
            match data {
                StreamData {
                    login_key,
                    should_connect
                } => {
                    if should_connect {
                        let mutex_clone = self.input_stream.clone();
                        let (send, recv) = channel();
                        let conn = Connection { key: login_key.clone() };
                        self.connection_channel.send((conn, send));
                        spawn(move || stream_communicate(stream, recv, mutex_clone, login_key));
                    }
                }
            }
        }
    }
    pub(crate) fn get_input_buffer(&self) -> Arc<Mutex<PlayerInputBuffer>> {
        self.input_stream.clone()
    }
}

impl Deref for PlayerInputBuffer {
    type Target = HashMap<String, VecDeque<Input>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PlayerInputBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

fn put_buffer(input_buffer: &mut InputBufferMutex, player: String, input: Input) {
    let mut lock = input_buffer.lock().unwrap();
    lock.push_input(player, input);
    drop(lock);
}

fn get_new_view(receiver: &mut Receiver<ClientView>) -> Option<ClientView> {
    match receiver.try_recv() {
        Ok(V) => Some(V),
        Err(E) => match E {
            Empty => None,
            _ => panic!("View channel was closed!")
        }
    }
}

const BUFFER_SIZE: usize = 512;
fn stream_communicate(mut stream: TcpStream, mut view_channel: Receiver<ClientView>, mut input_m: InputBufferMutex, key: String) {
    println!("Connection made!");
    let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
    buffer.put(&[0; BUFFER_SIZE][..]);
    stream.set_nonblocking(true);
    loop {
        // send new data to the client
        let mut view = get_new_view(&mut view_channel);
        if view.is_some() {
            // Get latest view
            loop {
                let mut tmp = get_new_view(&mut view_channel);
                if tmp.is_some() {
                    view = tmp;
                } else {
                    break;
                }
            }
            // Update the client's view:
            send_view_to_stream(&mut stream, view.unwrap());
        }

        use self::StreamReadResult::*;
        match read_from_message_from_stream_nonblocking(&mut stream, &mut buffer) {
            ValidMessage(s) => handle_msg(s, &mut input_m),
            InvalidMessage => println!("Invalid message from client!"),
            NotReady => continue,
            StreamError(e) => {
                println!("The stream has been closed and the client thread is exiting due to an error: {}", e);
                return;
            }
        }
    }
}

fn handle_msg(msg: String, mut input_m: &mut InputBufferMutex) {
    println!("{}", msg);
    let msg = serde_json::from_str(msg.as_str());
    match msg {
        Ok(InputMessage {
                clicks,
            keys
             }) => {
            println!("{:?}", clicks);
        },
        Err(_) => ()
    }
}
