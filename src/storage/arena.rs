use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{IdGenerator, Storage, StorageFamily},
};

pub use slotmap::{self, DefaultKey as ArenaId};
use slotmap::{SecondaryMap, SlotMap};

/// Type wrapper for a [`SlotMap`] storage with a default key.
pub struct Arena<T, K: slotmap::Key = ArenaId>(SecondaryMap<K, T>);

impl<K: slotmap::Key, T> Default for Arena<T, K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

unsafe impl<K: slotmap::Key, T> Storage<T> for Arena<T, K> {
    type Family = ArenaFamily<K>;
    type Id = K;
    // fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone {
    //     // SAFETY: `keys()` guarantees validity and uniqueness
    //     self.keys()
    // }
    fn insert(&mut self, id: Self::Id, value: T) {
        self.0.insert(id, value);
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
            &mut *(r as *mut T)
        })
    }
}

#[derive(Clone)]
pub struct ArenaIdGenerator<K: slotmap::Key> {
    alive: SlotMap<K, ()>,
}

impl<K: slotmap::Key> Default for ArenaIdGenerator<K> {
    fn default() -> Self {
        Self {
            alive: SlotMap::with_key(),
        }
    }
}

impl<K: slotmap::Key> IdGenerator for ArenaIdGenerator<K> {
    type Id = K;
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone {
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
pub struct ArenaFamily<K: slotmap::Key>(std::marker::PhantomData<K>);

impl<K: slotmap::Key> StorageFamily for ArenaFamily<K> {
    type Id = K;
    type Storage<T> = Arena<T, K>;
    type Generator = ArenaIdGenerator<K>;
}

impl<K: slotmap::Key, T: SplitFields<ArenaFamily<K>>> StructOfAble for Arena<T, K> {
    type Struct = T;
    type Family = ArenaFamily<K>;
}
