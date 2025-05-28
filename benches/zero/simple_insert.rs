use super::*;

#[entity]
pub(super) struct UnitInsert {
    pub position: Position,
    pub velocity: Velocity,
}

pub struct Benchmark;

impl Benchmark {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut world = World::default();

        for i in 0..crate::N_ENTITIES {
            world.create(UnitInsert {
                position: Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                velocity: Velocity { dx: 2.0, dy: 3.0 },
            });
        }
    }
}
