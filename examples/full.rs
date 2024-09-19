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
#[split(clone)] // implement to_owned method for the `ParticleRef` generated struct to clone the data into a `Particle`
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
    for (_id, unit) in world.units.iter() {
        println!("{unit:?}");
    }

    // Iterate over all fields of all particles
    println!("\nParticles:");
    for (_id, particle) in world.particles.iter() {
        let particle_cloned: Particle = particle.clone();
        println!("{particle_cloned:?}");
    }

    // Query fields
    {
        println!("\nPosition with tick:");

        // Declare a view struct to query into
        #[derive(Debug)]
        struct UnitRef<'a> {
            id: usize,
            pos: &'a (f32, f32),
            tick: &'a usize,
        }

        println!("\nQuerying into a struct:");
        for unit in query!(world.units, UnitRef { id, pos, tick }) {
            println!("{:?}", unit);
        }

        // Or just query into a tuple
        println!("\nQuerying into a tuple:");
        for unit in query!(world.units, (&id, &pos, &tick)) {
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
        for health in query!(world.units, (&mut health)) {
            println!("Updating {health:?}");

            // Iterate mutably over all units' ticks
            println!("  Inner query over ticks:");
            for tick in query!(world.units, (&mut tick)) {
                println!("  Incrementing {tick:?}");
                *tick += 1;
            }

            *health -= 5.0;
        }

        // Iterate over all units' healths again
        println!("\nUpdated healths");
        for health in query!(world.units, (&health)) {
            println!("{:?}", health);
        }
    }

    // Query from a nested storage
    {
        println!("\nTicks inside units inside corpses:");
        // `tick` is located inside `unit`
        for tick in query!(world.corpses, (&unit.tick)) {
            println!("{:?}", tick);
        }
    }

    // Combine queries of multiple entity types
    {
        println!("\nPositions of units and corpses:");
        let units = query!(world.units, (&pos));
        // And we can have different access patterns for each entity
        // so we can access the position of the nested unit of the corpse
        let corpses = query!(world.corpses, (&unit.pos));

        for pos in units.chain(corpses) {
            println!("{:?}", pos);
        }
    }

    // Query structurally similar types in a single query
    {
        println!("\nPositions of units and particles incremented:");
        for pos in query!([world.units, world.particles], (&mut pos)) {
            pos.0 += 1.0;
            println!("{:?}", pos);
        }
    }

    // Query the whole nested storage
    {
        println!("\nNested units:");
        for unit in query!(world.corpses, (&unit)) {
            println!("{:?}", unit);
        }
    }

    println!("\nTaking back ownership of all units:");
    for unit in world.units {
        println!("{unit:?}");
    }
}
