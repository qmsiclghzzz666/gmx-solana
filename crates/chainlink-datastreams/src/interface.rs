use anchor_lang::{prelude::Pubkey, Ids};

pub use mock_chainlink_verifier::cpi::{accounts::VerifyContext, verify, verify_bulk};

/// Chainlink DataStreams Interface.
#[derive(Clone, Copy)]
pub struct ChainlinkDataStreamsInterface;

impl Ids for ChainlinkDataStreamsInterface {
    fn ids() -> &'static [anchor_lang::prelude::Pubkey] {
        #[cfg(not(feature = "no-mock"))]
        static IDS: &[Pubkey] = &[crate::mock::ID];

        #[cfg(feature = "no-mock")]
        static IDS: &[Pubkey] = &[];

        IDS
    }
}
