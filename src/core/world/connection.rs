use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub struct ConnectionCollection {
    new_keys: VecDeque<String>,
    pub connections: Vec<Connection>
}

// Sharing connections can be hard, because both the ECS system and the server
// need to read/write to the client views. In this case, the world's connection collection
// is passed immutably to the server, which sends data to each respective channel.
#[derive(Clone, Debug, Default)]
pub struct Connection {
    pub key: String
}

#[derive(Clone, Debug, Serialize)]
pub struct ClientView {
    pub sprites: Vec<u64>,
    pub loc: Vec<(f32, f32)>
}

impl ConnectionCollection {
    pub fn new() -> Self {
        ConnectionCollection {
            new_keys: VecDeque::new(),
            connections: vec![]
        }
    }

    pub fn size(&self) -> usize {
        self.connections.len()
    }

    pub fn pop_new_key(&mut self) -> Option<String> {
        self.new_keys.pop_front()
    }

    pub fn pop_new_keys(&mut self) -> Vec<String> {
        let mut keys = vec!();
        loop {
            let key = self.pop_new_key();
            if key.is_some() {
                keys.push(key.unwrap());
            } else {
                break;
            }
        }
        self.new_keys.clear();
        keys
    }

    pub fn remove(&mut self, key: &String) {
        self.connections.retain(|x| x.key == *key);
    }

    pub fn push(&mut self, c: Connection) {
        self.new_keys.push_back(c.key.clone());
        self.connections.push(c);
    }
}

impl ClientView {
    pub fn new() -> Self {
        ClientView {
            sprites: vec!(),
            loc: vec!()
        }
    }
}