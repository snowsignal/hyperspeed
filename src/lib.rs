#![allow(dead_code)]
#![feature(try_from)]
#![feature(trait_alias)]

extern crate specs;
#[macro_use]
extern crate shred_derive;
#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate cascade;
extern crate tokio;
extern crate bytes;
#[macro_use]
pub extern crate cpython;

pub mod network;
pub mod ecs;
pub mod script;

#[cfg(test)]
mod tests;