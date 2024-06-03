//! Map-like data structure backed by sorted vectors.

/// Traits that required by [`DualVecMap`].
pub mod store;

use std::{borrow::Borrow, cmp::Ordering};

pub use self::store::{SearchStore, Store, StoreMut};

/// A map-like structure backed by a pair of vector-like stores:
/// one storing keys and the other storing associated values.
pub struct DualVecMap<K, V> {
    keys: K,
    values: V,
}

impl<K, V> DualVecMap<Vec<K>, Vec<V>> {
    /// Create a new empty [`DualVecMap`] from [`Vec`]s.
    pub fn new_vecs() -> Self {
        Self::from_sorted_stores_unchecked(Vec::default(), Vec::default())
    }
}

impl<K, V> DualVecMap<K, V> {
    /// Into inner stores.
    pub fn into_inner(self) -> (K, V) {
        (self.keys, self.values)
    }

    /// Create from sorted `keys` and `values` stores unchecked.
    /// One must make sure that:
    /// - `keys` is sorted and have no duplicate values.
    /// - `values` has the same length of `keys`.
    #[inline]
    pub const fn from_sorted_stores_unchecked(keys: K, values: V) -> Self {
        Self { keys, values }
    }

    /// Create from sorted `keys` and `values` stores.
    /// # Error
    /// Returns error if:
    /// - `keys` is not sorted or have duplicate values.
    /// - the length of `keys` and `values` mismatched.
    pub fn try_from_stores(keys: K, values: V) -> Result<Self, DualVecMapError>
    where
        K: Store,
        K::Value: PartialOrd,
        V: Store,
    {
        if keys.len() != values.len() {
            return Err(DualVecMapError::InvalidStores);
        }

        let is_strictly_sorted = keys.is_sorted_by(|a, b| {
            let ordering = a.partial_cmp(b)?;
            match ordering {
                Ordering::Equal => None,
                ordering => Some(ordering),
            }
        });
        if !is_strictly_sorted {
            return Err(DualVecMapError::InvalidStores);
        }

        Ok(Self::from_sorted_stores_unchecked(keys, values))
    }

    /// Returns the number of the elements (key-value pairs) in the map.
    pub fn len(&self) -> usize
    where
        K: Store,
    {
        self.keys.len()
    }

    /// Returns whether the map is empty.
    pub fn is_empty(&self) -> bool
    where
        K: Store,
    {
        self.keys.is_empty()
    }

    /// Remove all elements (key-value pairs) from the map.
    pub fn clear(&mut self)
    where
        K: StoreMut,
        V: StoreMut,
    {
        self.keys.clear();
        self.values.clear();
    }
}

impl<K, V> DualVecMap<K, V>
where
    K: SearchStore,
    V: Store,
{
    /// Binary search the map with `f` for a key, returning the value associated to it.
    pub fn get_by(&self, f: impl FnMut(&K::Value) -> Ordering) -> Option<&V::Value> {
        let index = self.keys.binary_search_by(f).ok()?;
        self.values.get(index)
    }

    /// Get the value associated to the key.
    pub fn get<Q>(&self, key: &Q) -> Option<&V::Value>
    where
        K::Value: Borrow<Q>,
        Q: ?Sized + Ord,
    {
        let index = self.keys.binary_search(key).ok()?;
        self.values.get(index)
    }

    /// Binary search the map with `f` for a key, returning the mutable reference
    /// to the value associated to it.
    pub fn get_mut_by(&mut self, f: impl FnMut(&K::Value) -> Ordering) -> Option<&mut V::Value>
    where
        V: StoreMut,
    {
        let index = self.keys.binary_search_by(f).ok()?;
        self.values.get_mut(index)
    }

    /// Get a mutable reference of the value associated to the key.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V::Value>
    where
        K::Value: Borrow<Q>,
        Q: ?Sized + Ord,
        V: StoreMut,
    {
        let index = self.keys.binary_search(key).ok()?;
        self.values.get_mut(index)
    }
}

