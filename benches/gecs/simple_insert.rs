use super::*;

ecs_world! {
    ecs_archetype!(ArchUnitSInsert, crate::N_ENTITIES, Position, Velocity);
}

pub struct Benchmark;

impl Benchmark {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut world = EcsWorld::new();

        for i in 0..crate::N_ENTITIES {
            world.create((
                Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                Velocity { dx: 2.0, dy: 3.0 },
            ));
        }
    }
}
