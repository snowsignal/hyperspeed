use specs::{Entity, World};

pub trait Blueprint {
    fn add_to_world(mut self, _w: &mut World);
}