impl<K, V> DualVecMap<K, V>
where
    K: SearchStore + StoreMut,
    K::Value: Ord,
    V: StoreMut,
{
    /// Attempts to insert a unique entry into the map.
    /// - If `key` is not in the map, inserts it with the corresponding `value` and returns `None`.
    /// - If `key` is already in the map, no change is made, and the `key` and `value` are returned.
    pub fn try_insert(
        &mut self,
        key: K::Value,
        value: V::Value,
    ) -> Result<(), (K::Value, V::Value)> {
        match self.keys.binary_search(&key) {
            Ok(_) => Err((key, value)),
            Err(index) => {
                self.keys.insert(index, key);
                self.values.insert(index, value);
                Ok(())
            }
        }
    }

    /// Insert `value` with `key`.
    /// - If `key` is not in the map, inserts it with the corresponding `value` and returns `None`.
    /// - If `key` is already in the map, updates the associated value with the given, and returns the `key` and the previous `value`.
    pub fn insert(&mut self, key: K::Value, mut value: V::Value) -> Option<(K::Value, V::Value)> {
        match self.keys.binary_search(&key) {
            Ok(index) => {
                let previous = self.values.get_mut(index).expect("must exist");
                std::mem::swap(previous, &mut value);
                Some((key, value))
            }
            Err(index) => {
                self.keys.insert(index, key);
                self.values.insert(index, value);
                None
            }
        }
    }

    /// Remove the key-value pair associated to the `key`, returning it if it exists.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<(K::Value, V::Value)>
    where
        K::Value: Borrow<Q>,
        Q: ?Sized + Ord,
    {
        match self.keys.binary_search(key) {
            Ok(found) => {
                let key = self.keys.remove(found);
                let value = self.values.remove(found);
                Some((key, value))
            }
            Err(_) => None,
        }
    }
}

/// Errors for [`DualVecMap`].
#[derive(Debug, thiserror::Error)]
pub enum DualVecMapError {
    /// Invalid keys or values stores.
    #[error("invalid keys or values stores")]
    InvalidStores,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_operations() {
        let mut map = DualVecMap::new_vecs();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        map.insert("hello", 1);
        assert!(!map.is_empty());
        assert!(map.len() == 1);
        assert_eq!(map.get("hello"), Some(&1));
        map.insert("world", 2);
        map.insert("apple", 3);
        assert!(map.len() == 3);
        assert_eq!(map.get("hello"), Some(&1));
        assert_eq!(map.get("world"), Some(&2));
        assert_eq!(map.get("apple"), Some(&3));
        assert!(map.keys.is_sorted());
        assert_eq!(map.remove("hello"), Some(("hello", 1)));
        let world = map.get_mut("world").unwrap();
        *world = 5;
        assert_eq!(map.get("hello"), None);
        assert_eq!(map.get("world"), Some(&5));
        assert_eq!(map.get("apple"), Some(&3));
        assert!(map.keys.is_sorted());
        assert!(map.len() == 2);
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn from_references() {
        let mut keys = Vec::from([1, 3, 5]);
        let mut values = Vec::from([2, 4, 6]);
        let mut map = DualVecMap::try_from_stores(&mut keys, &mut values).expect("must be ok");
        assert!(!map.is_empty());
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&1), Some(&2));
        assert_eq!(map.get(&3), Some(&4));
        assert_eq!(map.get(&5), Some(&6));
        map.insert(2, 3);
        assert_eq!(map.len(), 4);
        assert_eq!(map.get(&1), Some(&2));
        assert_eq!(map.get(&2), Some(&3));
        assert_eq!(map.get(&3), Some(&4));
        assert_eq!(map.get(&5), Some(&6));
        assert_eq!(map.remove(&3), Some((3, 4)));
        assert_eq!(keys, [1, 2, 5]);
        assert_eq!(values, [2, 3, 6]);
    }
}
