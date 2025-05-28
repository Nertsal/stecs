pub mod filter_iter;
pub mod simple_insert;
pub mod simple_iter;

use gecs::prelude::*;

pub struct Position {
    x: f64,
    y: f64,
}

pub struct Velocity {
    dx: f64,
    dy: f64,
}
