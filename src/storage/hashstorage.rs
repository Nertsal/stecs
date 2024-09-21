use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{Storage, StorageFamily},
};

use std::collections::HashMap;

/// Identifier type for a [`HashStorage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

/// A storage that keeps values inside a [`HashMap`].
#[derive(Clone)]
pub struct HashStorage<T> {
    next_id: Id,
    inner: HashMap<Id, T>,
}

impl<T> Default for HashStorage<T> {
    fn default() -> Self {
        Self {
            next_id: Id(0),
            inner: Default::default(),
        }
    }
}

unsafe impl<T> Storage<T> for HashStorage<T> {
    type Family = HashFamily;
    type Id = Id;
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone {
        // SAFETY: `keys()` guarantees validity and uniqueness
        self.inner.keys().copied()
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

/// Family of [`HashStorage<T>`] storages.
pub struct HashFamily;

impl StorageFamily for HashFamily {
    type Id = Id;
    type Storage<T> = HashStorage<T>;
}

impl<T: SplitFields<HashFamily>> StructOfAble for HashStorage<T> {
    type Struct = T;
    type Family = HashFamily;
}
