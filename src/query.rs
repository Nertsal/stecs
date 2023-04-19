use crate::archetype::{SplitFields, StructOf, StructOfAble};

pub trait StructQuery {
    type Item<'a>;
    fn query<Q: Queryable<Self>>(queryable: &Q) -> QueryIter<'_, Self, Q, Q::IdIter>
    where
        Self: Sized,
    {
        queryable.query()
    }
}

pub trait IdHolder {
    type Id;
    type IdIter: Iterator<Item = Self::Id>;
    fn ids(&self) -> Self::IdIter;
}

pub trait Queryable<Q: StructQuery>: IdHolder {
    fn get(&self, id: Self::Id) -> Option<Q::Item<'_>>;
    fn query(&self) -> QueryIter<'_, Q, Self, Self::IdIter>
    where
        Self: Sized,
    {
        QueryIter {
            ids: self.ids(),
            queryable: self,
            phantom_data: std::marker::PhantomData::default(),
        }
    }
}

pub struct QueryIter<'a, S: StructQuery, Q: Queryable<S>, I: Iterator<Item = Q::Id>> {
    ids: I,
    queryable: &'a Q,
    phantom_data: std::marker::PhantomData<S>,
}

// -- Query impl --

impl<'a, S: StructQuery, Q: Queryable<S>, I: Iterator<Item = Q::Id>> Iterator
    for QueryIter<'a, S, Q, I>
{
    type Item = S::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.ids.next()?;
        self.queryable.get(id)
    }
}

impl<S: StructOfAble> IdHolder for StructOf<S> {
    type Id = <<S::Struct as SplitFields>::StructOf<S::Family> as IdHolder>::Id;

    type IdIter = <<S::Struct as SplitFields>::StructOf<S::Family> as IdHolder>::IdIter;

    fn ids(&self) -> Self::IdIter {
        self.inner.ids()
    }
}

impl<Q: StructQuery, S: StructOfAble> Queryable<Q> for StructOf<S>
where
    <S::Struct as SplitFields>::StructOf<S::Family>: Queryable<Q>,
{
    fn get(&self, id: Self::Id) -> Option<<Q as StructQuery>::Item<'_>> {
        self.inner.get(id)
    }
}
