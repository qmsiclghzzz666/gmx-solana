use std::{
    borrow::Borrow,
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

/// Store for immutable operations.
pub trait Store {
    /// Element Type.
    type Value;

    /// Returns the number of elements.
    fn len(&self) -> usize;

    /// Returns the element at the given `index`.
    ///
    /// One must make sure that this method will always return `Some`
    /// if `index < len`.
    fn get(&self, index: usize) -> Option<&Self::Value>;

    /// Returns whether the store is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if the elements of this store are sorted using the given comparator function.
    ///
    /// See the `is_sorted_by` method of [`std::slice`](std::slice) for more information.
    fn is_sorted_by<'a, F>(&'a self, mut compare: F) -> bool
    where
        F: FnMut(&'a Self::Value, &'a Self::Value) -> Option<Ordering>,
    {
        if self.is_empty() {
            return true;
        }
        (0..self.len())
            .map(|idx| (idx, idx + 1))
            .take(self.len() - 1)
            .all(|(a, b)| {
                compare(
                    self.get(a).expect("shouldn't out of range"),
                    self.get(b).expect("shouldn't out of range"),
                )
                .map_or(false, Ordering::is_le)
            })
    }

    /// Checks if the elements of this store are sorted.
    ///
    /// See the `is_sorted` method of [`std::slice`](std::slice) for more information.
    fn is_sorted(&self) -> bool
    where
        Self::Value: PartialOrd,
    {
        self.is_sorted_by(|a, b| a.partial_cmp(b))
    }
}

/// Store that is binary searchable.
pub trait SearchStore: Store {
    /// Binary searches this store with a comparator function.
    ///
    /// See the `binary_search_by` method of [`std::slice`](std::slice) for more details.
    fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Self::Value) -> Ordering;

    /// Binary searches this store for the given key.
    ///
    /// See the `binary_search` method of [`std::slice`](std::slice) for more details.
    #[inline]
    fn binary_search<Q: ?Sized>(&self, key: &Q) -> Result<usize, usize>
    where
        Self::Value: Borrow<Q>,
        Q: Ord,
    {
        self.binary_search_by(|k| k.borrow().cmp(key))
    }
}

/// Store for mutable operations.
pub trait StoreMut: Store {
    /// Insert an element at the given index.
    ///
    /// # Panic
    /// Panics if `index` is greater than the length.
    fn insert(&mut self, index: usize, element: Self::Value);

    /// Remove an element at the given index.
    ///
    /// # Panic
    /// Panics if `index` is greater than the length.
    fn remove(&mut self, index: usize) -> Self::Value;

    /// Remove all elements from the store.
    fn clear(&mut self);

    /// Get a mutable reference of the element at the given index.
    ///
    /// One must make sure that this method will always return `Some`
    /// if `index < len`.
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Value>;
}

impl<T> Store for Vec<T> {
    type Value = T;

    fn len(&self) -> usize {
        self.deref().len()
    }

    fn get(&self, index: usize) -> Option<&Self::Value> {
        self.deref().get(index)
    }
}

impl<T> SearchStore for Vec<T> {
    fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Self::Value) -> Ordering,
    {
        self.deref().binary_search_by(f)
    }
}

impl<T> StoreMut for Vec<T> {
    fn insert(&mut self, index: usize, element: Self::Value) {
        self.insert(index, element)
    }

    fn remove(&mut self, index: usize) -> Self::Value {
        self.remove(index)
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Value> {
        self.deref_mut().get_mut(index)
    }
}

impl<'b, S> Store for &'b mut S
where
    S: Store,
{
    type Value = S::Value;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn get(&self, index: usize) -> Option<&Self::Value> {
        (**self).get(index)
    }

    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    fn is_sorted(&self) -> bool
    where
        Self::Value: PartialOrd,
    {
        (**self).is_sorted()
    }

    fn is_sorted_by<'a, F>(&'a self, compare: F) -> bool
    where
        F: FnMut(&'a Self::Value, &'a Self::Value) -> Option<Ordering>,
    {
        (**self).is_sorted_by(compare)
    }
}

impl<'b, S> SearchStore for &'b mut S
where
    S: SearchStore,
{
    fn binary_search<Q: ?Sized>(&self, key: &Q) -> Result<usize, usize>
    where
        Self::Value: Borrow<Q>,
        Q: Ord,
    {
        (**self).binary_search(key)
    }

    fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Self::Value) -> Ordering,
    {
        (**self).binary_search_by(f)
    }
}

impl<'b, S> StoreMut for &'b mut S
where
    S: StoreMut,
{
    fn insert(&mut self, index: usize, element: Self::Value) {
        (**self).insert(index, element)
    }

    fn remove(&mut self, index: usize) -> Self::Value {
        (**self).remove(index)
    }

    fn clear(&mut self) {
        (**self).clear()
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Value> {
        (**self).get_mut(index)
    }
}

impl<'b, S> Store for &'b S
where
    S: Store,
{
    type Value = S::Value;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn get(&self, index: usize) -> Option<&Self::Value> {
        (**self).get(index)
    }

    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    fn is_sorted(&self) -> bool
    where
        Self::Value: PartialOrd,
    {
        (**self).is_sorted()
    }

    fn is_sorted_by<'a, F>(&'a self, compare: F) -> bool
    where
        F: FnMut(&'a Self::Value, &'a Self::Value) -> Option<Ordering>,
    {
        (**self).is_sorted_by(compare)
    }
}

impl<'b, S> SearchStore for &'b S
where
    S: SearchStore,
{
    fn binary_search<Q: ?Sized>(&self, key: &Q) -> Result<usize, usize>
    where
        Self::Value: Borrow<Q>,
        Q: Ord,
    {
        (**self).binary_search(key)
    }

    fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Self::Value) -> Ordering,
    {
        (**self).binary_search_by(f)
    }
}
