mod connection;
mod input;
mod mc;
mod system;
mod blueprint;

pub use connection::{ConnectionCollection, Connection, ClientView};
pub use input::Input;
pub use mc::{MasterController, EngineInstruction};
pub use system::{SystemExecutor, SystemExecutorBuilder};

pub struct World<'a, 'b> {
    pub(crate) system_executor: SystemExecutor<'a, 'b>,
    pub ecs_world: specs::prelude::World,
    pub connections: ConnectionCollection,
}