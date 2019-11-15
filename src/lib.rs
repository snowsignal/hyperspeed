#![allow(dead_code)]
#![feature(duration_float)]
#![feature(trait_alias)]

extern crate specs;
#[macro_use]
extern crate shred_derive;
#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate cascade;
extern crate tokio;
#[macro_use]
pub extern crate cpython;

pub mod network;
pub mod ecs;
pub mod script;

#[cfg(test)]
mod tests;