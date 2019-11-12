use specs::prelude::{Dispatcher, DispatcherBuilder, System};

pub struct SystemExecutor<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>
}

pub struct SystemExecutorBuilder<'a, 'b> {
    dispatcher_builder: DispatcherBuilder<'a, 'b>
}

impl<'a, 'b> SystemExecutor<'a, 'b> {
    pub fn new() -> SystemExecutorBuilder<'a, 'b> {
        SystemExecutorBuilder {
            dispatcher_builder: DispatcherBuilder::new()
        }
    }
    
    pub fn run(&mut self, world: &mut specs::World) {
        self.dispatcher.dispatch(&world.res);
    }
}

impl<'a, 'b> SystemExecutorBuilder<'a, 'b> {
    
    pub fn add_system<S>(&mut self, system: S, name: &str, dep: &[&str])
    where
        S: for<'c> System<'c> + Send + 'a {
        self.dispatcher_builder.add(system, name, dep);
    }
    pub fn build(mut self) -> SystemExecutor<'a, 'b> {
        SystemExecutor {
            dispatcher: self.dispatcher_builder.build()
        }
    }
}