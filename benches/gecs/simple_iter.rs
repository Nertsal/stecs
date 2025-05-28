use super::*;

ecs_world! {
    ecs_archetype!(ArchUnitSIter, crate::N_ENTITIES, Position, Velocity);
}

pub struct Benchmark(EcsWorld);

impl Benchmark {
    pub fn new() -> Self {
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

        Self(world)
    }

    pub fn run(&mut self) {
        ecs_iter!(self.0, |pos: &mut Position, vel: &Velocity| {
            pos.x += vel.dx;
            pos.y += vel.dy;
        });
    }
}
