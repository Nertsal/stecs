#![allow(dead_code)]

use ecs::prelude::*;

#[derive(Clone)] // `StructOf` implements Clone if possible
struct GameWorld {
    units: StructOf<Vec<Unit>>,         // UnitStructOf<VecFamily>,
    corpses: StructOf<Vec<Corpse>>,     // CorpseStructOf<VecFamily>,
    particles: StructOf<Vec<Particle>>, // ParticleStructOf<VecFamily>,
}

#[derive(SplitFields, Debug)]
#[split(debug)] // derive `Debug` for the `UnitRef` generated struct
struct Unit {
    // id: Id,
    pos: (f32, f32),
    health: f32,
    tick: usize,
    damage: Option<f32>,
}

#[derive(SplitFields)]
struct Corpse {
    // Nest `Unit` to efficiently store the fields and to refer to them directly in the queries.
    // But you can still access the whole `Unit` as a single component.
    #[split(nested)]
    unit: Unit,
    time: f32,
}

#[derive(SplitFields, Debug)]
#[split(to_owned)] // implement to_owned method for the `ParticleRef` generated struct to clone the data into a `Particle`
struct Particle {
    pos: (f32, f32),
    time: f32,
}

fn main() {
    println!("Hello, example!");

    let mut world = GameWorld {
        units: Default::default(),
        corpses: Default::default(),
        particles: Default::default(),
    };

    let player_id = world.units.insert(Unit {
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

    world.corpses.insert(Corpse {
        unit: Unit {
            pos: (-4.0, 3.0),
            health: 0.0,
            tick: 10,
            damage: None,
        },
        time: 1.0,
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
    for (_, particle) in world.particles.iter() {
        let particle_cloned: Particle = particle.to_owned();
        println!("{particle_cloned:?}");
    }

    // Query fields
    {
        println!("\nPosition with tick:");

        // Declare a view struct to query into
        #[derive(Debug)]
        struct UnitRef<'a> {
            pos: &'a (f32, f32),
            tick: &'a usize,
        }

        println!("\nQuerying into a struct:");
        for unit in query!(world.units, UnitRef { pos, tick }) {
            println!("{:?}", unit);
        }

        // Or just query into a tuple
        println!("\nQuerying into a tuple:");
        for unit in query!(world.units, (&pos, &tick)) {
            println!("{:?}", unit);
        }

        // Query a single entity
        println!("\nSingle entity:");
        if let Some(player) = get!(world.units, player_id, (&pos, &tick)) {
            println!("{:?}", player);
        }
    }

    // Query an optional field
    {
        println!("\nHealth with damage:");
        for unit in query!(world.units, (&health, &damage.Get.Some)) {
            // Now we get access to units which have health *and* damage
            println!("{:?}", unit);
        }
    }

    // Splitting mutable access to components
    {
        // Iterate mutably over all units' healths
        println!("\nHealths:");
        let ids = world.units.ids();
        for &id in &ids {
            // Sadly you cant `query!` mutably, so you have to manually iterate over id's and `get!` each entity
            let (health,) = get!(world.units, id, (&mut health)).unwrap();
            println!("Updating {health:?}");

            // Iterate mutably over all units' ticks
            println!("  Inner query over ticks:");
            for &id in &ids {
                let (tick,) = get!(world.units, id, (&mut tick)).unwrap();
                println!("  Incrementing {tick:?}");
                *tick += 1;
            }

            *health -= 5.0;
        }

        // Iterate over all units' healths again
        println!("\nUpdated healths");
        for id in ids {
            let (health,) = get!(world.units, id, (&health)).unwrap();
            println!("{:?}", health);
        }
    }

    // Query from a nested storage
    {
        println!("\nTicks inside units inside corpses:");
        // `tick` is located inside `unit`
        for corpse in query!(world.corpses, (&unit.tick)) {
            println!("{:?}", corpse);
        }
    }

    // Query multiple entity types at the same time
    {
        // Declare a view struct to have the same access to both entities
        #[derive(Debug)]
        struct PosRef<'a> {
            pos: &'a (f32, f32),
        }

        println!("\nPositions of units and corpses:");
        let units = query!(world.units, PosRef { pos });
        // And we can have different access patterns for each entity
        // so we can access the position of the nested unit of the corpse
        let particles = query!(world.corpses, PosRef { pos: &unit.pos });

        for item in units.chain(particles) {
            println!("{:?}", item);
        }
    }

    // Query the whole nested storage
    {
        println!("\nNested units:");
        for item in query!(world.corpses, (&unit)) {
            println!("{:?}", item);
        }
    }

    println!("\nTaking back ownership of all units:");
    for unit in world.units.into_iter() {
        println!("{unit:?}");
    }
}
