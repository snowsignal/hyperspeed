mod world;
use specs::prelude::*;
use world::*;
use tokio::prelude::*;
use tokio::sync::mpsc::Sender;

pub struct GameUpdate();
/*
pub struct GameBuilder {

}

pub struct Game {
    tx: Sender<GameUpdate>,
    rx: Receiver<ClientInput>,

}

pub struct GameTickFuture<'a> {
    game: &'a mut Game
}

impl Future for GameTickFuture {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        // Get client input
        let client_input = self.game.server_client_input();
        self.game.execute_systems();
        self.game.tx.send
    }
}

impl Game {
    pub fn tick<'a>(& mut self) -> GameTickFuture<'a> {

    }

    pub fn server_client_input(&mut self) -> ClientInput {

    }
}
*/