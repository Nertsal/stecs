pub mod filter_iter;
pub mod fragmented_iter;
pub mod simple_insert;
pub mod simple_iter;

use slotmap::SlotMap;
use stecs::prelude::*;

struct Position {
    x: f64,
    y: f64,
}

struct Velocity {
    dx: f64,
    dy: f64,
}
