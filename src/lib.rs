// World
// - resources
// - archetypes (i.e. entity types): collections of components bundled together
//   specify the storage type for components (e.g. `Vec` or `Collection`)
//   - component storages
//     - component values (the actual data)

// -- Macros --

// Adapted from [soa_derive](https://github.com/lumol-org/soa-derive)
#[macro_export]
macro_rules! query {
    // Query a struct that implements `StructQuery`.
    // ($structof: expr, $query: ty) => {{
    //     <$query>::query(&$structof)
    // }};

    // Query a tuple by field names.
    ($structof: expr, ($($fields: tt)*) $(, $external: expr)* $(,)*) => {{
        $crate::query_impl!(@munch $structof.inner, {$($fields)*} -> [] $($external ,)*)
    }};
}

// Adapted from [soa_derive](https://github.com/lumol-org/soa-derive)
#[macro_export]
#[doc(hidden)]
macro_rules! query_impl {
    // @flatten creates a tuple-flattening closure for .map() call
    // Finish recursion
    (@flatten $p:pat => $tup:expr ) => {
        |$p| $tup
    };
    // Eat an element ($_iter) and add it to the current closure. Then recurse
    (@flatten $p:pat => ( $($tup:tt)* ) , $_iter:expr $( , $tail:expr )* ) => {
        $crate::query_impl!(@flatten ($p, a) => ( $($tup)*, a ) $( , $tail )*)
    };

    // The main code is emmited here: we create an iterator, zip it and then
    // map the zipped iterator to flatten it
    (@last , $first: expr, $($tail: expr,)*) => {
        ::std::iter::IntoIterator::into_iter($first)
            $(
                .zip($tail)
            )*
            .map(
                $crate::query_impl!(@flatten a => (a) $( , $tail )*)
            )
    };

    // Eat the last `mut $field` and then emit code
    (@munch $self: expr, {mut $field: ident} -> [$($output: tt)*] $($ext: expr ,)*) => {
        $crate::query_impl!(@last $($output)*, $self.$field.iter_mut(), $($ext, )*)
    };
    // Eat the last `$field` and then emit code
    (@munch $self: expr, {$field: ident} -> [$($output: tt)*] $($ext: expr ,)*) => {
        $crate::query_impl!(@last $($output)*, $self.$field.iter(), $($ext, )*)
    };

    // Eat the next `mut $field` and then recurse
    (@munch $self: expr, {mut $field: ident, $($tail: tt)*} -> [$($output: tt)*] $($ext: expr ,)*) => {
        $crate::query_impl!(@munch $self, {$($tail)*} -> [$($output)*, $self.$field.iter_mut()] $($ext, )*)
    };
    // Eat the next `$field` and then recurse
    (@munch $self: expr, {$field: ident, $($tail: tt)*} -> [$($output: tt)*] $($ext: expr ,)*) => {
        $crate::query_impl!(@munch $self, {$($tail)*} -> [$($output)*, $self.$field.iter()] $($ext, )*)
    };
}

// -- Structural Types --

pub trait Storage<T>: Default {
    type Id: Copy;
    type IdIter: Iterator<Item = Self::Id>;
    type Iterator<'a>: Iterator<Item = &'a T> + 'a
    where
        Self: 'a,
        T: 'a;
    type IteratorMut<'a>: Iterator<Item = &'a mut T> + 'a
    where
        Self: 'a,
        T: 'a;

    fn ids(&self) -> Self::IdIter;
    fn insert(&mut self, value: T) -> Self::Id;
    fn get(&self, id: Self::Id) -> Option<&T>;
    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T>;
    fn remove(&mut self, id: Self::Id) -> Option<T>;
    fn iter(&self) -> Self::Iterator<'_>;
    fn iter_mut(&mut self) -> Self::IteratorMut<'_>;
}

pub type ArchetypeId<A> = <<<A as Archetype>::Family as StorageFamily>::Storage<
    <A as Archetype>::Item,
> as Storage<<A as Archetype>::Item>>::Id;

pub trait Archetype: IdHolder + Default {
    type Item;
    type Family: StorageFamily;

    fn insert(&mut self, value: Self::Item) -> ArchetypeId<Self>;
    // fn get()
    // fn get_mut()
    fn remove(&mut self, id: ArchetypeId<Self>) -> Option<Self::Item>;
}

pub struct StructOf<S: StructOfAble> {
    pub inner: <S::Struct as SplitFields>::StructOf<S::Family>,
}

/// Implemented for "T's of structs" to convert into "structs of T's".
pub trait StructOfAble {
    type Struct: SplitFields;
    type Family: StorageFamily;
}

/// Implemented for structs to split into independent field.
pub trait SplitFields: Sized {
    type StructOf<F: StorageFamily>: Archetype;
}

/// A family of storages for different component types.
pub trait StorageFamily {
    type Id: Copy;
    type IdIter: Iterator<Item = Self::Id>;
    type Storage<T>: Storage<T, Id = Self::Id, IdIter = Self::IdIter>;
}

// -- Querying Types --

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

// -- Util impl --

impl<S: StructOfAble> std::ops::Deref for StructOf<S> {
    type Target = <S::Struct as SplitFields>::StructOf<S::Family>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: StructOfAble> std::ops::DerefMut for StructOf<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S: StructOfAble> Default for StructOf<S> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

// -- Vec storage --

impl<T> Storage<T> for Vec<T> {
    type Id = usize;

    type IdIter = std::ops::Range<usize>;

    type Iterator<'a> = <&'a [T] as IntoIterator>::IntoIter where Self: 'a, T: 'a;

    type IteratorMut<'a> = <&'a mut [T] as IntoIterator>::IntoIter where Self: 'a, T: 'a;

    fn insert(&mut self, value: T) -> Self::Id {
        let id = self.len();
        self.push(value);
        id
    }

    fn ids(&self) -> Self::IdIter {
        0..self.len()
    }

    fn get(&self, id: Self::Id) -> Option<&T> {
        self.as_slice().get(id)
    }

    fn get_mut(&mut self, id: Self::Id) -> Option<&mut T> {
        self.as_mut_slice().get_mut(id)
    }

    fn remove(&mut self, id: Self::Id) -> Option<T> {
        (id < self.len()).then(|| self.swap_remove(id))
    }

    fn iter(&self) -> Self::Iterator<'_> {
        self.as_slice().iter()
    }

    fn iter_mut(&mut self) -> Self::IteratorMut<'_> {
        self.as_mut_slice().iter_mut()
    }
}

impl<T: SplitFields> StructOfAble for Vec<T> {
    type Struct = T;
    type Family = VecFamily;
}

pub struct VecFamily;

impl StorageFamily for VecFamily {
    type Id = usize;
    type IdIter = std::ops::Range<usize>;
    type Storage<T> = Vec<T>;
}
