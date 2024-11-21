use crate::logic;

use miniquad::RenderingBackend;
use stecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(SplitFields)]
pub struct Brick {
    color: Color,
}

impl crate::World {
    pub fn draw(&mut self, ctx: &mut Box<dyn RenderingBackend>) {
        for (size, color) in query!(
            self.bricks,
            (&size, &foreign self.render_bricks color)
        ) {}
    }
}
