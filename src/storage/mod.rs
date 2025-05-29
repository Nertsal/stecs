/// SlotMap storage.
#[cfg(feature = "slotmap")]
pub mod slotmap;
#[cfg(feature = "zero_vec")]
pub mod zero_vec;

/// A storage of components.
pub trait Storage<T>: Default {
    /// Type of the abstract family corresponding to the storages of this type.
    type Family: StorageFamily;
    /// Type of the identifier used for components/entities.
    type Id: Copy;

    /// Insert a new component at the given id.
    /// Returns the old component if it was present at that location.
    fn insert(&mut self, id: Self::Id, value: T) -> Option<T>;
    /// Get an immutable reference to a component a given id.
    fn get(&self, id: Self::Id) -> Option<&T>;
    /// Get a mutable reference to a component a given id.
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T>;
    /// Remove an component with a given id.
    fn remove(&mut self, id: Self::Id) -> Option<T>;
    /// Get an immutable reference to a component at the given id.
    /// # Safety
    /// The given `id` must be present in the storage.
    unsafe fn get_unchecked(&self, id: Self::Id) -> &T;
    /// Get an mutable reference to a component at the given id.
    /// # Safety
    /// The given `id` must be present in the storage.
    unsafe fn get_unchecked_mut(&mut self, id: Self::Id) -> &mut T;
}

/// A storage of sparse (optional) components.
pub trait SparseStorage<T>: Default {
    /// Type of the abstract family corresponding to the storages of this type.
    type Family: StorageFamily;
    /// Type of the identifier used for components/entities.
    type Id: Copy;

    /// Insert a new component at the given id.
    /// Returns the old component if it was present at that location.
    fn insert(&mut self, id: Self::Id, value: T) -> Option<T>;
    /// Get an immutable reference to a component a given id.
    fn get(&self, id: Self::Id) -> Option<&T>;
    /// Get a mutable reference to a component a given id.
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T>;
    /// Remove an component with a given id.
    fn remove(&mut self, id: Self::Id) -> Option<T>;
    /// Get an immutable reference to a component at the given id.
    /// # Safety
    /// The given `id` must be present in the storage.
    unsafe fn get_unchecked(&self, id: Self::Id) -> Option<&T>;
    /// Get an mutable reference to a component at the given id.
    /// # Safety
    /// The given `id` must be present in the storage.
    unsafe fn get_unchecked_mut(&mut self, id: Self::Id) -> Option<&mut T>;
}

/// A generator of identifiers to use with [`Storage`].
///
/// # Safety
/// The [`IdGenerator::ids`] method must return an iterator of unique and valid id's.
/// That is, they must not repeat, and must correspond to valid entities when
/// used in [`Storage::get`] or [`Storage::get_mut`] (unless removed).
///
pub unsafe trait IdGenerator: Default {
    /// The identifier type being generated.
    type Id: Copy;

    /// Returns the unique id's of all active entities in the storage in an arbitrary order.
    ///
    /// **Note**: [`Clone`](trait@std::clone::Clone) is constrained for sharing between multiple fields' accessors when implementing [`get_many_unchecked_mut`](Storage::get_many_unchecked_mut).
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone;
    /// Generate a new available id.
    fn spawn(&mut self) -> Self::Id;
    /// Remove/free an id, returns `true` if the id was present.
    fn remove(&mut self, id: Self::Id) -> bool;
}

/// A family of storages for different component types.
pub trait StorageFamily {
    /// Type of the identifier used for components/entities.
    type Id: Copy;
    type IdGenerator: IdGenerator<Id = Self::Id>;
    /// Type of a specific storage.
    type Storage<T>: Storage<T, Family = Self, Id = Self::Id>;
    /// Type of a specific sparse storage used for optional components.
    type SparseStorage<T>: SparseStorage<T, Family = Self, Id = Self::Id>;
}
