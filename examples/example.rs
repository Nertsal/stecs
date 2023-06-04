#![allow(dead_code)]

use ecs::prelude::*;

use collection::Collection;

#[derive(Clone)] // `StructOf` implements Clone if possible
struct GameWorld {
    units: StructOf<Collection<Unit>>, // UnitStructOf<CollectionFamily>,
    corpses: StructOf<Vec<Corpse>>,    // CorpseStructOf<VecFamily>,
    particles: StructOf<Vec<Particle>>, // ParticleStructOf<VecFamily>,
}

#[derive(StructOf, Debug, Clone)]
struct Unit {
    // id: Id,
    pos: (f32, f32),
    health: f32,
    tick: usize,
    damage: Option<f32>,
}

#[derive(StructOf, Debug)]
struct Corpse {
    // Nest `Unit` to efficiently store the fields and to refer to them directly in the queries.
    #[structof(nested)]
    unit: Unit,
    time: f32,
}

#[derive(StructOf, Debug)]
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
        #[derive(StructQuery, Debug)]
        struct PosTickRef<'a> {
            pos: &'a (f32, f32),
            tick: &'a usize,
        }

        println!("\nPosition with tick:");
        for item in &query_pos_tick_ref!(world.units) {
            println!("{item:?}");
        }
    }

    // Query an optional field
    {
        #[derive(StructQuery, Debug)]
        struct HealthDamageRef<'a> {
            health: &'a f32,
            // query from a component of type `Option<f32>` with value `Some(damage)`
            #[query(component = "._Some")]
            damage: &'a f32,
        }

        println!("\nHealth with damage:");
        for item in &query_health_damage_ref!(world.units) {
            println!("{item:?}");
        }
    }

    // Splitting mutable access to components
    {
        #[derive(StructQuery, Debug)]
        struct HealthRef<'a> {
            health: &'a mut f32,
        }

        #[derive(StructQuery, Debug)]
        struct TickRef<'a> {
            tick: &'a mut usize,
        }

        // Iterate mutably over all units' healths
        println!("\nHealths:");
        let mut query = query_health_ref!(world.units);
        let mut iter = query.iter_mut();
        while let Some((_, health)) = iter.next() {
            println!("Updating {health:?}");

            // Iterate mutably over all units' ticks
            println!("  Inner query over ticks:");
            let mut query = query_tick_ref!(world.units);
            let mut iter = query.iter_mut();
            while let Some((_, tick)) = iter.next() {
                println!("  Incrementing {tick:?}");
                *tick.tick += 1;
            }

            *health.health -= 5.0;
        }

        // Iterate over all units' healths again
        println!("\nUpdated healths");
        for health in &query_health_ref!(world.units) {
            println!("{health:?}");
        }
    }

    // Query multiple entity types at the same time
    {
        #[derive(StructQuery, Debug)]
        struct PosRef<'a> {
            pos: &'a (f32, f32),
        }

        println!();
        let units = query_pos_ref!(world.units);
        let particles = query_pos_ref!(world.particles);
        for pos in units.values().chain(particles.values()) {
            println!("{pos:?}");
        }
    }

    // Query from a nested storage
    {
        #[derive(StructQuery, Debug)]
        struct TickRef<'a> {
            #[query(storage = ".unit.tick")]
            tick: &'a usize,
            time: &'a mut f32,
        }

        println!();
        let corpses = query_tick_ref!(world.corpses);
        for tick in corpses.values() {
            println!("{tick:?}");
        }
    }

    println!("\nTaking back ownership of all units:");
    for unit in world.units.inner.into_iter() {
        println!("{unit:?}");
    }
}

/// Collection storage.
mod collection {
    use std::collections::HashMap;

    use ecs::prelude::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Id(u64);

    #[derive(Clone)]
    pub struct Collection<T> {
        next_id: Id,
        inner: HashMap<Id, T>,
    }

    impl<T> Default for Collection<T> {
        fn default() -> Self {
            Self {
                next_id: Id(0),
                inner: Default::default(),
            }
        }
    }

    impl<T> Storage<T> for Collection<T> {
        type Family = CollectionFamily;
        type Id = Id;
        type IdIter = std::vec::IntoIter<Id>;

        fn ids(&self) -> Self::IdIter {
            self.inner.keys().copied().collect::<Vec<_>>().into_iter()
        }

        fn insert(&mut self, value: T) -> Self::Id {
            let id = self.next_id;
            self.next_id.0 += 1;
            let res = self.inner.insert(id, value);
            assert!(
                res.is_none(),
                "Failed to generate a unique id in a collection"
            );
            id
        }

        fn get(&self, id: Self::Id) -> Option<&T> {
            self.inner.get(&id)
        }

        fn get_mut(&mut self, id: Self::Id) -> Option<&mut T> {
            self.inner.get_mut(&id)
        }

        fn remove(&mut self, id: Self::Id) -> Option<T> {
            self.inner.remove(&id)
        }

        fn iter(&self) -> Box<dyn Iterator<Item = (Self::Id, &T)> + '_> {
            Box::new(self.inner.iter().map(|(&id, v)| (id, v)))
        }

        fn iter_mut(&mut self) -> Box<dyn Iterator<Item = (Self::Id, &mut T)> + '_> {
            Box::new(self.inner.iter_mut().map(|(&id, v)| (id, v)))
        }
    }

    pub struct CollectionFamily;

    impl StorageFamily for CollectionFamily {
        type Id = Id;
        type IdIter = std::vec::IntoIter<Id>;
        type Storage<T> = Collection<T>;
    }

    impl<T: SplitFields<CollectionFamily>> StructOfAble for Collection<T> {
        type Struct = T;
        type Family = CollectionFamily;
    }
}
