use std::ops::Deref;

use anchor_client::{
    solana_sdk::{instruction::Instruction, signer::Signer},
    Client, Program,
};

use self::wormhole::WORMHOLE_PROGRAM_ID;

pub use self::wormhole::WormholeOps;

/// Wormhole Ops.
pub mod wormhole;

/// Pyth Instructions.
#[derive(Debug, Default)]
pub struct PythIxs {
    post: Vec<Instruction>,
    close: Vec<Instruction>,
}

/// Pyth Pull Oracle.
pub trait PythPullOracle<C> {
    /// Get Pyth Program.
    fn pyth(&self) -> crate::Result<Program<C>>;

    /// Get Wormhole Program.
    fn wormhole(&self) -> crate::Result<Program<C>>;
}

impl<S, C> PythPullOracle<C> for Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn pyth(&self) -> crate::Result<Program<C>> {
        Ok(self.program(pyth_solana_receiver_sdk::ID)?)
    }

    fn wormhole(&self) -> crate::Result<Program<C>> {
        Ok(self.program(WORMHOLE_PROGRAM_ID)?)
    }
}
