extern crate hyperspeed;
use hyperspeed::network::*;
use hyperspeed::script::{PythonBackend, ScriptBackend};

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


fn main() {}