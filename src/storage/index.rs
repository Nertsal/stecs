use std::{collections::HashMap, hash::Hash};

use super::{Storage, StorageFamily};

pub struct Indexed<F: StorageFamily, T: Hash> {
    data: F::Storage<T>,
    index: HashMap<T, F::Id>,
}

pub struct RefMut<'a, T: Hash, Id> {
    data: &'a mut T,
    index: &'a mut HashMap<T, Id>,
}

impl<F: StorageFamily, T: Hash> Indexed<F, T> {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            index: HashMap::new(),
        }
    }

    pub fn get(&mut self, id: F::Id) -> &T {
        self.data.get(id).expect("Invalid Id index")
    }

    pub fn get_mut(&mut self, id: F::Id) -> RefMut<'_, T, F::Id> {
        RefMut {
            data: self.data.get_mut(id).expect("Invalid Id index"),
            index: &mut self.index,
        }
    }

    pub fn insert(&mut self) {
        todo!()
    }
}

impl<F: StorageFamily, T: Hash> Default for Indexed<F, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash, Id> Drop for RefMut<'_, T, Id> {
    fn drop(&mut self) {
        todo!()
    }
}
