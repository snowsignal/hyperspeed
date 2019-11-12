use super::*;
use super::core::*;
use std::collections::{HashMap, VecDeque};

use std::net::TcpStream;

pub mod server;

// This is a comprehensive utility function collection to make your
// Hyperspeed code look nicer.


/// A macro that makes writing components easier and consistent.

#[macro_export]
macro_rules! define_component {
    ($id:ident) => {
      impl Component for $id {
        type Storage = VecStorage<Self>;
      }
    };
}

/// A shorthand macro for registering multiple components.

#[macro_export]
macro_rules! register_components {
    ($e:ident, $($c:ty),+) => {
        $(
           $e.register::<$c>();
        )+
    };
}

// Closure types

pub type StreamHandler = fn(&mut TcpStream) -> StreamData;

// Resource fetching

pub type Messages<E> = Vec<E>;

pub type ReadMessages<'a, E> = Read<'a, Messages<E>>;

pub type WriteMessages<'a, E> = Write<'a, Messages<E>>;

pub type InputMap = HashMap<String, VecDeque<Input>>;

pub type ReadInputMap<'a> = Read<'a, InputMap>;

pub type WriteInputMap<'a> = Write<'a, InputMap>;

pub type ViewMap = HashMap<String, ClientView>;

pub type ReadViewMap<'a> = Read<'a, ViewMap>;

pub type WriteViewMap<'a> = Write<'a, ViewMap>;

pub type ReadConnections<'a> = Read<'a, ConnectionCollection>;

pub type WriteConnections<'a> = Write<'a, ConnectionCollection>;