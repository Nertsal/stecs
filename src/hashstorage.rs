use crate::{SplitFields, Storage, StorageFamily, StructOfAble};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

#[derive(Clone)]
pub struct HashStorage<T> {
    next_id: Id,
    inner: HashMap<Id, T>,
}

impl<T> Default for HashStorage<T> {
    fn default() -> Self {
        Self {
            next_id: Id(0),
            inner: Default::default(),
        }
    }
}

impl<T> Storage<T> for HashStorage<T> {
    type Family = HashFamily;
    type Id = Id;
    type IdIter = std::vec::IntoIter<Id>;

    fn ids(&self) -> Self::IdIter {
        self.inner.keys().copied().collect::<Vec<_>>().into_iter()
    }

    fn insert(&mut self, value: T) -> Self::Id {
        let id = self.next_id;
        self.next_id.0 += 1;
        let res = self.inner.insert(id, value);
        assert!(
            res.is_none(),
            "Failed to generate a unique id in a collection"
        );
        id
    }

    fn get(&self, id: Self::Id) -> Option<&T> {
        self.inner.get(&id)
    }

    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T> {
        self.inner.get_mut(&id)
    }

    fn remove(&mut self, id: Self::Id) -> Option<T> {
        self.inner.remove(&id)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (Self::Id, &T)> + '_> {
        Box::new(self.inner.iter().map(|(&id, v)| (id, v)))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = (Self::Id, &mut T)> + '_> {
        Box::new(self.inner.iter_mut().map(|(&id, v)| (id, v)))
    }
}

pub struct HashFamily;

impl StorageFamily for HashFamily {
    type Id = Id;
    type IdIter = std::vec::IntoIter<Id>;
    type Storage<T> = HashStorage<T>;
}

impl<T: SplitFields<HashFamily>> StructOfAble for HashStorage<T> {
    type Struct = T;
    type Family = HashFamily;
}
