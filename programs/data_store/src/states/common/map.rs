use anchor_lang::{prelude::*, solana_program::hash::hashv};
use dual_vec_map::DualVecMap;

use crate::{states::InitSpace, DataStoreError};

/// Store for a dual vec map.
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MapStore<K, V, const MAX_LEN: usize> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K, V, const MAX_LEN: usize> Default for MapStore<K, V, MAX_LEN> {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Default::default(),
        }
    }
}

impl<K, V, const MAX_LEN: usize> Space for MapStore<K, V, MAX_LEN>
where
    K: InitSpace,
    V: InitSpace,
{
    const INIT_SPACE: usize = (4 + K::INIT_SPACE * MAX_LEN) + (4 + V::INIT_SPACE * MAX_LEN);
}

impl<K, V, const MAX_LEN: usize> MapStore<K, V, MAX_LEN> {
    /// As a map.
    pub fn as_map(&self) -> DualVecMap<&Vec<K>, &Vec<V>> {
        DualVecMap::from_sorted_stores_unchecked(&self.keys, &self.values)
    }

    /// As a mutable map.
    pub fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<K>, &mut Vec<V>> {
        DualVecMap::from_sorted_stores_unchecked(&mut self.keys, &mut self.values)
    }
}

// TODO: Define type alias (e.g., `ConfigStore`) when it won't cause the compiler to panic.
impl<V, const MAX_LEN: usize> MapStore<[u8; 32], V, MAX_LEN> {
    /// Get the internal hash key with the given namespace and key.
    #[inline]
    pub fn hash(namespace: &str, key: &str) -> [u8; 32] {
        hashv(&[namespace.as_bytes(), key.as_bytes()]).to_bytes()
    }

    /// Get the value corressponding to the given namespace and key,
    /// with the given function.
    pub fn get_with<T>(&self, namespace: &str, key: &str, f: impl FnOnce(Option<&V>) -> T) -> T {
        let hash = Self::hash(namespace, key);
        let map = self.as_map();
        let value = map.get(&hash);
        (f)(value)
    }

    /// Get the value corresponding to the given namespace and key,
    /// with the given function.
    pub fn get_mut_with<T>(
        &mut self,
        namespace: &str,
        key: &str,
        f: impl FnOnce(Option<&mut V>) -> T,
    ) -> T {
        let hash = Self::hash(namespace, key);
        let mut map = self.as_map_mut();
        let value = map.get_mut(&hash);
        (f)(value)
    }

    /// Insert value with the given namespace and key.
    pub fn insert(&mut self, namespace: &str, key: &str, value: V) -> Option<V> {
        let hash = Self::hash(namespace, key);
        self.as_map_mut().insert(hash, value).map(|(_, v)| v)
    }

    /// Insert value with the given namespace and key,
    /// return an error if the given key in the namespace already exists.
    pub fn insert_new(&mut self, namespace: &str, key: &str, value: V) -> Result<()> {
        let hash = Self::hash(namespace, key);
        self.as_map_mut()
            .try_insert(hash, value)
            .map_err(|_| DataStoreError::InvalidArgument)?;
        Ok(())
    }

    /// Remove the value corresponding to the given namespace and key.
    pub fn remove(&mut self, namespace: &str, key: &str) -> Option<V> {
        let hash = Self::hash(namespace, key);
        self.as_map_mut().remove(&hash).map(|(_, v)| v)
    }
}
