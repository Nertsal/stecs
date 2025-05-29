use crate::{
    archetype::{SplitFields, Splitable},
    storage::{IdGenerator, SparseStorage, Storage, StorageFamily},
};

use slotmap::SecondaryMap;
pub use slotmap::{self, SlotMap};

impl<K: slotmap::Key, T> Storage<T> for SecondaryMap<K, T> {
    type Family = SlotMapFamily<K>;
    type Id = K;
    fn insert(&mut self, id: Self::Id, value: T) -> Option<T> {
        self.insert(id, value)
    }
    fn get(&self, id: Self::Id) -> Option<&T> {
        self.get(id)
    }
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T> {
        self.get_mut(id)
    }
    fn remove(&mut self, id: Self::Id) -> Option<T> {
        self.remove(id)
    }
    unsafe fn get_unchecked(&self, id: Self::Id) -> &T {
        unsafe { self.get_unchecked(id) }
    }
    unsafe fn get_unchecked_mut(&mut self, id: Self::Id) -> &mut T {
        unsafe { self.get_unchecked_mut(id) }
    }
}

#[derive(Debug, Clone)]
pub struct SparseSecondaryMap<K: slotmap::Key, T>(SecondaryMap<K, T>);

impl<K: slotmap::Key, T> Default for SparseSecondaryMap<K, T> {
    fn default() -> Self {
        Self(SecondaryMap::default())
    }
}

impl<K: slotmap::Key, T> SparseStorage<T> for SparseSecondaryMap<K, T> {
    type Family = SlotMapFamily<K>;
    type Id = K;
    fn insert(&mut self, id: Self::Id, value: T) -> Option<T> {
        self.0.insert(id, value)
    }
    fn get(&self, id: Self::Id) -> Option<&T> {
        self.0.get(id)
    }
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T> {
        self.0.get_mut(id)
    }
    fn remove(&mut self, id: Self::Id) -> Option<T> {
        self.0.remove(id)
    }
    unsafe fn get_unchecked(&self, id: Self::Id) -> Option<&T> {
        self.0.get(id)
    }
    unsafe fn get_unchecked_mut(&mut self, id: Self::Id) -> Option<&mut T> {
        self.0.get_mut(id)
    }
}

#[derive(Clone)]
pub struct SlotMapIdGenerator<K: slotmap::Key> {
    alive: SlotMap<K, ()>,
}

impl<K: slotmap::Key> Default for SlotMapIdGenerator<K> {
    fn default() -> Self {
        Self {
            alive: SlotMap::with_key(),
        }
    }
}

unsafe impl<K: slotmap::Key> IdGenerator for SlotMapIdGenerator<K> {
    type Id = K;
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone {
        // SAFETY: `keys()` guarantees uniqueness and partially validity;
        // proper validity is dependent on the derived implementation of Archetype::insert
        // passing the generated id's to the storages below.
        self.alive.keys()
    }
    fn spawn(&mut self) -> Self::Id {
        self.alive.insert(())
    }
    fn remove(&mut self, id: Self::Id) -> bool {
        self.alive.remove(id).is_some()
    }
}

/// Family of [`SlotMap<K, V>`] storages.
pub struct SlotMapFamily<K: slotmap::Key>(std::marker::PhantomData<K>);

impl<K: slotmap::Key> StorageFamily for SlotMapFamily<K> {
    type Id = K;
    type IdGenerator = SlotMapIdGenerator<K>;
    type Storage<T> = SecondaryMap<K, T>;
    type SparseStorage<T> = SparseSecondaryMap<K, T>;
}

impl<K: slotmap::Key, T: SplitFields<SlotMapFamily<K>>> Splitable for SlotMap<K, T> {
    type Struct = T;
    type Family = SlotMapFamily<K>;
}
