use anchor_lang::{prelude::*, solana_program::hash::hashv};
use dual_vec_map::DualVecMap;

use crate::{states::InitSpace, DataStoreError};

/// Store for a dual vec map with a max length.
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

/// Store for a dual vec map with dynamic length.
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct DynamicMapStore<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K, V> Default for DynamicMapStore<K, V> {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Default::default(),
        }
    }
}

impl<K, V> DynamicMapStore<K, V>
where
    K: InitSpace,
    V: InitSpace,
{
    /// Get the space required by the store.
    pub fn init_space(num_pools: u8) -> usize {
        let len = num_pools as usize;
        (4 + K::INIT_SPACE * len) + (4 + V::INIT_SPACE * len)
    }
}

impl<K, V> DynamicMapStore<K, V> {
    /// As a map.
    pub fn as_map(&self) -> DualVecMap<&Vec<K>, &Vec<V>> {
        DualVecMap::from_sorted_stores_unchecked(&self.keys, &self.values)
    }

    /// As a mutable map.
    pub fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<K>, &mut Vec<V>> {
        DualVecMap::from_sorted_stores_unchecked(&mut self.keys, &mut self.values)
    }
}

// A map indexed by `u8`.
// Useful when the key type is a `repr(u8)` enum.
impl<V> DynamicMapStore<u8, V> {
    /// Initialize the store according to the given size.
    pub fn init_with(&mut self, num_keys: u8, mut value: impl FnMut(u8) -> V) {
        let mut map = DualVecMap::new_vecs();
        for key in 0..num_keys {
            map.insert(key, (value)(key));
        }
        let (keys, values) = map.into_inner();
        self.keys = keys;
        self.values = values;
    }

    /// Get the value corresponding to the given key.
    #[inline]
    pub fn get_with<T>(&self, key: impl Into<u8>, f: impl FnOnce(Option<&V>) -> T) -> T {
        (f)(self.as_map().get(&key.into()))
    }

    /// Get the value mutably corresponding to the given key,
    #[inline]
    pub fn get_mut_with<T>(
        &mut self,
        key: impl Into<u8>,
        f: impl FnOnce(Option<&mut V>) -> T,
    ) -> T {
        (f)(self.as_map_mut().get_mut(&key.into()))
    }
}

// FIXME: replace this with `DynamicMapStore` when anchor can parse the generated IDL.
pub(crate) mod pools {
    use crate::states::Pool;

    use super::*;

    /// Store for a dual vec map with dynamic length.
    #[derive(AnchorDeserialize, AnchorSerialize, Clone, Default)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Pools {
        keys: Vec<u8>,
        values: Vec<Pool>,
    }

    impl Pools {
        /// Get the space required by the store.
        pub fn init_space(num_pools: u8) -> usize {
            let len = num_pools as usize;
            (4 + u8::INIT_SPACE * len) + (4 + <Pool as InitSpace>::INIT_SPACE * len)
        }
    }

    impl Pools {
        /// As a map.
        pub fn as_map(&self) -> DualVecMap<&Vec<u8>, &Vec<Pool>> {
            DualVecMap::from_sorted_stores_unchecked(&self.keys, &self.values)
        }

        /// As a mutable map.
        pub fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<u8>, &mut Vec<Pool>> {
            DualVecMap::from_sorted_stores_unchecked(&mut self.keys, &mut self.values)
        }
    }

    impl Pools {
        /// Initialize the store according to the given size.
        pub fn init_with(&mut self, num_keys: u8, mut value: impl FnMut(u8) -> Pool) {
            let mut map = DualVecMap::new_vecs();
            for key in 0..num_keys {
                map.insert(key, (value)(key));
            }
            let (keys, values) = map.into_inner();
            self.keys = keys;
            self.values = values;
        }

        /// Get the value corresponding to the given key.
        #[inline]
        pub fn get_with<T>(&self, key: impl Into<u8>, f: impl FnOnce(Option<&Pool>) -> T) -> T {
            (f)(self.as_map().get(&key.into()))
        }

        /// Get the value mutably corresponding to the given key,
        #[inline]
        pub fn get_mut_with<T>(
            &mut self,
            key: impl Into<u8>,
            f: impl FnOnce(Option<&mut Pool>) -> T,
        ) -> T {
            (f)(self.as_map_mut().get_mut(&key.into()))
        }
    }
}
