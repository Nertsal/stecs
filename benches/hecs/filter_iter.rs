use super::*;

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::new();
        world.spawn_batch((0..crate::N_ENTITIES_FILTER_VEL).map(|i| {
            (
                Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                Velocity { dx: 2.0, dy: 3.0 },
            )
        }));
        world.spawn_batch(
            (crate::N_ENTITIES_FILTER_VEL..crate::N_ENTITIES_FILTER).map(|i| {
                (Position {
                    x: i as f64,
                    y: -(i as f64),
                },)
            }),
        );
        Self(world)
    }

    pub fn run(&mut self) {
        for (_, (position, velocity)) in self.0.query_mut::<(&mut Position, &Velocity)>() {
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }
}
