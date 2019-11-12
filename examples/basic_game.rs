extern crate hyperspeed;

use hyperspeed::{System, WriteStorage, ReadStorage,
                 Read, WriteViewMap, Entities, WriteConnections,
                 define_component, Component, VecStorage, Join};
use hyperspeed::core::{World, Engine, MasterController, EngineInstruction, ClientView, StreamData};

use std::thread::sleep;
use std::time::Duration;
use std::net::TcpStream;
use bytes::{BufMut, BytesMut};
use std::io::Read as R;
use specs::world::EntitiesRes;
use hyperspeed::utils::server::read_message_from_stream;
use hyperspeed::utils::server::StreamReadResult::{ValidMessage, StreamError, InvalidMessage};
use hyperspeed::components::Visible;


struct Position {
    pub x: f32,
    pub y: f32
}

define_component!(Position);

struct PlayerControllable {
    pub player_key: String
}

define_component!(PlayerControllable);

fn start_game(world: &mut World) -> bool {
    if world.connections.size() < 2 {
        println!("We can't start the game yet!");
        return false;
    }

    true
}

struct MoveSystem {}

impl<'a> System<'a> for MoveSystem {
    type SystemData = WriteStorage<'a, Position>;

    fn run(&mut self, mut pos: Self::SystemData) {
        for mut p in (&mut pos).join() {
            p.x += 0.1;
        }
    }
}

struct ConnectionSystem {

}

impl<'a> System<'a> for ConnectionSystem {
    type SystemData = (Entities<'a>, WriteConnections<'a>, WriteStorage<'a, Position>, WriteStorage<'a, PlayerControllable>, WriteStorage<'a, Visible>);

    fn run(&mut self, (entities, mut connections, mut pos, mut player_controllable, mut visible): Self::SystemData) {
        for key in (*connections).pop_new_keys() {
            println!("Making new entity!!");
            entities.build_entity()
                .with(PlayerControllable { player_key: key }, &mut player_controllable)
                .with(Position { x: 100.0, y: 100.0 }, &mut pos)
                .with(Visible { sprite: 0 }, &mut visible)
                .build();
        }
    }
}

struct RenderSystem {}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (WriteViewMap<'a>, Read<'a, bool>, ReadStorage<'a, Position>, ReadStorage<'a, PlayerControllable>);
    fn run(&mut self, (mut view_map, should_render, positions, players): Self::SystemData) {
        if *should_render {
            for pc in players.join() {
                let mut view = ClientView {
                    sprites: vec!(),
                    loc: vec!()
                };
                for p in positions.join() {
                    view.sprites.push(0);
                    view.loc.push((p.x, p.y));
                }
                view_map.insert(pc.player_key.clone(), view);
            }
        }
    }
}

struct MC {}

impl MasterController for MC {
    type ObserverEvent = Message;

    fn start(&mut self, world: &mut World, dt: f64) {

    }

    fn tick(&mut self, world: &mut World, dt: f64) -> EngineInstruction {
        // Regulate ticks
        sleep(Duration::from_millis(20));
        if world.connections.size() > 0 {
            world.ecs_world.add_resource(true);
        } else {
            world.ecs_world.add_resource(false);
        }
        EngineInstruction::Run {
            run_dispatcher: true
        }
    }
}

fn process_stream(mut stream: &mut TcpStream) -> StreamData {
    let mut buffer = BytesMut::new();
    buffer.reserve(512);
    buffer.put(&[0; 512][..]);
    match read_message_from_stream(&mut stream, &mut buffer) {
        ValidMessage(msg) => StreamData::do_connect(msg),
        InvalidMessage => StreamData::dont_connect(),
        StreamError(e) => StreamData::dont_connect(),
        _ => unreachable!()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Message {
    Up,
    Down,
    Left,
    Right
}

fn main() {
    let mut engine = Engine::<Message>::new().with_mc(MC {})
        .with_system(ConnectionSystem {}, "c", &[])
        .with_system(MoveSystem {}, "m", &["c"])
        .with_system(RenderSystem {}, "render", &["c", "m"])
        .with_stream_handler(process_stream)
        .build();
    if let Some(mut engine) = engine {
        engine.register::<Position>();
        engine.register::<PlayerControllable>();
        engine.start_server();
        loop {
            engine.tick();
        }
    } else {
        println!("Engine could not be initialized!");
    }
}