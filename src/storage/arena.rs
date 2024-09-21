use crate::{
    archetype::{SplitFields, StructOfAble},
    storage::{Storage, StorageFamily},
};

pub use generational_arena::{Arena, Index};

/// Family of Arena<T> storages.
pub struct ArenaFamily;

impl StorageFamily for ArenaFamily {
    type Id = Index;
    type Storage<T> = Arena<T>;
}

unsafe impl<T> Storage<T> for Arena<T> {
    type Family = ArenaFamily;
    type Id = Index;
    fn ids(&self) -> impl Iterator<Item = Self::Id> + Clone {
        // SAFETY: `iter()` guarantees validity and uniqueness
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
    unsafe fn get_many_unchecked_mut<'a>(
        &'a mut self,
        ids: impl Iterator<Item = Self::Id>,
    ) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        ids.map(move |i| {
            let r = self.get_mut(i).expect("invalid id: entry absent");
            &mut *(r as *mut T)
        })
    }
}

impl<T: SplitFields<ArenaFamily>> StructOfAble for Arena<T> {
    type Struct = T;
    type Family = ArenaFamily;
}
