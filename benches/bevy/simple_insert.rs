use super::*;

pub struct Benchmark;

impl Benchmark {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut world = World::new();
        world.spawn_batch((0..crate::N_ENTITIES).map(|i| {
            (
                Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                Velocity { dx: 2.0, dy: 3.0 },
            )
        }));

        let _ = world;
    }
}
