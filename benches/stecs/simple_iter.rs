use super::*;

use stecs::storage::IdGenerator;

#[derive(SplitFields)]
struct Unit {
    position: Position,
    velocity: Velocity,
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

    pub fn run_semi_manual(&mut self) {
        let query = {
            let field_0 = {
                self.0.units.ids.ids().map(|i| {
                    let r = self
                        .0
                        .units
                        .inner
                        .position
                        .get_mut(i)
                        .expect("invalid id: entry absent");
                    unsafe { &mut *(r as *mut Position) }
                })
            };
            let field_1 = self
                .0
                .units
                .ids
                .ids()
                .map(|id| self.0.units.inner.velocity.get(id));
            field_0.zip(field_1).filter_map(|(field_0, field_1)| {
                let field_1 = field_1?;
                Some((field_0, field_1))
            })
        };
        for (position, velocity) in query {
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }

    pub fn run_iter_zip(&mut self) {
        // NOTE: we can safely zip iter's only because the implementation is known
        let query = self
            .0
            .units
            .inner
            .position
            .iter_mut()
            .zip(self.0.units.inner.velocity.iter());
        for ((key_pos, position), (key_vel, velocity)) in query {
            debug_assert_eq!(key_pos, key_vel);
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }

    pub fn run_manual(&mut self) {
        let ids = self.0.units.ids.ids();
        for id in ids {
            let position = self.0.units.inner.position.get_mut(id).unwrap();
            let velocity = self.0.units.inner.velocity.get(id).unwrap();
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }

    pub fn run_manual_unchecked(&mut self) {
        let ids = self.0.units.ids.ids();
        for id in ids {
            unsafe {
                let position = self.0.units.inner.position.get_unchecked_mut(id);
                let velocity = self.0.units.inner.velocity.get_unchecked(id);
                position.x += velocity.dx;
                position.y += velocity.dy;
            }
        }
    }
}
