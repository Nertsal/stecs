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

// #[derive(StructQuery, Debug)]
// #[query(base = "Unit")]
#[derive(Debug)]
struct UnitRef<'a> {
    health: &'a f32,
    tick: &'a usize,
}

struct UnitRefComponents<'a, F: StorageFamily> {
    health: &'a F::Storage<f32>,
    tick: &'a F::Storage<usize>,
}

impl<'b, F: StorageFamily> QueryComponents<F> for UnitRefComponents<'b, F> {
    type Item<'a> = UnitRef<'a>
    where
        Self: 'a;

    fn ids(&self) -> F::IdIter {
        self.health.ids()
    }

    fn get(&self, id: F::Id) -> Option<Self::Item<'_>> {
        let health = self.health.get(id)?;
        let tick = self.tick.get(id)?;
        Some(UnitRef { health, tick })
    }
}

impl<'b, F: StorageFamily + 'static> StructQuery<F> for UnitRef<'b> {
    type Components<'a> = UnitRefComponents<'a, F>;
}

// #[derive(StructQuery, Debug)]
// #[query(base = "Particle")]
// struct ParticleRef<'a> {
//     time: &'a f32,
// }

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

    let components = query_components!(world.units, UnitRefComponents, (health, tick));
    let query: Query<_, collection::CollectionFamily> = UnitRef::query(components);
    // let query = query!(world.units, UnitRef);
    for unit in query.iter() {
        println!("{unit:?}");
    }

    // println!("Units:");
    // for (id, unit) in UnitRef::query(&world.units).enumerate() {
    //     println!("{id:?}: {unit:?}");
    // }

    // println!("\nParticles:");
    // for (i, particle) in ParticleRef::query(&world.particles).enumerate() {
    //     println!("{i:02}: {particle:?}");
    // }

    // println!("\nHealths:");
    // for (i, health) in query!(world.units, (mut health)).enumerate() {
    //     println!("{i:02}: {health:?}");
    //     println!("  Inner query over ticks:");
    //     for (j, tick) in query!(world.units, (mut tick)).enumerate() {
    //         println!("  {j:02}: {tick:?}");
    //     }
    // }

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
        type Id = Id;

        type IdIter = std::vec::IntoIter<Id>;

        type Iterator<'a> = std::collections::hash_map::Values<'a, Id, T>
        where
            Self: 'a,
            T: 'a;

        type IteratorMut<'a> = std::collections::hash_map::ValuesMut<'a, Id, T>
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
            self.inner.values()
        }

        fn iter_mut(&mut self) -> Self::IteratorMut<'_> {
            self.inner.values_mut()
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
