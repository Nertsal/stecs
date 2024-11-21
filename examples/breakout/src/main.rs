mod game;
mod logic;
mod render;

use stecs::prelude::*;

#[derive(World, Default)]
struct World {
    bricks: StructOf<Dense<logic::Brick>>,
    render_bricks: SplitOf<Dense<render::Brick>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
}

fn main() {
    let conf = miniquad::conf::Conf::default();
    miniquad::start(conf, move || Box::new(game::Stage::new()))
}
