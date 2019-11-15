extern crate hyperspeed;
use hyperspeed::network::*;
use hyperspeed::script::{PythonInterpreter, InterpreterResult};

#[derive(Clone, Debug)]
pub struct Message {
    count: u32
}

impl From<ClientMessage> for Message {
    fn from(_: ClientMessage) -> Message {
        Message {
            count: 0
        }
    }
}

impl Into<ClientMessage> for Message {
    fn into(self) -> ClientMessage {
        ClientMessage {

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
    Ok(())
}