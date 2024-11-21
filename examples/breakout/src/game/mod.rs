use crate::{logic, render};

use macroquad::prelude::*;
use stecs::prelude::*;

#[derive(World, Default)]
pub struct World {
    pub resources: Resources,
    pub bricks: StructOf<Dense<logic::Brick>>,
    pub render_bricks: SplitOf<Dense<render::Brick>>,
}

pub struct Resources {
    pub camera: Camera2D,
    pub bounds: Rect,
}

impl Default for Resources {
    fn default() -> Self {
        const ASPECT: f32 = 16.0 / 9.0;
        const VERT_FOV: f32 = 20.0;
        Self {
            camera: Camera2D::from_display_rect(Rect::new(
                -0.5 * VERT_FOV * ASPECT,
                -0.5 * VERT_FOV,
                VERT_FOV * ASPECT,
                VERT_FOV,
            )),
            bounds: Rect::new(-15.0, -7.0, 30.0, 15.0),
        }
    }
}

impl World {
    pub fn new() -> Self {
        let mut world = Self::default();
        world.init();
        world
    }

    fn init(&mut self) {
        const AMOUNT_X: usize = 10;
        const AMOUNT_Y: usize = 5;
        const WIDTH: f32 = 3.0;
        const HEIGHT: f32 = 1.0;
        for y in 0..AMOUNT_Y {
            for x in 0..AMOUNT_X {
                let size = vec2(WIDTH, HEIGHT);
                let position =
                    vec2(x as f32 - AMOUNT_X.saturating_sub(1) as f32 / 2.0, y as f32) * size;
                let halfsize = size / 2.0;

                let brick = self.bricks.insert(logic::Brick { position, halfsize });

                let color = Color::new(
                    macroquad::rand::gen_range(0.4, 0.6),
                    macroquad::rand::gen_range(0.4, 0.6),
                    macroquad::rand::gen_range(0.4, 0.6),
                    1.0,
                );
                self.render_bricks.insert(brick, render::Brick { color });
            }
        }
    }

    pub async fn run(mut self) {
        loop {
            self.update(get_frame_time());
            clear_background(Color::new(0.0, 0.0, 0.0, 1.0));
            self.draw();
            next_frame().await
        }
    }
}
