use super::*;

pub struct QueryIter<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> {
    pub(crate) ids: F::IdIter,
    pub(crate) components: &'iter mut Q::Components<'comp>,
}

impl<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> QueryIter<'comp, 'iter, Q, F> {
    pub fn next(&mut self) -> Option<<Q::Components<'comp> as QueryComponents<F>>::Item<'_>> {
        let id = self.ids.next()?;
        self.components.get(id)
    }
}

// impl<'comp: 'iter, 'iter, Q: StructQuery<F>, F: StorageFamily> Iterator
//     for QueryIter<'comp, 'iter, Q, F>
// {
//     type Item = <Q::Components<'comp> as QueryComponents<F>>::Item<'iter>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let id = self.ids.next()?;
//         self.components.get(id)
//     }
// }

// impl<'w, 'a, Q: StructQuery<F>, F: StorageFamily> IntoIterator for &'w mut Query<'a, Q, F> {
//     type Item = <QueryIter<'a, 'w, Q, F> as Iterator>::Item;

//     type IntoIter = QueryIter<'a, 'w, Q, F>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }
