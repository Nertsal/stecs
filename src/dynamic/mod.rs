use crate::storage::{Storage, StorageFamily};

use std::marker::PhantomData;

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

pub struct DynamicStorage<F> {
    inner: Map<dyn CloneAny>,
    id: PhantomData<F>,
}

type InnerMap<F, T> = <F as StorageFamily>::Storage<T>;

impl<F> Clone for DynamicStorage<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: PhantomData,
        }
    }
}

impl<F> Default for DynamicStorage<F> {
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

// TODO: docs
impl<F: StorageFamily> DynamicStorage<F> {
    pub fn insert<T>(&mut self, entity_id: F::Id, component: T)
    where
        InnerMap<F, T>: CloneAny,
    {
        self.inner
            .entry::<InnerMap<F, T>>()
            .or_default()
            .insert(entity_id, component)
    }

    pub fn remove<T>(&mut self, entity_id: F::Id) -> Option<T>
    where
        InnerMap<F, T>: CloneAny,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.remove(entity_id)
    }

    pub fn get<T>(&self, entity_id: F::Id) -> Option<&T>
    where
        InnerMap<F, T>: CloneAny,
    {
        self.inner.get::<InnerMap<F, T>>()?.get(entity_id)
    }

    pub fn get_mut<T>(&mut self, entity_id: F::Id) -> Option<&mut T>
    where
        InnerMap<F, T>: CloneAny,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.get_mut(entity_id)
    }

    /// Get mutable references to all id's in the iterator.
    ///
    /// # Safety
    /// The given `ids` must not repeat and must be valid and present id's in the storage.
    ///
    pub unsafe fn get_many_mut<'a, T: 'a>(
        &'a mut self,
        ids: impl Iterator<Item = F::Id>,
    ) -> impl Iterator<Item = Option<&'a mut T>>
    where
        InnerMap<F, T>: CloneAny,
    {
        let inner = self.inner.entry::<InnerMap<F, T>>().or_default();
        ids.map(move |id| inner.get_mut(id).map(|r| unsafe { &mut *(r as *mut T) }))
    }
}
