use stecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vec2<T = f32> {
    pub x: T,
    pub y: T,
}

#[derive(SplitFields)]
pub struct Brick {
    pub size: Vec2,
}

impl crate::World {
    pub fn update(&mut self, delta_time: f64) {}
}
