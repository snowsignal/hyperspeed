use super::world::*;
use super::Server;
use super::ServerConfig;
use super::PlayerInputBuffer;
use crate::utils::*;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::spawn;

use std::collections::{HashMap, VecDeque};
use specs::Component;
use super::server::InputBufferMutex;
use std::time::Instant;
use crate::core::world::Connection;
use crate::core::server::StreamData;
use std::net::TcpStream;
use crate::components::{Position, Camera, Visible};

pub struct Engine<'a, 'b, E: Sync + Send + Clone + 'static> {
    pub world: World<'a, 'b>,
    master_controller: Box<MasterController<ObserverEvent=E>>,
    input_buffer: Option<InputBufferMutex>,
    server_conf: ServerConfig,
    prev_time: Instant,
    connection_channel: Receiver<(Connection, Sender<ClientView>)>,
    view_channels: HashMap<String, Sender<ClientView>>,
    server_stream_handler: Option<StreamHandler>
}

pub struct EngineBuilder<'a, 'b, E: Sync + Send + Clone + 'static> {
    server_conf: ServerConfig,
    system_executor_builder: SystemExecutorBuilder<'a, 'b>,
    master_controller: Option<Box<MasterController<ObserverEvent=E>>>,
    server_stream_handler: Option<StreamHandler>
}

impl<'a, 'b, E: Sync + Send + Clone + 'static> Engine<'a, 'b, E> {
    pub fn new() -> EngineBuilder<'a, 'b, E> {
        EngineBuilder {
            server_conf: ServerConfig::new(),
            system_executor_builder: SystemExecutor::new(),
            master_controller: None,
            server_stream_handler: None
        }
    }

    pub fn init_resources(&mut self) {
        // This is the event/messaging
        self.world.ecs_world.add_resource(Messages::<E>::new());
        self.world.ecs_world.add_resource(InputMap::new());
        self.world.ecs_world.add_resource(ViewMap::new());
        self.world.ecs_world.add_resource(ConnectionCollection::new());

        // Register default components

        self.world.ecs_world.register::<Position>();
        self.world.ecs_world.register::<Visible>();
        self.world.ecs_world.register::<Camera>();
    }

    pub fn register<T: Component>(&mut self)
    where <T as Component>::Storage : std::default::Default {
        self.world.ecs_world.register::<T>();
    }

    pub fn start_server(&mut self) {
        fn default(t: &mut TcpStream) -> StreamData {
            StreamData::do_connect_str("default_key")
        }


        let (sender, reciever) = channel();

        let mut handler = None;

        ::std::mem::swap(&mut self.server_stream_handler, &mut handler);

        let handler = handler
            .unwrap_or(default);

        let mut server = Server::new(self.server_conf.clone(), sender, handler);

        self.connection_channel = reciever;

        self.input_buffer = Some(server.get_input_buffer()); // Get a reference to the input buffer even after it gets moved to another thread

        spawn( move || server.main_loop());

        self.prev_time = Instant::now();

        // Call MC init
        self.master_controller.start(&mut self.world, 0.0);
    }

    fn get_new_connection(&mut self) -> Option<(Connection, Sender<ClientView>)> {
        match self.connection_channel.try_recv() {
            Ok(C) => Some(C),
            Err(E) => match E {
                Empty => None,
                Disconnected => panic!("Engine fault: Server channel was disconnected")
            }
        }
    }

    fn get_inputs(&mut self) -> HashMap<String, VecDeque<Input>> {
        let mut lock = self.input_buffer.as_mut().unwrap().lock();
        match lock {
            Ok(ref mut lock) => {
                let mut input_map = HashMap::new();
                ::std::mem::swap(&mut input_map, lock);
                input_map
            }
            _ => {
                panic!("The input buffer mutex was poisoned!");
            }
        }

    }
    
    pub fn tick(&mut self) {
        let tmp = self.prev_time;
        self.prev_time = Instant::now();
        let time = self.prev_time - tmp;
        let instruction = self.master_controller.tick(&mut self.world, time.as_float_secs());

        let mut new_connection = self.get_new_connection();

        while new_connection.is_some() {
            println!("Processing new connection!");
            match new_connection.unwrap() {
                (conn, sender) => {
                    self.view_channels.insert(conn.key.clone(), sender);
                    self.world.connections.push(conn);
                }
            }
            new_connection = self.get_new_connection();
        }

        let mut conn_ref = self.world.ecs_world.write_resource::<ConnectionCollection>();
        ::std::mem::swap(&mut *conn_ref, &mut self.world.connections);
        drop(conn_ref);

        match instruction {
            EngineInstruction::Run {
                run_dispatcher
            } => {
                if run_dispatcher {
                    let inputs = self.get_inputs();
                    self.world.ecs_world.add_resource(inputs);
                    self.world.system_executor.run(&mut self.world.ecs_world);
                    self.world.ecs_world.maintain();
                }
            }
            _ => {}
        }

        // Get views
        let mut view_ref = self.world.ecs_world.write_resource::<ViewMap>();
        let mut views = ViewMap::new();
        // Swap views
        ::std::mem::swap(&mut *view_ref, &mut views);
        drop(view_ref);
        // Send views through view channels
        for (key, view) in views {
            match self.view_channels.get_mut(&key) {
                Some(channel) => {
                    match channel.send(view) {
                            Ok(_) => {},
                            Err(_) => {
                            println!("Engine detects client stream thread has exited. Deleting connection.");
                            self.view_channels.remove(&key); // TODO: Remove connection
                            self.remove_connection(&key);
                        }
                    }
                },
                None => {
                    // The view channel does not exist, but it could be initialised later. So we do nothing here.
                }
            }
        }
    }

    fn remove_connection(&mut self, key: &String) {
        self.world.connections.remove(key);
    }
}

impl<'a, 'b, E: Sync + Send + Clone + 'static> EngineBuilder<'a, 'b, E> {
    pub fn with_name(mut self, name: &str) -> Self {
        self.server_conf.server_name = name.to_string();
        self
    }
    
    pub fn on_port(mut self, port: u16) -> Self {
        self.server_conf.port = port;
        self
    }
    
    pub fn with_system<S>(mut self, system: S, name: &str, dep: &[&str]) -> Self
    where
        S: for<'c> specs::System<'c> + Send + 'a {
        self.system_executor_builder.add_system(system, name, dep);
        self
    }
    
    pub fn with_mc<M: 'static>(mut self, master_controller: M) -> Self
    where
        M: MasterController<ObserverEvent=E> {
        self.master_controller = Some(Box::new(master_controller));
        self
    }
    pub fn with_stream_handler(mut self, handler: StreamHandler) -> Self {
        self.server_stream_handler = Some(handler);
        self
    }
    
    pub fn build(mut self) -> Option<Engine<'a, 'b, E>> {
        let mut engine = Engine {
            world: World {
                system_executor: self.system_executor_builder.build(),
                ecs_world: specs::prelude::World::new(),
                connections: ConnectionCollection::new(),
            },
            master_controller: self.master_controller?,
            server_conf: self.server_conf,
            input_buffer: None,
            prev_time: Instant::now(),
            // This is a fake channel
            connection_channel: channel().1,
            view_channels: HashMap::new(),
            server_stream_handler: self.server_stream_handler
        };
        engine.init_resources();
        Some(engine)
    }
}