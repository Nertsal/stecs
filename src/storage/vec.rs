use crate::archetype::{SplitFields, StructOfAble};

use super::*;

use std::collections::BTreeSet;

impl<T> Storage<T> for Vec<T> {
    type Family = VecFamily;
    type Id = usize;
    fn insert(&mut self, value: T) -> Self::Id {
        let id = self.len();
        self.push(value);
        id
    }
    fn ids(&self) -> BTreeSet<Self::Id> {
        (0..self.len()).collect()
    }
    fn get(&self, id: Self::Id) -> Option<&T> {
        self.as_slice().get(id)
    }
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T> {
        self.as_mut_slice().get_mut(id)
    }
    fn remove(&mut self, id: Self::Id) -> Option<T> {
        (id < self.len()).then(|| self.swap_remove(id))
    }
}

pub struct VecFamily;

impl StorageFamily for VecFamily {
    type Id = usize;
    type Storage<T> = Vec<T>;
}

impl<T: SplitFields<VecFamily>> StructOfAble for Vec<T> {
    type Struct = T;
    type Family = VecFamily;
}
