/// Arena storage.
#[cfg(feature = "arena")]
pub mod arena;
/// Hash storage.
#[cfg(feature = "hashstorage")]
pub mod hashstorage;
/// Vec storage.
pub mod vec;

/// A storage of components.
///
/// # Safety
/// The [Storage::ids] method must return an iterator of unique and valid id's.
/// That is, they must not repeat, and must correspond to valid entities when
/// used in [Storage::get] or [Storage::get_mut] (unless removed).
///
pub unsafe trait Storage<T>: Default {
    /// Type of the abstract family corresponding to the storages of this type.
    type Family: StorageFamily;
    /// Type of the identifier used for components/entities.
    type Id: Copy;

    fn phantom_data(&self) -> std::marker::PhantomData<Self::Family> {
        Default::default()
    }
    /// Returns the unique id's of all active entities in the storage in an arbitrary order.
    ///
    /// **Note**: [`Clone`](trait@std::clone::Clone) is constrained for sharing between multiple fields' accessors when implementing [`get_many_unchecked_mut`](Storage::get_many_unchecked_mut).
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone;
    /// Insert a new component, returning its id.
    fn insert(&mut self, value: T) -> Self::Id;
    /// Get an immutable reference to a component a given id.
    fn get(&self, id: Self::Id) -> Option<&T>;
    /// Get a mutable reference to a component a given id.
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T>;
    /// Remove an component with a given id.
    fn remove(&mut self, id: Self::Id) -> Option<T>;

    /// Get mutable references to all id's in the iterator.
    ///
    /// # Safety
    /// The given `ids` must not repeat and must be valid and present id's in the storage.
    ///
    #[cfg(feature = "query_mut")]
    unsafe fn get_many_unchecked_mut<'a>(
        &'a mut self,
        ids: impl Iterator<Item = Self::Id>,
    ) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a;
}

/// A family of storages for different component types.
pub trait StorageFamily {
    /// Type of the identifier used for components/entities.
    type Id: Copy;
    /// Type of a specific storage.
    type Storage<T>: Storage<T, Family = Self, Id = Self::Id>;
}
