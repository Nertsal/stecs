use ecs::*;

use collection::Collection;

/// Collection storage.
mod collection {
    use std::collections::HashMap;

    use ecs::*;

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

        type Iterator<'a> = std::collections::hash_map::Values<'a, Id, T>
        where
            Self: 'a,
            T: 'a;

        type IteratorMut<'a> = std::collections::hash_map::ValuesMut<'a, Id, T>
        where
            Self: 'a,
            T: 'a;

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
            todo!()
        }

        fn iter_mut(&mut self) -> Self::IteratorMut<'_> {
            todo!()
        }
    }

    pub struct CollectionFamily;

    impl StorageFamily for CollectionFamily {
        type Id = Id;
        type Storage<T> = Collection<T>;
    }

    impl<T: SplitFields> StructOfAble for Collection<T> {
        type Struct = T;
        type Family = CollectionFamily;
    }
}

// -- Example --

struct GameWorld {
    units: StructOf<Collection<Unit>>, // UnitStructOf<CollectionFamily>,
    particles: StructOf<Vec<Particle>>, // ParticleStructOf<VecFamily>,
}

// #[derive(StructOf)]
#[derive(Debug)]
struct Unit {
    // id: Id,
    health: f32,
}

// #[derive(StructOf)]
#[derive(Debug)]
struct Particle {
    time: f32,
}

fn main() {
    println!("Hello, example!");

    let mut world = GameWorld {
        units: Default::default(),
        particles: Default::default(),
    };

    world.units.insert(Unit { health: 10.0 });
    world.units.insert(Unit { health: 15.0 });

    for _ in 0..10 {
        world.particles.insert(Particle { time: 1.0 });
    }

    // for unit in world.units.iter() {
    //     println!("unit: {unit:?}");
    // }
}

// -- TODO: derive --

impl SplitFields for Unit {
    type StructOf<F: StorageFamily> = UnitStructOf<F>;
}

struct UnitStructOf<F: StorageFamily> {
    health: F::Storage<f32>,
}

impl<F: StorageFamily> Archetype for UnitStructOf<F> {
    type Item = Unit;
    type Family = F;

    fn insert(&mut self, value: Self::Item) -> ArchetypeId<Self> {
        self.health.insert(value.health)
    }

    fn remove(&mut self, id: ArchetypeId<Self>) -> Option<Self::Item> {
        let health = self.health.remove(id)?;
        Some(Unit { health })
    }
}

impl<F: StorageFamily> Default for UnitStructOf<F> {
    fn default() -> Self {
        Self {
            health: Default::default(),
        }
    }
}

impl SplitFields for Particle {
    type StructOf<F: StorageFamily> = ParticleStructOf<F>;
}

struct ParticleStructOf<F: StorageFamily> {
    time: F::Storage<f32>,
}

impl<F: StorageFamily> Archetype for ParticleStructOf<F> {
    type Item = Particle;
    type Family = F;

    fn insert(&mut self, value: Self::Item) -> ArchetypeId<Self> {
        self.time.insert(value.time)
    }

    fn remove(&mut self, id: ArchetypeId<Self>) -> Option<Self::Item> {
        let time = self.time.remove(id)?;
        Some(Particle { time })
    }
}

impl<F: StorageFamily> Default for ParticleStructOf<F> {
    fn default() -> Self {
        Self {
            time: Default::default(),
        }
    }
}
