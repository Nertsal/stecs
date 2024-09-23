use criterion::{criterion_group, criterion_main, Criterion};
use stecs::prelude::*;

/// Entities with position and velocity.
pub const N_POS_VEL: usize = 10000;

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
    velocity: Velocity,
}

struct World {
    units: StructOf<Vec<Unit>>,
}

fn build() -> World {
    let mut world = World {
        units: Default::default(),
    };

    for _ in 0..N_POS_VEL {
        world.units.insert(Unit {
            position: Position { x: 0.0, y: 0.0 },
            velocity: Velocity { dx: 0.0, dy: 0.0 },
        });
    }

    world
}

fn process(world: &mut World) {
    for (position, velocity) in query!(world.units, (&mut position, &velocity)) {
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
                    .position
                    .get_mut(i)
                    .expect("invalid id: entry absent");
                unsafe { &mut *(r as *mut Position) }
            })
        };
        let field_1 = world.units.ids.ids().map(|id| world.units.velocity.get(id));
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
        .position
        .iter_mut()
        .zip(world.units.velocity.iter());
    for (position, velocity) in query {
        position.x += velocity.dx;
        position.y += velocity.dy;
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
    simple,
    bench_build,
    bench_process,
    bench_semi_manual,
    bench_manual
);
criterion_main!(simple);
