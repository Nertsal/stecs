pub mod dense;
pub mod sparse;

/// A storage of components.
pub trait Storage<T>: Default {
    /// Type of the abstract family corresponding to the storages of this type.
    type Family: StorageFamily;
    /// Type of the identifier used for components/entities.
    type Id: slotmap::Key;

    /// Insert a new component to at the specified id.
    fn insert(&mut self, id: Self::Id, value: T);
    /// Get an immutable reference to a component at the given id.
    fn get(&self, id: Self::Id) -> Option<&T>;
    /// Get a mutable reference to a component at the given id.
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T>;
    /// Remove a component at the given id.
    fn remove(&mut self, id: Self::Id) -> Option<T>;
    /// Iterate over all components immutably.
    fn iter<'a>(&'a self) -> impl Iterator<Item = (Self::Id, &'a T)>
    where
        T: 'a;
    /// Iterate over all components mutably.
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (Self::Id, &'a mut T)>
    where
        T: 'a;

    /// Get mutable references to the components corresponding to the id's in the iterator.
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
    type Id: slotmap::Key;
    /// Type of a specific storage.
    type Storage<T>: Storage<T, Family = Self, Id = Self::Id>;
    type Generator: IdGenerator<Id = Self::Id>;
}

/// A generator of identifiers to use with [Storage]s.
///
/// # Safety
/// The [`IdGenerator::ids`] method must return an iterator of unique and valid id's.
/// That is, they must not repeat, and must correspond to valid entities when
/// used in [`Storage::get`] or [`Storage::get_mut`] (unless removed).
///
pub unsafe trait IdGenerator {
    /// The identifier type being generated.
    type Id: slotmap::Key;

    /// Returns the unique id's of all active entities in the storage in an arbitrary order.
    ///
    /// **Note**: [`Clone`](trait@std::clone::Clone) is constrained for sharing between multiple fields' accessors when implementing [`get_many_unchecked_mut`](Storage::get_many_unchecked_mut).
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone;
    /// Generate a new available id.
    fn spawn(&mut self) -> Self::Id;
    /// Remove/free an id, returns `true` if the id was present.
    fn remove(&mut self, id: Self::Id) -> bool;
}
