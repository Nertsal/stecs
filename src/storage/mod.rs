mod vec;

pub use vec::*;

/// A single component storage.
pub trait Storage<T>: Default {
    type Id: Copy;
    type IdIter: Iterator<Item = Self::Id>;
    type Iterator<'a>: Iterator<Item = &'a T> + 'a
    where
        Self: 'a,
        T: 'a;
    type IteratorMut<'a>: Iterator<Item = &'a mut T> + 'a
    where
        Self: 'a,
        T: 'a;

    fn ids(&self) -> Self::IdIter;
    fn insert(&mut self, value: T) -> Self::Id;
    fn get(&self, id: Self::Id) -> Option<&T>;
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T>;
    fn remove(&mut self, id: Self::Id) -> Option<T>;
    fn iter(&self) -> Self::Iterator<'_>;
    fn iter_mut(&mut self) -> Self::IteratorMut<'_>;
}

/// A family of storages for different component types.
pub trait StorageFamily {
    type Id: Copy;
    type IdIter: Iterator<Item = Self::Id>;
    type Storage<T>: Storage<T, Id = Self::Id, IdIter = Self::IdIter>;
}
