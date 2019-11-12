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
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bytes;
extern crate serde_json;

pub mod core;
pub mod utils;
pub mod systems;
pub mod components;

pub use specs::prelude::*;

pub use self::core::*;

pub use self::utils::*;
