use ecs::prelude::*;

use collection::Collection;

struct GameWorld {
    units: StructOf<Collection<Unit>>, // UnitStructOf<CollectionFamily>,
    particles: StructOf<Vec<Particle>>, // ParticleStructOf<VecFamily>,
}

#[derive(StructOf, Debug)]
struct Unit {
    // id: Id,
    health: f32,
    tick: usize,
}

#[derive(StructOf, Debug)]
struct Particle {
    time: f32,
}

#[derive(StructQuery, Debug)]
struct HealthRef<'a> {
    health: &'a mut f32,
}

#[derive(StructQuery, Debug)]
struct TickRef<'a> {
    tick: &'a mut usize,
}

fn main() {
    println!("Hello, example!");

    let mut world = GameWorld {
        units: Default::default(),
        particles: Default::default(),
    };

    world.units.insert(Unit {
        health: 10.0,
        tick: 7,
    });
    world.units.insert(Unit {
        health: 15.0,
        tick: 3,
    });

    for _ in 0..10 {
        world.particles.insert(Particle { time: 1.0 });
    }

    println!("Units:");
    for unit in world.units.iter() {
        println!("{unit:?}");
    }

    println!("\nParticles:");
    for particle in world.particles.iter() {
        println!("{particle:?}");
    }

    println!("\nHealths:");
    let mut query = query_health_ref!(world.units);
    let mut iter = query.iter_mut();
    while let Some((_, health)) = iter.next() {
        println!("Updating {health:?}");
        println!("  Inner query over ticks:");
        let mut query = query_tick_ref!(world.units);
        let mut iter = query.iter_mut();
        while let Some((_, tick)) = iter.next() {
            println!("  Incrementing {tick:?}");
            *tick.tick += 1;
        }
        *health.health -= 5.0;
    }

    println!("\nUpdated healths");
    for health in &query_health_ref!(world.units) {
        println!("{health:?}");
    }

    // Check that we still own the world
    drop(world);
}

/// Collection storage.
mod collection {
    use std::collections::HashMap;

    use ecs::prelude::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Id(u64);

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

        type Iterator<'a> = std::iter::Map<std::collections::hash_map::Iter<'a, Id, T>, fn((&Id, &'a T)) -> (Id, &'a T)>
        where
            Self: 'a,
            T: 'a;

        type IteratorMut<'a> = std::iter::Map<std::collections::hash_map::IterMut<'a, Id, T>, fn((&Id, &'a mut T)) -> (Id, &'a mut T)>
        where
            Self: 'a,
            T: 'a;

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

        fn iter(&self) -> Self::Iterator<'_> {
            self.inner.iter().map(copy_id)
        }

        fn iter_mut(&mut self) -> Self::IteratorMut<'_> {
            self.inner.iter_mut().map(copy_id_mut)
        }
    }

    fn copy_id<'a, T>((&id, v): (&Id, &'a T)) -> (Id, &'a T) {
        (id, v)
    }

    fn copy_id_mut<'a, T>((&id, v): (&Id, &'a mut T)) -> (Id, &'a mut T) {
        (id, v)
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
