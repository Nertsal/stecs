use criterion::{criterion_group, criterion_main, Criterion};
use stecs::prelude::*;

/// Entities with velocity and position component.
pub const N_POS_PER_VEL: usize = 10;

/// Entities with position component only.
pub const N_POS: usize = 10000;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Velocity {
    dx: f64,
    dy: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, SplitFields)]
struct Unit {
    position: Position,
    velocity: Option<Velocity>,
}

struct World {
    units: StructOf<Arena<Unit>>,
}

fn build() -> World {
    let mut world = World {
        units: Default::default(),
    };

    for i in 0..N_POS {
        let velocity = (i % N_POS_PER_VEL == 0).then_some(Velocity { dx: 0.0, dy: 0.0 });
        world.units.insert(Unit {
            position: Position { x: 0.0, y: 0.0 },
            velocity,
        });
    }

    world
}

fn process(world: &mut World) {
    for (position, velocity) in query!(world.units, (&mut position, &velocity.Get.Some)) {
        position.x += velocity.dx;
        position.y += velocity.dy;
    }
}

fn semi_manual_process(world: &mut World) {
    let query = {
        let field_0 = {
            world.units.ids.ids().map(|i| {
                let r = world
                    .units
                    .inner
                    .position
                    .get_mut(i)
                    .expect("invalid id: entry absent");
                unsafe { &mut *(r as *mut Position) }
            })
        };
        let field_1 = world
            .units
            .ids
            .ids()
            .map(|id| match world.units.inner.velocity.get(id) {
                None => None,
                Some(value) => value.as_ref(),
            });
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

fn manual_process(world: &mut World) {
    // NOTE: we can safely zip iter's only because the implementation is known
    let query = world
        .units
        .inner
        .position
        .iter_mut()
        .zip(world.units.inner.velocity.iter());
    for ((key_pos, position), (key_vel, velocity)) in query {
        debug_assert_eq!(key_pos, key_vel);
        if let Some(velocity) = velocity {
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }
}

fn bench_build(c: &mut Criterion) {
    c.bench_function("build", |b| b.iter(build));
}

fn bench_process(c: &mut Criterion) {
    let mut world = build();
    c.bench_function("process", |b| b.iter(|| process(&mut world)));
}

fn bench_manual(c: &mut Criterion) {
    let mut world = build();
    c.bench_function("manual", |b| b.iter(|| manual_process(&mut world)));
}

fn bench_semi_manual(c: &mut Criterion) {
    let mut world = build();
    c.bench_function("semi_manual", |b| {
        b.iter(|| semi_manual_process(&mut world))
    });
}

criterion_group!(
    pos_vel,
    bench_build,
    bench_process,
    bench_semi_manual,
    bench_manual
);
criterion_main!(pos_vel);
