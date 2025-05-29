use crate::storage::{Storage, StorageFamily};

use std::{any::Any, marker::PhantomData};

use anymap3::{AnyMap, CloneAny, Map};

pub struct DynamicArchetype<F: StorageFamily> {
    inner: AnyMap,
    id: PhantomData<F::Id>,
}

type InnerMap<F, T> = <F as StorageFamily>::Storage<T>;

/// A storage of dynamic components.
///
/// **NOTE**: you likely do not need to use it manually,
/// as it is used internally by the generated archetypes.
impl<F: StorageFamily> Default for DynamicArchetype<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: StorageFamily> DynamicArchetype<F> {
    pub fn new() -> Self {
        Self {
            inner: Map::new(),
            id: PhantomData,
        }
    }

    /// Insert a dynamic component into an entity, returing the old value if it was present.
    pub fn insert<T: Any>(&mut self, entity_id: F::Id, component: T) -> Option<T>
    where
        F::Storage<T>: 'static,
    {
        self.inner
            .entry::<InnerMap<F, T>>()
            .or_default()
            .insert(entity_id, component)
    }

    /// Remove a dynamic component from an entity.
    pub fn remove<T: Any>(&mut self, entity_id: F::Id) -> Option<T>
    where
        F::Storage<T>: 'static,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.remove(entity_id)
    }

    /// Get a reference to a dynamic component of an entity.
    pub fn get<T: Any>(&self, entity_id: F::Id) -> Option<&T>
    where
        F::Storage<T>: 'static,
    {
        self.inner.get::<InnerMap<F, T>>()?.get(entity_id)
    }

    /// Get a mutable reference a dynamic component from an entity.
    pub fn get_mut<T: Any>(&mut self, entity_id: F::Id) -> Option<&mut T>
    where
        F::Storage<T>: 'static,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.get_mut(entity_id)
    }

    pub fn get_many<'a, T: Any>(
        &'a self,
        ids: impl Iterator<Item = F::Id> + 'a,
    ) -> impl Iterator<Item = Option<&'a T>> + 'a
    where
        F::Storage<T>: 'static,
    {
        GetMany::<F, T, _> {
            map: self.inner.get::<InnerMap<F, T>>(),
            ids,
        }
    }

    /// # Safety
    /// The given `ids` must not repeat and must be valid and present id's in the storage.
    pub unsafe fn get_many_mut<T: Any>(
        &mut self,
        ids: impl Iterator<Item = F::Id>,
    ) -> impl Iterator<Item = Option<&mut T>>
    where
        F::Storage<T>: 'static,
    {
        let inner = self.inner.entry::<InnerMap<F, T>>().or_default();
        ids.map(move |id| inner.get_mut(id).map(|r| unsafe { &mut *(r as *mut T) }))
    }
}

struct GetMany<'a, F: StorageFamily, T, I: Iterator<Item = F::Id>> {
    map: Option<&'a InnerMap<F, T>>,
    ids: I,
}

impl<'a, F: StorageFamily, T: 'a, I: Iterator<Item = F::Id>> Iterator for GetMany<'a, F, T, I> {
    type Item = Option<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.ids.next()?;
        let map = self.map.as_ref()?;
        Some(map.get(id))
    }
}

/// A storage of dynamic `Clone`-able components.
///
/// **NOTE**: you likely do not need to use it manually,
/// as it is used internally by the generated archetypes.
pub struct DynamicCloneArchetype<F: StorageFamily> {
    inner: Map<dyn CloneAny>,
    id: PhantomData<F::Id>,
}

impl<F: StorageFamily> Default for DynamicCloneArchetype<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: StorageFamily> Clone for DynamicCloneArchetype<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: PhantomData,
        }
    }
}

impl<F: StorageFamily> DynamicCloneArchetype<F> {
    pub fn new() -> Self {
        Self {
            inner: Map::new(),
            id: PhantomData,
        }
    }

    /// Insert a dynamic component into an entity, returing the old value if it was present.
    pub fn insert<T: CloneAny>(&mut self, entity_id: F::Id, component: T) -> Option<T>
    where
        F::Storage<T>: 'static + Clone,
    {
        self.inner
            .entry::<InnerMap<F, T>>()
            .or_default()
            .insert(entity_id, component)
    }

    /// Remove a dynamic component from an entity.
    pub fn remove<T: CloneAny>(&mut self, entity_id: F::Id) -> Option<T>
    where
        F::Storage<T>: 'static + Clone,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.remove(entity_id)
    }

    /// Get a reference to a dynamic component of an entity.
    pub fn get<T: CloneAny>(&self, entity_id: F::Id) -> Option<&T>
    where
        F::Storage<T>: 'static + Clone,
    {
        self.inner.get::<InnerMap<F, T>>()?.get(entity_id)
    }

    /// Get a mutable reference a dynamic component from an entity.
    pub fn get_mut<T: CloneAny>(&mut self, entity_id: F::Id) -> Option<&mut T>
    where
        F::Storage<T>: 'static + Clone,
    {
        self.inner.get_mut::<InnerMap<F, T>>()?.get_mut(entity_id)
    }

    pub fn get_many<'a, T: CloneAny>(
        &'a self,
        ids: impl Iterator<Item = F::Id> + 'a,
    ) -> impl Iterator<Item = Option<&'a T>> + 'a
    where
        F::Storage<T>: 'static + Clone,
    {
        GetMany::<F, T, _> {
            map: self.inner.get::<InnerMap<F, T>>(),
            ids,
        }
    }

    /// # Safety
    /// The given `ids` must not repeat and must be valid and present id's in the storage.
    pub unsafe fn get_many_mut<T: CloneAny>(
        &mut self,
        ids: impl Iterator<Item = F::Id>,
    ) -> impl Iterator<Item = Option<&mut T>>
    where
        F::Storage<T>: 'static + Clone,
    {
        let inner = self.inner.entry::<InnerMap<F, T>>().or_default();
        ids.map(move |id| inner.get_mut(id).map(|r| unsafe { &mut *(r as *mut T) }))
    }
}
