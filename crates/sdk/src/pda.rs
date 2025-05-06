use solana_sdk::pubkey::Pubkey;

#[allow(unused_imports)]
use gmsol_programs::gmsol_store::accounts as store_accounts;

use crate::builders::NonceBytes;

/// Event authority SEED.
pub const EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";

/// Seed for [`Store`](store_accounts::Store).
pub const STORE_SEED: &[u8] = b"data_store";

/// Seed for store wallet.
pub const STORE_WALLET_SEED: &[u8] = b"store_wallet";

/// Seed for [`Market`](store_accounts::Market).
pub const MARKET_SEED: &[u8] = b"market";

/// Seed for [`Order`](store_accounts::Order).
pub const ORDER_SEED: &[u8] = b"order";

/// Seed for [`Position`](store_accounts::Position).
pub const POSITION_SEED: &[u8] = b"position";

/// Seed for [`User`](store_accounts::UserHeader).
pub const USER_SEED: &[u8] = b"user";

fn to_seed(key: &str) -> [u8; 32] {
    use solana_sdk::hash::hash;
    hash(key.as_bytes()).to_bytes()
}

/// Find the PDA for `event_authority` account.
pub fn find_event_authority_address(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[EVENT_AUTHORITY_SEED], program_id)
}

/// Find the PDA for [`Store`](store_accounts::Store) account.
pub fn find_store_address(key: &str, store_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[STORE_SEED, &to_seed(key)], store_program_id)
}

/// Find the PDA for store wallet account.
pub fn find_store_wallet_address(store: &Pubkey, store_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[STORE_WALLET_SEED, store.as_ref()], store_program_id)
}

/// Find the PDA for [`Order`](store_accounts::Order) account.
pub fn find_order_address(
    store: &Pubkey,
    owner: &Pubkey,
    nonce: &NonceBytes,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[ORDER_SEED, store.as_ref(), owner.as_ref(), nonce.as_ref()],
        store_program_id,
    )
}

/// Find the PDA for [`Market`](store_accounts::Market) account.
pub fn find_market_address(
    store: &Pubkey,
    token: &Pubkey,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[MARKET_SEED, store.as_ref(), token.as_ref()],
        store_program_id,
    )
}

/// Find PDA for [`User`](store_accounts::UserHeader) account.
pub fn find_user_address(
    store: &Pubkey,
    owner: &Pubkey,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[USER_SEED, store.as_ref(), owner.as_ref()],
        store_program_id,
    )
}

/// Create PDA for position.
pub fn find_position_address(
    store: &Pubkey,
    owner: &Pubkey,
    market_token: &Pubkey,
    collateral_token: &Pubkey,
    is_long: bool,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    // See the definition of `PositionKind`.
    let kind = if is_long { 1 } else { 2 };
    Pubkey::find_program_address(
        &[
            POSITION_SEED,
            store.as_ref(),
            owner.as_ref(),
            market_token.as_ref(),
            collateral_token.as_ref(),
            &[kind],
        ],
        store_program_id,
    )
}
