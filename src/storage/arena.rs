use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{Storage, StorageFamily},
};

pub use slotmap::{self, DefaultKey as ArenaId, SlotMap};

/// Type alias for a [`SlotMap`] storage with a default key.
pub type Arena<T> = SlotMap<ArenaId, T>;

/// Family of [`SlotMap<K, V>`] storages.
pub struct SlotMapFamily<K: slotmap::Key>(std::marker::PhantomData<K>);

impl<K: slotmap::Key> StorageFamily for SlotMapFamily<K> {
    type Id = K;
    type Storage<T> = SlotMap<K, T>;
}

unsafe impl<K: slotmap::Key, T> Storage<T> for SlotMap<K, T> {
    type Family = SlotMapFamily<K>;
    type Id = K;
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone {
        // SAFETY: `keys()` guarantees validity and uniqueness
        self.keys()
    }
    fn insert(&mut self, value: T) -> Self::Id {
        self.insert(value)
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

impl<K: slotmap::Key, T: SplitFields<SlotMapFamily<K>>> StructOfAble for SlotMap<K, T> {
    type Struct = T;
    type Family = SlotMapFamily<K>;
}
