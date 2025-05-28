#![allow(
    clippy::multiple_bound_locations,
    dead_code,
    non_snake_case,
    clippy::enum_variant_names
)]

pub mod filter_iter;
pub mod fragmented_iter;
pub mod simple_insert;
pub mod simple_iter;

use self::{
    filter_iter::{OptVelocity, UnitFilter, run_filter_iter},
    fragmented_iter::{
        A, B, C, D, Data, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, UnitFrag, V, W, X, Y,
        Z, run_frag_iter,
    },
    simple_insert::UnitInsert,
    simple_iter::{UnitIter, run_simple_iter},
};

use zero_ecs::{component, entity, system};

include!(concat!(env!("OUT_DIR"), "/zero_ecs.rs"));

#[component]
pub struct Position {
    x: f64,
    y: f64,
}

#[component]
pub struct Velocity {
    dx: f64,
    dy: f64,
}
