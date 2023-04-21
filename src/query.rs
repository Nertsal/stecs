use crate::{archetype::SplitFields, prelude::Archetype, storage::StorageFamily};

pub trait StructQuery<F: StorageFamily> {
    type Base: SplitFields<F>;
    type Item<'a>;

    fn get(
        struct_of: &<Self::Base as SplitFields<F>>::StructOf,
        id: F::Id,
    ) -> Option<Self::Item<'_>>;

    fn query(struct_of: &<Self::Base as SplitFields<F>>::StructOf) -> QueryIter<'_, Self, F>
    where
        Self: Sized,
    {
        QueryIter {
            ids: struct_of.ids(),
            struct_of,
        }
    }
}

pub struct QueryIter<'a, Q: StructQuery<F>, F: StorageFamily> {
    ids: F::IdIter,
    struct_of: &'a <Q::Base as SplitFields<F>>::StructOf,
}

// -- Query impl --

impl<'a, Q: StructQuery<F>, F: StorageFamily> Iterator for QueryIter<'a, Q, F> {
    type Item = Q::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.ids.next()?;
        Q::get(self.struct_of, id)
    }
}
