use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{IdGenerator, Storage, StorageFamily},
};

use slotmap::{DenseSlotMap, SparseSecondaryMap};

slotmap::new_key_type! { pub struct SparseId; }

/// Wrapper for a [`SparseSecondaryMap`] storage.
///
/// Note: uses a [`DenseSlotMap`] to store id's for optimal id iteration (query) speed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sparse<T, K: slotmap::Key = SparseId>(SparseSecondaryMap<K, T>);

impl<K: slotmap::Key, T> Default for Sparse<T, K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: slotmap::Key, T> Storage<T> for Sparse<T, K> {
    type Family = SparseFamily<K>;
    type Id = K;
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
    fn iter<'a>(&'a self) -> impl Iterator<Item = (Self::Id, &'a T)>
    where
        T: 'a,
    {
        self.0.iter()
    }
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (Self::Id, &'a mut T)>
    where
        T: 'a,
    {
        self.0.iter_mut()
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
pub struct DenseIdGenerator<K: slotmap::Key> {
    alive: DenseSlotMap<K, ()>,
}

impl<K: slotmap::Key> Default for DenseIdGenerator<K> {
    fn default() -> Self {
        Self {
            alive: DenseSlotMap::with_key(),
        }
    }
}

unsafe impl<K: slotmap::Key> IdGenerator for DenseIdGenerator<K> {
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
pub struct SparseFamily<K: slotmap::Key>(std::marker::PhantomData<K>);

impl<K: slotmap::Key> StorageFamily for SparseFamily<K> {
    type Id = K;
    type Storage<T> = Sparse<T, K>;
    type Generator = DenseIdGenerator<K>;
}

impl<K: slotmap::Key, T: SplitFields<SparseFamily<K>>> StructOfAble for Sparse<T, K> {
    type Struct = T;
    type Family = SparseFamily<K>;
}
