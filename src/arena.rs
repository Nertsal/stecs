use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{Storage, StorageFamily},
};

pub use generational_arena::{Arena, Index};

pub struct ArenaFamily;

impl StorageFamily for ArenaFamily {
    type Id = Index;
    type IdIter = std::vec::IntoIter<Self::Id>;
    type Storage<T> = Arena<T>;
}

impl<T> Storage<T> for Arena<T> {
    type Family = ArenaFamily;
    type Id = Index;
    type IdIter = std::vec::IntoIter<Self::Id>;
    fn ids(&self) -> Self::IdIter {
        self.iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>()
            .into_iter()
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
