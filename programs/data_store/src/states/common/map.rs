use anchor_lang::prelude::*;
use dual_vec_map::DualVecMap;

use crate::states::InitSpace;

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
