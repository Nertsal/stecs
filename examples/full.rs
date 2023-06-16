#![allow(dead_code)]

use ecs::prelude::*;

#[derive(Clone)] // `StructOf` implements Clone if possible
struct GameWorld {
    units: StructOf<Vec<Unit>>,         // UnitStructOf<VecFamily>,
    corpses: StructOf<Vec<Corpse>>,     // CorpseStructOf<VecFamily>,
    particles: StructOf<Vec<Particle>>, // ParticleStructOf<VecFamily>,
}

#[derive(SplitFields, Debug, Clone)]
struct Unit {
    // id: Id,
    pos: (f32, f32),
    health: f32,
    tick: usize,
    damage: Option<f32>,
}

#[derive(SplitFields, Debug)]
struct Corpse {
    // Nest `Unit` to efficiently store the fields and to refer to them directly in the queries.
    // But you can still access the whole `Unit` as a single component.
    #[split(nested)]
    unit: Unit,
    time: f32,
}

#[derive(SplitFields, Debug)]
struct Particle {
    pos: (f32, f32),
    time: f32,
}

fn main() {
    println!("Hello, example!");

    let mut world = GameWorld {
        units: StructOf::new(),
        corpses: StructOf::new(),
        particles: StructOf::new(),
    };

    world.units.insert(Unit {
        pos: (0.0, 0.0),
        health: 10.0,
        tick: 7,
        damage: None,
    });
    world.units.insert(Unit {
        pos: (1.0, -2.0),
        health: 15.0,
        tick: 3,
        damage: Some(1.5),
    });

    for _ in 0..3 {
        world.particles.insert(Particle {
            pos: (1.0, -0.5),
            time: 1.0,
        });
    }

    // Iterate over all fields of all units
    println!("Units:");
    for unit in world.units.iter() {
        println!("{unit:?}");
    }

    // Iterate over all fields of all particles
    println!("\nParticles:");
    for particle in world.particles.iter() {
        println!("{particle:?}");
    }

    // Query fields
    {
        // #[derive(StructQuery, Debug)]
        // struct PosTickRef<'a> {
        //     pos: &'a (f32, f32),
        //     tick: &'a usize,
        // }

        println!("\nPosition with tick:");
        // for item in &query_pos_tick_ref!(world.units) {
        //     println!("{item:?}");
        // }
        // for item in &storage_zip!(world.units, { pos, tick }) {
        //     println!("{item:?}");
        // }
        // for item in world.units.ids().filter_map(|id| {
        //     let pos = world.units.inner.pos.get(id)?;
        //     let tick = world.units.inner.tick.get(id)?;
        //     Some(structx! { pos, tick })
        // }) {
        //     println!("{item:?}");
        // }

        // Declare a view struct to query into
        #[derive(Debug)]
        struct UnitRef<'a> {
            pos: &'a (f32, f32),
            tick: &'a usize,
        }

        println!("\nQuerying into a struct:");
        for id in world.units.ids() {
            let item = get!(world.units, id, UnitRef { pos, tick });
            let Some(UnitRef {pos, tick}) = item else { continue };
            println!("{:?}, {:?}", pos, tick);
        }

        // Or just query into a tuple
        println!("\nQuerying into a tuple:");
        for id in world.units.ids() {
            let item = get!(world.units, id, (pos, tick));
            let Some((pos, tick)) = item else { continue };
            println!("{:?}, {:?}", pos, tick);
        }
    }

    // Query an optional field
    {
        println!("\nHealth with damage:");
        for id in world.units.ids() {
            let item = get!(world.units, id, (health, damage.Get.Some));
            let Some((health, damage)) = item else { continue };
            // Now we get access to units which have health *and* damage
            println!("{:?}, {:?}", health, damage);
        }
    }

    // Splitting mutable access to components
    {
        // Iterate mutably over all units' healths
        println!("\nHealths:");
        let ids = world.units.ids();
        for &id in &ids {
            let item = get!(world.units, id, (mut health));
            let Some((health,)) = item else { continue };
            println!("Updating {health:?}");

            // Iterate mutably over all units' ticks
            println!("  Inner query over ticks:");
            for &id in &ids {
                let item = get!(world.units, id, (mut tick));
                let Some((tick,)) = item else { continue };
                println!("  Incrementing {tick:?}");
                *tick += 1;
            }

            *health -= 5.0;
        }

        // Iterate over all units' healths again
        println!("\nUpdated healths");
        for id in ids {
            let item = get!(world.units, id, (health));
            let Some((health,)) = item else { continue };
            println!("{:?}", health);
        }
    }

    // // Query multiple entity types at the same time
    // {
    //     #[derive(StructQuery, Debug)]
    //     struct PosRef<'a> {
    //         pos: &'a (f32, f32),
    //     }

    //     println!();
    //     let units = query_pos_ref!(world.units);
    //     let particles = query_pos_ref!(world.particles);
    //     for pos in units.values().chain(particles.values()) {
    //         println!("{pos:?}");
    //     }
    // }

    // Query from a nested storage
    {
        // #[derive(StructQuery, Debug)]
        // struct TickRef<'a> {
        //     #[query(storage = ".unit")] // same as `optic = ".unit.tick._get"`
        //     tick: &'a usize,
        //     time: &'a mut f32,
        // }

        println!();
        // let corpses = query_tick_ref!(world.corpses);
        // for tick in corpses.values() {
        //     println!("{tick:?}");
        // }

        for id in world.corpses.ids() {
            let item = get!(world.corpses, id, (unit.tick, mut time));
            let Some((tick, time)) = item else { continue };
            println!("{:?}, {:?}", tick, time);
        }
    }

    // // Query the whole nested storage
    // {
    //     #[derive(StructQuery, Debug)]
    //     struct UnitRef<'a> {
    //         #[query(nested)]
    //         unit: &'a mut Unit,
    //     }

    //     println!();
    //     let corpses = query_unit_ref!(world.corpses);
    //     for tick in corpses.values() {
    //         println!("{tick:?}");
    //     }
    // }

    println!("\nTaking back ownership of all units:");
    for unit in world.units.inner.into_iter() {
        println!("{unit:?}");
    }
}
