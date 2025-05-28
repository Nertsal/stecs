use super::*;

#[component]
pub struct OptVelocity(Option<Velocity>);

#[entity]
pub struct UnitFilter {
    pub position: Position,
    pub velocity: OptVelocity,
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();

        for i in 0..crate::N_ENTITIES_FILTER {
            let velocity =
                (i < crate::N_ENTITIES_FILTER_VEL).then_some(Velocity { dx: 2.0, dy: 3.0 });
            world.create(UnitFilter {
                position: Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                velocity: OptVelocity(velocity),
            });
        }

        Self(world)
    }

    pub fn run(&mut self) {
        run_filter_iter(&mut self.0, Query::new());
    }
}

#[system]
pub fn run_filter_iter(world: &mut World, query: Query<(&mut Position, &OptVelocity)>) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(pos, vel)| {
            if let Some(vel) = &vel.0 {
                pos.x += vel.dx;
                pos.y += vel.dy;
            }
        });
}
