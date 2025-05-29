use super::*;

#[derive(SplitFields)]
struct Unit {
    position: Position,
    velocity: Velocity,
}

struct World<F: StorageFamily> {
    units: <Unit as SplitFields<F>>::StructOf,
}

pub struct Benchmark<F: StorageFamily>(World<F>);

impl<F: StorageFamily> Benchmark<F> {
    pub fn new() -> Self {
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

        Self(world)
    }

    pub fn run(&mut self) {
        for (position, velocity) in query!(self.0.units, (&mut position, &velocity)) {
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }
}
