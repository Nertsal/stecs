use super::*;

pub struct OptVelocity(Option<Velocity>);

ecs_world! {
    ecs_archetype!(ArchUnitFilter, crate::N_ENTITIES_FILTER, Position, OptVelocity);
}

pub struct Benchmark(EcsWorld);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = EcsWorld::new();

        for i in 0..crate::N_ENTITIES {
            let velocity =
                (i < crate::N_ENTITIES_FILTER_VEL).then_some(Velocity { dx: 2.0, dy: 3.0 });
            world.create((
                Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                OptVelocity(velocity),
            ));
        }

        Self(world)
    }

    pub fn run(&mut self) {
        ecs_iter!(self.0, |pos: &mut Position, vel: &OptVelocity| {
            if let Some(vel) = &vel.0 {
                pos.x += vel.dx;
                pos.y += vel.dy;
            }
        });
    }
}
