use crate::storage::{sparse::Sparse, Storage, StorageFamily};

use std::{any::Any, marker::PhantomData};

use anymap3::{CloneAny, Map};

type InnerMap<F, T> = Sparse<T, <F as StorageFamily>::Id>;

/// A storage of dynamic components.
///
/// **NOTE**: you likely do not need to use it manually,
/// as it is used internally by the generated archetypes.
pub struct DynamicStorage<F> {
    inner: Map<dyn Any>,
    id: PhantomData<F>,
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

impl<F: StorageFamily> DynamicStorage<F> {
    /// Insert a dynamic component into an entity.
    pub fn insert<T>(&mut self, entity_id: F::Id, component: T)
    where
        InnerMap<F, T>: Any,
    {
        self.inner
            .entry::<InnerMap<F, T>>()
            .or_default()
            .insert(entity_id, component)
    }

    /// Remove a dynamic component from an entity.
    pub fn remove<T>(&mut self, entity_id: F::Id) -> Option<T>
    where
        InnerMap<F, T>: Any,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.remove(entity_id)
    }

    /// Get a reference to a dynamic component from an entity.
    pub fn get<T>(&self, entity_id: F::Id) -> Option<&T>
    where
        InnerMap<F, T>: Any,
    {
        self.inner.get::<InnerMap<F, T>>()?.get(entity_id)
    }

    /// Get a mutable reference a dynamic component from an entity.
    pub fn get_mut<T>(&mut self, entity_id: F::Id) -> Option<&mut T>
    where
        InnerMap<F, T>: Any,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.get_mut(entity_id)
    }

    /// Get mutable references to components corresponding to the id's in the iterator.
    ///
    /// # Safety
    /// The given `ids` must not repeat.
    ///
    pub unsafe fn get_many_mut<'a, T: 'a>(
        &'a mut self,
        ids: impl Iterator<Item = F::Id>,
    ) -> impl Iterator<Item = Option<&'a mut T>>
    where
        InnerMap<F, T>: Any,
    {
        let inner = self.inner.entry::<InnerMap<F, T>>().or_default();
        ids.map(move |id| inner.get_mut(id).map(|r| unsafe { &mut *(r as *mut T) }))
    }
}

/// A storage of dynamic Clone-able components.
///
/// **NOTE**: you likely do not need to use it manually,
/// as it is used internally by the generated archetypes.
pub struct DynamicCloneStorage<F> {
    inner: Map<dyn CloneAny>,
    id: PhantomData<F>,
}

impl<F> Clone for DynamicCloneStorage<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: PhantomData,
        }
    }
}

impl<F> Default for DynamicCloneStorage<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id> DynamicCloneStorage<Id> {
    pub fn new() -> Self {
        Self {
            inner: Map::new(),
            id: PhantomData,
        }
    }
}

impl<F: StorageFamily> DynamicCloneStorage<F> {
    /// Insert a dynamic component into an entity.
    pub fn insert<T>(&mut self, entity_id: F::Id, component: T)
    where
        InnerMap<F, T>: CloneAny,
    {
        self.inner
            .entry::<InnerMap<F, T>>()
            .or_default()
            .insert(entity_id, component)
    }

    /// Remove a dynamic component from an entity.
    pub fn remove<T>(&mut self, entity_id: F::Id) -> Option<T>
    where
        InnerMap<F, T>: CloneAny,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.remove(entity_id)
    }

    /// Get a reference to a dynamic component from an entity.
    pub fn get<T>(&self, entity_id: F::Id) -> Option<&T>
    where
        InnerMap<F, T>: CloneAny,
    {
        self.inner.get::<InnerMap<F, T>>()?.get(entity_id)
    }

    /// Get a mutable reference a dynamic component from an entity.
    pub fn get_mut<T>(&mut self, entity_id: F::Id) -> Option<&mut T>
    where
        InnerMap<F, T>: CloneAny,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.get_mut(entity_id)
    }

    /// Get mutable references to components corresponding to the id's in the iterator.
    ///
    /// # Safety
    /// The given `ids` must not repeat.
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
