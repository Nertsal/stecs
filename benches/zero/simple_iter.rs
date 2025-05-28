use super::*;

#[entity]
pub struct UnitIter {
    pub position: Position,
    pub velocity: Velocity,
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();

        for i in 0..crate::N_ENTITIES {
            world.create(UnitIter {
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
        run_simple_iter(&mut self.0, Query::new());
    }
}

#[system]
pub fn run_simple_iter(world: &mut World, query: Query<(&mut Position, &Velocity)>) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(pos, vel)| {
            pos.x += vel.dx;
            pos.y += vel.dy;
        });
}
