use super::*;

pub struct QueryIter<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> {
    pub(crate) ids: F::IdIter,
    pub(crate) components: &'iter Q::Components<'comp>,
}

impl<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> Iterator
    for QueryIter<'comp, 'iter, Q, F>
{
    type Item = <Q::Components<'comp> as QueryComponents<F>>::Item<'iter>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.ids.next()?;
        self.components.get(id)
    }
}

impl<'w, 'a, Q: StructQuery<F>, F: StorageFamily> IntoIterator for &'w Query<'a, Q, F> {
    type Item = <QueryIter<'a, 'w, Q, F> as Iterator>::Item;

    type IntoIter = QueryIter<'a, 'w, Q, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
