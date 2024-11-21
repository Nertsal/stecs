use crate::game::World;

use macroquad::prelude::*;
use stecs::prelude::*;

#[derive(SplitFields)]
pub struct Brick {
    pub color: Color,
}

impl World {
    pub fn draw(&mut self) {
        set_camera(&self.resources.camera);

        let bounds = &self.resources.bounds;
        draw_rectangle(
            bounds.x,
            bounds.y,
            bounds.w,
            bounds.h,
            Color::new(0.1, 0.1, 0.1, 1.0),
        );

        for (position, halfsize, &color) in query!(
            self.bricks,
            (&position, &halfsize, &foreign self.render_bricks color)
        ) {
            draw_rectangle(
                position.x - halfsize.x,
                position.y - halfsize.y,
                halfsize.x * 2.0,
                halfsize.y * 2.0,
                color,
            );
        }
    }
}
