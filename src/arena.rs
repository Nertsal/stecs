use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{Storage, StorageFamily},
};

pub use generational_arena::{Arena, Index};

use std::collections::HashSet;

pub struct ArenaFamily;

impl StorageFamily for ArenaFamily {
    type Id = Index;
    type Storage<T> = Arena<T>;
}

impl<T> Storage<T> for Arena<T> {
    type Family = ArenaFamily;
    type Id = Index;
    fn ids(&self) -> HashSet<Self::Id> {
        self.iter().map(|(id, _)| id).collect()
    }
    fn insert(&mut self, value: T) -> Self::Id {
        self.insert(value)
    }
    fn get(&self, id: Self::Id) -> Option<&T> {
        self.get(id)
    }
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T> {
        self.get_mut(id)
    }
    fn remove(&mut self, id: Self::Id) -> Option<T> {
        self.remove(id)
    }
}

impl<T: SplitFields<ArenaFamily>> StructOfAble for Arena<T> {
    type Struct = T;
    type Family = ArenaFamily;
}
