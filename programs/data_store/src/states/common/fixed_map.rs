use anchor_lang::solana_program::hash::hashv;

/// Fixed size map key.
pub type MapKey = [u8; 32];

/// Convert to fixed-size key.
pub fn to_key(key: &str) -> MapKey {
    hashv(&[key.as_bytes()]).to_bytes()
}

#[macro_export]
macro_rules! fixed_map {
    ($map:ident, $value:ty, $len:expr, $padding:expr) => {
        $crate::fixed_map!($map, str, $crate::states::common::fixed_map::to_key, $value, $len, $padding);
    };
    ($map:ident, $key:ty, $to_key:path, $value:ty, $len:expr, $padding:expr) => {
        paste::paste! {
            /// Entry.
            #[anchor_lang::zero_copy]
            #[cfg_attr(feature = "debug", derive(Debug))]
            struct [<$map Entry>] {
                key: [u8; 32],
                value: $value,
            }

            impl Default for [<$map Entry>] {
                fn default() -> Self {
                    Self {
                        key: Default::default(),
                        value: Default::default(),
                    }
                }
            }

            /// Fixed size map.
            #[anchor_lang::zero_copy]
            #[cfg_attr(feature = "debug", derive(Debug))]
            pub struct $map {
                data: [[<$map Entry>]; $len],
                count: usize,
                padding: [u8; $padding],
            }

            impl $crate::states::InitSpace for $map {
                const INIT_SPACE: usize = std::mem::size_of::<$map>();
            }

            #[cfg(test)]
            const_assert_eq!(
                std::mem::size_of::<$map>(),
                <$map as $crate::states::InitSpace>::INIT_SPACE
            );

            impl Default for $map {
                fn default() -> Self {
                    Self {
                        data: Default::default(),
                        count: 0,
                        padding: Default::default(),
                    }
                }
            }

            impl $map {
                fn binanry_search(&self, key: &$crate::states::common::fixed_map::MapKey) -> std::result::Result<usize, usize> {
                    self.data[..self.count].binary_search_by(|entry| entry.key.cmp(key))
                }

                /// Get.
                pub fn get(&self, key: &$key) -> Option<&$value> {
                    let key = $to_key(key);
                    self
                        .binanry_search(&key)
                        .ok()
                        .map(|index| &self.data[index].value)
                }

                /// Get mutable reference to the corressponding value.
                pub fn get_mut(&mut self, key: &$key) -> Option<&mut $value> {
                    let key = $to_key(key);
                    self
                        .binanry_search(&key)
                        .ok()
                        .map(|index| &mut self.data[index].value)
                }

                /// Insert.
                pub fn insert(&mut self, key: &$key, value: $value) -> Option<$value> {
                    self.insert_with_options(key, value, false).expect("must be success")
                }

                /// Insert with options.
                pub fn insert_with_options(
                    &mut self,
                    key: &$key,
                    value: $value,
                    new: bool,
                ) -> std::result::Result<Option<$value>, anchor_lang::error::Error> {
                    let key = $to_key(key);
                    match self.binanry_search(&key) {
                        Ok(index) => {
                            if new {
                                anchor_lang::err!($crate::DataStoreError::AlreadyExist)
                            } else {
                                let previous = std::mem::replace(&mut self.data[index].value, value);
                                Ok(Some(previous))
                            }
                        }
                        Err(index) => {
                            if self.count >= 32 {
                                anchor_lang::err!($crate::DataStoreError::ExceedMaxLengthLimit)
                            } else {
                                for i in (index..self.count).rev() {
                                    self.data[i + 1] = self.data[i];
                                }
                                self.data[index] = [<$map Entry>] { key, value };
                                self.count += 1;
                                Ok(None)
                            }
                        }
                    }
                }

                /// Remove.
                pub fn remove(&mut self, key: &$key) -> Option<$value> {
                    let key = $to_key(key);
                    self.binanry_search(&key).ok().map(|index| {
                        let value = std::mem::take(&mut self.data[index].value);
                        let len = self.count;
                        for i in index..len {
                            self.data[i] = self.data[i + 1];
                        }
                        self.data[len - 1] = [<$map Entry>]::default();
                        self.count -= 1;
                        value
                    })
                }

                /// Get length.
                pub fn len(&self) -> usize {
                    self.count
                }

                /// Is empty.
                pub fn is_empty(&self) -> bool {
                    self.count == 0
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use anchor_lang::solana_program::pubkey::Pubkey;

    fixed_map!(FixedFactorMap, u128, 32, 8);

    #[test]
    fn test_insert_and_get() {
        let mut map = FixedFactorMap::default();

        assert!(map.is_empty());

        assert_eq!(map.insert("key1", 123), None);
        assert_eq!(map.insert("key1", 234), Some(123));

        assert_eq!(map.insert("key2", 345), None);
        assert_eq!(map.insert("key2", 456), Some(345));

        assert_eq!(map.insert("key1", 789), Some(234));
        assert_eq!(map.get("key1"), Some(&789));

        *map.get_mut("key2").unwrap() = 42;
        assert_eq!(map.get("key2"), Some(&42));

        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_insert_and_remove() {
        let mut map = FixedFactorMap::default();

        assert_eq!(map.insert("key1", 123), None);
        assert_eq!(map.insert("key1", 234), Some(123));

        assert_eq!(map.insert("key2", 345), None);
        assert_eq!(map.insert("key2", 456), Some(345));

        assert_eq!(map.remove("key1"), Some(234));
        assert_eq!(map.insert("key1", 789), None);

        assert_eq!(map.len(), 2);
    }

    fn to_bytes(key: &Pubkey) -> [u8; 32] {
        key.to_bytes()
    }

    fixed_map!(RolesMap, Pubkey, to_bytes, u64, 32, 0);

    #[test]
    fn test_insert_and_get_for_roles_map() {
        let mut map = RolesMap::default();

        let address_1 = Pubkey::new_unique();
        let address_2 = Pubkey::new_unique();

        assert!(map.is_empty());

        assert_eq!(map.insert(&address_1, 123), None);
        assert_eq!(map.insert(&address_1, 234), Some(123));

        assert_eq!(map.insert(&address_2, 345), None);
        assert_eq!(map.insert(&address_2, 456), Some(345));

        assert_eq!(map.insert(&address_1, 789), Some(234));
        assert_eq!(map.get(&address_1), Some(&789));

        *map.get_mut(&address_2).unwrap() = 42;
        assert_eq!(map.get(&address_2), Some(&42));

        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_insert_and_remove_for_roles_map() {
        let mut map = RolesMap::default();

        let address_1 = Pubkey::new_unique();
        let address_2 = Pubkey::new_unique();

        assert_eq!(map.insert(&address_1, 123), None);
        assert_eq!(map.insert(&address_1, 234), Some(123));

        assert_eq!(map.insert(&address_2, 345), None);
        assert_eq!(map.insert(&address_2, 456), Some(345));

        assert_eq!(map.remove(&address_1), Some(234));
        assert_eq!(map.insert(&address_1, 789), None);

        assert_eq!(map.len(), 2);
    }
}
