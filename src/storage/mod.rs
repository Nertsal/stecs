#[cfg(feature = "arena")]
pub mod arena;
#[cfg(feature = "hashstorage")]
pub mod hashstorage;
pub mod vec;

/// A single component storage.
pub trait Storage<T>: Default {
    type Family: StorageFamily;
    type Id: Copy;

    fn phantom_data(&self) -> std::marker::PhantomData<Self::Family> {
        Default::default()
    }
    fn ids(&self) -> impl Iterator<Item = Self::Id>;
    fn insert(&mut self, value: T) -> Self::Id;
    fn get(&self, id: Self::Id) -> Option<&T>;
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T>;
    fn remove(&mut self, id: Self::Id) -> Option<T>;

    /// Get mutable references to all id's in the iterator.
    ///
    /// # Safety
    /// `ids` given must not repeat and be valid and present id's in the storage.
    ///
    unsafe fn get_many_unchecked_mut<'a>(
        &'a mut self,
        ids: impl Iterator<Item = Self::Id>,
    ) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a;
}

/// A family of storages for different component types.
pub trait StorageFamily {
    type Id: Copy;
    type Storage<T>: Storage<T, Family = Self, Id = Self::Id>;
}
