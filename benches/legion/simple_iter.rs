use super::*;

pub struct Benchmark(World, Query<(Write<Position>, Read<Velocity>)>);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();

        world.extend((0..crate::N_ENTITIES).map(|i| {
            (
                Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                Velocity { dx: 2.0, dy: 3.0 },
            )
        }));

        let query = <(Write<Position>, Read<Velocity>)>::query();

        Self(world, query)
    }

    pub fn run(&mut self) {
        self.1.for_each_mut(&mut self.0, |(position, velocity)| {
            position.x += velocity.dx;
            position.y += velocity.dy;
        });
    }
}
