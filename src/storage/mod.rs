mod vec;

pub use vec::*;

/// A single component storage.
pub trait Storage<T>: Default {
    type Family: StorageFamily;
    type Id: Copy;
    type IdIter: Iterator<Item = Self::Id>;

    fn phantom_data(&self) -> std::marker::PhantomData<Self::Family> {
        Default::default()
    }
    fn ids(&self) -> Self::IdIter;
    fn insert(&mut self, value: T) -> Self::Id;
    fn get(&self, id: Self::Id) -> Option<&T>;
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T>;
    fn remove(&mut self, id: Self::Id) -> Option<T>;
}

/// A family of storages for different component types.
pub trait StorageFamily {
    type Id: Copy;
    type IdIter: Iterator<Item = Self::Id>;
    type Storage<T>: Storage<T, Family = Self, Id = Self::Id, IdIter = Self::IdIter>;
}
