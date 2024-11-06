use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{IdGenerator, Storage, StorageFamily},
};

use std::collections::{HashMap, HashSet};

/// Identifier type for a [`HashStorage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

/// A storage that keeps values inside a [`HashMap`].
#[derive(Clone)]
pub struct HashStorage<T>(HashMap<Id, T>);

impl<T> Default for HashStorage<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

unsafe impl<T> Storage<T> for HashStorage<T> {
    type Family = HashFamily;
    type Id = Id;
    // fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone {
    //     // SAFETY: `keys()` guarantees validity and uniqueness
    //     self.inner.keys().copied()
    // }
    fn insert(&mut self, id: Self::Id, value: T) {
        self.0.insert(id, value);
    }
    fn get(&self, id: Self::Id) -> Option<&T> {
        self.0.get(&id)
    }
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T> {
        self.0.get_mut(&id)
    }
    fn remove(&mut self, id: Self::Id) -> Option<T> {
        self.0.remove(&id)
    }
    #[cfg(feature = "query_mut")]
    unsafe fn get_many_unchecked_mut<'a>(
        &'a mut self,
        ids: impl Iterator<Item = Self::Id>,
    ) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        ids.map(move |i| {
            let r = self.get_mut(i).expect("invalid id: entry absent");
            unsafe { &mut *(r as *mut T) }
        })
    }
}

#[derive(Clone)]
pub struct HashIdGenerator {
    next_id: Id,
    alive: HashSet<Id>,
}

impl Default for HashIdGenerator {
    fn default() -> Self {
        Self {
            next_id: Id(1),
            alive: HashSet::new(),
        }
    }
}

impl IdGenerator for HashIdGenerator {
    type Id = Id;
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone {
        self.alive.iter().copied()
    }
    fn spawn(&mut self) -> Self::Id {
        let id = self.next_id;
        self.next_id.0 += 1;
        self.alive.insert(id);
        id
    }
    fn remove(&mut self, id: Self::Id) -> bool {
        self.alive.remove(&id)
    }
}

/// Family of [`HashStorage<T>`] storages.
pub struct HashFamily;

impl StorageFamily for HashFamily {
    type Id = Id;
    type Storage<T> = HashStorage<T>;
    type Generator = HashIdGenerator;
}

impl<T: SplitFields<HashFamily>> StructOfAble for HashStorage<T> {
    type Struct = T;
    type Family = HashFamily;
}
