use super::*;

/// Iterator over the entities in an [Archetype].
pub struct ArchetypeIntoIter<F: StorageFamily, S: Archetype<F>> {
    ids: std::vec::IntoIter<F::Id>,
    archetype: S,
}

impl<F: StorageFamily, S: Archetype<F>> ArchetypeIntoIter<F, S> {
    /// Construct an iterator, consuming the archetype.
    pub fn new(archetype: S) -> Self {
        Self {
            ids: archetype.ids().collect::<Vec<_>>().into_iter(), // TODO: without collecting
            archetype,
        }
    }
}

impl<F: StorageFamily, S: Archetype<F>> Iterator for ArchetypeIntoIter<F, S> {
    type Item = (F::Id, S::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.ids.next()?;
        let item = self.archetype.remove(id)?;
        Some((id, item))
    }
}
