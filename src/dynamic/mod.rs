use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use anymap3::{CloneAny, Map};

// /// A trait for a dynamic component: data that can be attached to entities arbitrarily at runtime.
// pub trait DynamicComponent {
//     type Storage: DynamicStorage;
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum DynamicStorageType {
//     SparseSet,
// }

// TODO: rewrite anymap3
// TODO: optional clone
// TODO: different component storage types

#[derive(Clone)]
pub struct DynamicStorage<Id> {
    inner: Map<dyn CloneAny>,
    id: PhantomData<Id>,
}

type InnerMap<Id, T> = HashMap<Id, T>;

impl<Id> Default for DynamicStorage<Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id> DynamicStorage<Id> {
    pub fn new() -> Self {
        Self {
            inner: Map::new(),
            id: PhantomData,
        }
    }
}

impl<Id: 'static + Clone + Hash + Eq> DynamicStorage<Id> {
    pub fn insert<T: CloneAny + Clone>(&mut self, entity_id: Id, component: T) -> Option<T> {
        self.inner
            .entry::<InnerMap<Id, T>>()
            .or_default()
            .insert(entity_id, component)
    }

    pub fn remove<T: CloneAny + Clone>(&mut self, entity_id: Id) -> Option<T> {
        self.inner.get_mut::<InnerMap<Id, T>>()?.remove(&entity_id)
    }

    pub fn get<T: CloneAny + Clone>(&self, entity_id: Id) -> Option<&T> {
        self.inner.get::<InnerMap<Id, T>>()?.get(&entity_id)
    }

    pub fn get_mut<T: CloneAny + Clone>(&mut self, entity_id: Id) -> Option<&mut T> {
        self.inner.get_mut::<InnerMap<Id, T>>()?.get_mut(&entity_id)
    }

    /// Get mutable references to all id's in the iterator.
    ///
    /// # Safety
    /// The given `ids` must not repeat and must be valid and present id's in the storage.
    ///
    pub unsafe fn get_many_mut<T: CloneAny + Clone>(
        &mut self,
        ids: impl Iterator<Item = Id>,
    ) -> impl Iterator<Item = Option<&mut T>> {
        let inner = self.inner.entry::<InnerMap<Id, T>>().or_default();
        ids.map(move |id| inner.get_mut(&id).map(|r| unsafe { &mut *(r as *mut T) }))
    }
}
