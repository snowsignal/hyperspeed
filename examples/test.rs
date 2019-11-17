#![feature(try_from)]
extern crate hyperspeed;
use hyperspeed::network::*;
use hyperspeed::script::{PythonInterpreter, InterpreterResult};
use bytes::{BytesMut};
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct Message {
    count: u32
}

impl TryFrom<ClientMessage> for Message {
    type Error = ();

    fn try_from(_: ClientMessage) -> Result<Message, ()> {
        Ok(Message {
            count: 0
        })
    }
}

impl Into<ClientMessage> for Message {
    fn into(self) -> ClientMessage {
        ClientMessage {
            bytes: BytesMut::new()
        }
    }
}


fn main() -> InterpreterResult<()> {
    let mut py = PythonInterpreter::new();
    py.include("./examples")?;
    let module = py.load_module("example")?;
    let test_value = py.get_value(module, "test_value")?;
    let test_value: Box<u32> = py.convert(&test_value)?;
    println!("Test value from Python: {}", test_value);

    let server = Server::<Message>::new().build().run();
    loop {}

    Ok(())
}