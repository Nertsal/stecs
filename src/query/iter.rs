use super::*;

pub struct QueryIter<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> {
    pub(crate) ids: F::IdIter,
    pub(crate) components: &'iter Q::Components<'comp>,
}

pub struct QueryIterMut<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> {
    pub(crate) ids: F::IdIter,
    pub(crate) components: &'iter mut Q::Components<'comp>,
}

impl<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> Iterator
    for QueryIter<'comp, 'iter, Q, F>
{
    type Item = (
        F::Id,
        <Q::Components<'comp> as QueryComponents<F>>::ItemReadOnly<'iter>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.ids.next()?;
        let item = self.components.get(id)?;
        Some((id, item))
    }
}

impl<'w, 'a, Q: StructQuery<F>, F: StorageFamily> IntoIterator for &'w Query<'a, Q, F> {
    type Item = <QueryIter<'a, 'w, Q, F> as Iterator>::Item;

    type IntoIter = QueryIter<'a, 'w, Q, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> QueryIterMut<'comp, 'iter, Q, F> {
    pub fn next(
        &mut self,
    ) -> Option<(
        F::Id,
        <Q::Components<'comp> as QueryComponents<F>>::Item<'_>,
    )> {
        let id = self.ids.next()?;
        let item = self.components.get_mut(id)?;
        Some((id, item))
    }
}
