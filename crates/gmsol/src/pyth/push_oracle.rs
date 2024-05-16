use anchor_client::anchor_lang::solana_program::pubkey::Pubkey;
use data_store::states::Pyth;

/// Find Pyth Feed Account PDA.
pub fn find_pyth_feed_account(shard_id: u16, feed_id: [u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[shard_id.to_le_bytes().as_slice(), feed_id.as_slice()],
        &Pyth::PUSH_ORACLE_ID,
    )
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "pyth-solana-receiver-sdk")]
    #[test]
    fn test_sol_feed_account() {
        use super::{find_pyth_feed_account, Pubkey};
        use pyth_solana_receiver_sdk::price_update::get_feed_id_from_hex;
        use std::str::FromStr;

        let feed_id = get_feed_id_from_hex(
            "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
        )
        .unwrap();
        let expected_address =
            Pubkey::from_str("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE").unwrap();
        assert_eq!(find_pyth_feed_account(0, feed_id).0, expected_address);
    }
}
