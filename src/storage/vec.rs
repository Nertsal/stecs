use crate::archetype::{SplitFields, StructOfAble};

use super::*;

impl<T> Storage<T> for Vec<T> {
    type Family = VecFamily;
    type Id = usize;
    fn insert(&mut self, value: T) -> Self::Id {
        let id = self.len();
        self.push(value);
        id
    }
    fn ids(&self) -> impl Iterator<Item = Self::Id> {
        0..self.len()
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
    fn get_many_mut<'a>(
        &'a mut self,
        ids: impl Iterator<Item = Self::Id>,
    ) -> impl Iterator<Item = Option<&'a mut T>>
    where
        T: 'a,
    {
        let mut collected = Vec::new(); // TODO: remove allocation
        ids.map(move |i| {
            if collected.contains(&i) {
                return None;
            }
            // SAFETY: `collected` checks that no indices are repeated.
            self.get_mut(i).map(|r| {
                collected.push(i);
                unsafe { &mut *(r as *mut T) }
            })
        })
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
