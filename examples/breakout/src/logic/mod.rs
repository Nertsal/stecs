use crate::game::World;

use macroquad::math::Vec2;
use stecs::prelude::*;

#[derive(SplitFields)]
pub struct Brick {
    pub position: Vec2,
    pub halfsize: Vec2,
}

#[derive(SplitFields)]
pub struct Ball {
    pub position: Vec2,
    pub radius: Vec2,
}

#[derive(SplitFields)]
pub struct Player {
    pub position: Vec2,
    pub halfsize: Vec2,
}

impl World {
    pub fn update(&mut self, delta_time: f32) {}
}
