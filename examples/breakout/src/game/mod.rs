use miniquad::{window, RenderingBackend};

pub struct Stage {
    ctx: Box<dyn RenderingBackend>,
    last_time: f64,
    world: crate::World,
}

impl Stage {
    pub fn new() -> Self {
        Self {
            ctx: window::new_rendering_backend(),
            last_time: 0.0,
            world: crate::World::new(),
        }
    }
}

impl miniquad::EventHandler for Stage {
    fn update(&mut self) {
        let time = miniquad::date::now();
        let delta_time = time - self.last_time;
        self.last_time = time;
        self.world.update(delta_time);
    }

    fn draw(&mut self) {
        self.ctx.begin_default_pass(Default::default());
        self.world.draw(&mut self.ctx);
        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }
}
