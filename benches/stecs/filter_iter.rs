use super::*;

#[derive(SplitFields)]
struct Unit {
    position: Position,
    velocity: Option<Velocity>,
}

struct World {
    units: StructOf<SlotMap<slotmap::DefaultKey, Unit>>,
}

pub struct Benchmark(World);

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World {
            units: Default::default(),
        };

        for i in 0..crate::N_ENTITIES_FILTER {
            let velocity =
                (i < crate::N_ENTITIES_FILTER_VEL).then_some(Velocity { dx: 2.0, dy: 3.0 });
            world.units.insert(Unit {
                position: Position {
                    x: i as f64,
                    y: -(i as f64),
                },
                velocity,
            });
        }

        Self(world)
    }

    pub fn run(&mut self) {
        for (position, velocity) in query!(self.0.units, (&mut position, &velocity.Get.Some)) {
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }
}
