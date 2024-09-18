use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{Storage, StorageFamily},
};

pub use generational_arena::{Arena, Index};

pub struct ArenaFamily;

impl StorageFamily for ArenaFamily {
    type Id = Index;
    type Storage<T> = Arena<T>;
}

impl<T> Storage<T> for Arena<T> {
    type Family = ArenaFamily;
    type Id = Index;
    fn ids(&self) -> impl Iterator<Item = Self::Id> {
        self.iter().map(|(id, _)| id)
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
            // SAFETY: `collected` checks that no Index's are repeated.
            self.get_mut(i).map(|r| {
                collected.push(i);
                unsafe { &mut *(r as *mut T) }
            })
        })
    }
}

impl<T: SplitFields<ArenaFamily>> StructOfAble for Arena<T> {
    type Struct = T;
    type Family = ArenaFamily;
}
