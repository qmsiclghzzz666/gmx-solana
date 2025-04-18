use solana_sdk::pubkey::Pubkey;
use typed_builder::TypedBuilder;

use crate::{pda, utils::serde::StringPubkey};

/// Instruction builders related to token.
pub mod token;

/// Instruction builders related to order.
pub mod order;

pub(crate) mod utils;

/// Nonce Bytes.
pub type NonceBytes = StringPubkey;

/// A store program.
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypedBuilder)]
pub struct StoreProgram {
    /// Program ID.
    #[cfg_attr(serde, serde(with = "crate::utils::serde::pubkey"))]
    pub id: Pubkey,
    /// Store address.
    #[cfg_attr(serde, serde(with = "crate::utils::serde::pubkey"))]
    pub store: Pubkey,
}

impl Default for StoreProgram {
    fn default() -> Self {
        use gmsol_programs::gmsol_store::ID;
        Self {
            id: ID,
            store: pda::find_store_address("", &ID).0,
        }
    }
}

impl StoreProgram {
    /// Find order address.
    pub fn find_order_address(&self, owner: &Pubkey, nonce: &NonceBytes) -> Pubkey {
        pda::find_order_address(&self.store, owner, nonce, &self.id).0
    }

    /// Find market address.
    pub fn find_market_address(&self, market_token: &Pubkey) -> Pubkey {
        pda::find_market_address(&self.store, market_token, &self.id).0
    }

    /// Find user address.
    pub fn find_user_address(&self, owner: &Pubkey) -> Pubkey {
        pda::find_user_address(&self.store, owner, &self.id).0
    }

    /// Find position address.
    pub fn find_position_address(
        &self,
        owner: &Pubkey,
        market_token: &Pubkey,
        collateral_token: &Pubkey,
        is_long: bool,
    ) -> Pubkey {
        pda::find_position_address(
            &self.store,
            owner,
            market_token,
            collateral_token,
            is_long,
            &self.id,
        )
        .0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_store_program() {
        let program = StoreProgram::default();
        assert_eq!(
            program.store.to_string(),
            "CTDLvGGXnoxvqLyTpGzdGLg9pD6JexKxKXSV8tqqo8bN"
        );
    }

    #[cfg(serde)]
    #[test]
    fn serde() {
        let program = StoreProgram::default();
        assert_eq!(
            serde_json::to_string(&program).unwrap(),
            r#"{"id":"Gmso1uvJnLbawvw7yezdfCDcPydwW2s2iqG3w6MDucLo","store":"CTDLvGGXnoxvqLyTpGzdGLg9pD6JexKxKXSV8tqqo8bN"}"#
        );
    }
}
