use super::*;

#[derive(SplitFields)]
struct Unit {
    position: Position,
    velocity: Velocity,
}

struct World {
    units: StructOf<SlotMap<slotmap::DefaultKey, Unit>>,
}

pub struct Benchmark;

impl Benchmark {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut world = World {
            units: Default::default(),
        };

        for i in 0..crate::N_ENTITIES {
            world.units.insert(Unit {
                position: Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                velocity: Velocity { dx: 2.0, dy: 3.0 },
            });
        }

        let _ = world;
    }
}
