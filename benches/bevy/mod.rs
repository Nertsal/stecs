pub mod filter_iter;
pub mod fragmented_iter;
pub mod simple_insert;
pub mod simple_iter;

use bevy_ecs::prelude::*;

#[derive(Component)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Component)]
struct Velocity {
    dx: f64,
    dy: f64,
}
