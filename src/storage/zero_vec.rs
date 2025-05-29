use crate::{
    archetype::{SplitFields, Splitable},
    storage::{IdGenerator, Storage, StorageFamily},
};

// Adapted from Zero ECS

/// Family of [`ZeroVec<T>`] storages.
pub struct ZeroVecFamily;

/// Identifier type for elements of [`ZeroVec<T>`] storage.
// NOTE: the id is not used to index into the storage directly,
// but as an auto-inc identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZeroVecId(usize);

impl StorageFamily for ZeroVecFamily {
    type Id = ZeroVecId;
    type Storage<T> = ZeroVec<T>;
    type IdGenerator = ZeroVecIdGenerator;
}

impl<T: SplitFields<ZeroVecFamily>> Splitable for ZeroVec<T> {
    type Struct = T;
    type Family = ZeroVecFamily;
}

/// Storage that acts as a wrapper over [`Vec`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZeroVec<T> {
    data: Vec<T>,
    ids: Vec<ZeroVecId>,
    index_lookup: Vec<Option<usize>>,
}

/// The generator of identifiers for the [`ZeroVec`] storage.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZeroVecIdGenerator {
    next_id: usize,
    ids: Vec<ZeroVecId>,
    index_lookup: Vec<Option<usize>>,
}

impl<T> Storage<T> for ZeroVec<T> {
    type Family = ZeroVecFamily;
    type Id = ZeroVecId;
    fn insert(&mut self, id: Self::Id, value: T) -> Option<T> {
        self.insert(id, value)
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
    unsafe fn get_unchecked(&self, id: Self::Id) -> &T {
        unsafe { self.get_unchecked(id) }
    }
    unsafe fn get_unchecked_mut(&mut self, id: Self::Id) -> &mut T {
        unsafe { self.get_unchecked_mut(id) }
    }
}

unsafe impl IdGenerator for ZeroVecIdGenerator {
    type Id = ZeroVecId;

    fn ids(&self) -> impl Iterator<Item = Self::Id> {
        self.ids.iter().copied()
    }

    fn spawn(&mut self) -> Self::Id {
        self.index_lookup.push(Some(self.ids.len()));
        let id = ZeroVecId(self.next_id);
        self.ids.push(id);
        self.next_id += 1;
        id
    }

    fn remove(&mut self, id: Self::Id) -> bool {
        let Some(&Some(old_index)) = self.index_lookup.get(id.0) else {
            return false;
        };
        self.index_lookup[id.0] = None;

        // NOTE: subtraction cannot underflow because at this point it is
        // guaranteed that at least one element is present (and is being deleted).
        let last_index = self.ids.len() - 1;

        if old_index != last_index {
            let last_id = self.ids[last_index];
            self.ids.swap(old_index, last_index);
            self.ids.pop();
            self.index_lookup[last_id.0] = Some(old_index);
        } else {
            self.ids.pop();
        }

        true
    }
}

impl ZeroVecIdGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> Default for ZeroVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ZeroVec<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            ids: Vec::new(),
            index_lookup: Vec::new(),
        }
    }

    /// Insert data to be associated with the identifier.
    ///
    /// Returns the old data if it was present.
    pub fn insert(&mut self, id: ZeroVecId, mut value: T) -> Option<T> {
        match self.index_lookup.get(id.0) {
            None => {
                // Create an entity slot and extend the lookup table
                self.index_lookup
                    .extend((id.0..self.index_lookup.len().saturating_sub(1)).map(|_| None));
                self.index_lookup.push(Some(self.data.len()));
                self.data.push(value);
                self.ids.push(id);
                None
            }
            Some(None) => {
                // Create an entity slot and update the lookup table
                self.index_lookup[id.0] = Some(self.data.len());
                self.data.push(value);
                self.ids.push(id);
                None
            }
            Some(&Some(index)) => {
                // Modify an existing slot
                let slot = self
                    .data
                    .get_mut(index)
                    .expect("data was allocated at this slot but the slot does not exist");
                std::mem::swap(&mut value, slot);
                Some(value)
            }
        }
    }

    /// Remove data associated with the given identifier.
    /// Returns the data if it was present.
    pub fn remove(&mut self, id: ZeroVecId) -> Option<T> {
        let Some(&Some(old_index)) = self.index_lookup.get(id.0) else {
            return None;
        };
        self.index_lookup[id.0] = None;

        // NOTE: subtraction cannot underflow because at this point it is
        // guaranteed that at least one element is present (and is being deleted).
        let last_index = self.data.len() - 1;

        let item = if old_index != last_index {
            let last_id = self.ids[last_index];
            self.index_lookup[last_id.0] = Some(old_index);
            self.ids.swap(old_index, last_index);
            self.ids.pop();
            self.data.swap(old_index, last_index);
            self.data
                .pop()
                .expect("desync between data and identifiers in ZeroVec")
        } else {
            self.ids.pop();
            self.data
                .pop()
                .expect("desync between data and identifiers in ZeroVec")
        };

        Some(item)
    }

    /// Returns a reference to an element associated with the given identifier.
    pub fn get(&self, id: ZeroVecId) -> Option<&T> {
        let &index = self.index_lookup.get(id.0)?.as_ref()?;
        self.data.get(index)
    }

    /// Returns a mutable reference to an element associated with the given identifier.
    pub fn get_mut(&mut self, id: ZeroVecId) -> Option<&mut T> {
        let &index = self.index_lookup.get(id.0)?.as_ref()?;
        self.data.get_mut(index)
    }

    /// Returns a reference to an element associated with the given identifier.
    ///
    /// # Safety
    /// `id` must be an identifier that was previosly used to insert data
    /// and has not been removed since.
    pub unsafe fn get_unchecked(&self, id: ZeroVecId) -> &T {
        let index = unsafe { self.index_lookup.get_unchecked(id.0) }.unwrap();
        unsafe { self.data.get_unchecked(index) }
    }

    /// Returns a mutable reference to an element associated with the given identifier.
    ///
    /// # Safety
    /// `id` must be an identifier that was previosly used to insert data
    /// and has not been removed since.
    pub unsafe fn get_unchecked_mut(&mut self, id: ZeroVecId) -> &mut T {
        let index = unsafe { self.index_lookup.get_unchecked(id.0) }.unwrap();
        unsafe { self.data.get_unchecked_mut(index) }
    }
}
