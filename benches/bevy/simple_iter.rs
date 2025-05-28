use super::*;

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
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

        Self(world)
    }

    pub fn run(&mut self) {
        let mut query = self.0.query::<(&Velocity, &mut Position)>();

        for (velocity, mut position) in query.iter_mut(&mut self.0) {
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }
}
