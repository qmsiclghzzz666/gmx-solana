use std::sync::Arc;

use anchor_client::solana_sdk::{self, pubkey::Pubkey, signer::Signer};

pub struct DynSigner(pub Arc<dyn Signer + Send + Sync>);

impl Signer for DynSigner {
    fn pubkey(&self) -> Pubkey {
        self.0.pubkey()
    }

    fn try_pubkey(&self) -> Result<Pubkey, solana_sdk::signer::SignerError> {
        self.0.try_pubkey()
    }

    fn sign_message(&self, message: &[u8]) -> solana_sdk::signature::Signature {
        self.0.sign_message(message)
    }

    fn try_sign_message(
        &self,
        message: &[u8],
    ) -> Result<solana_sdk::signature::Signature, solana_sdk::signer::SignerError> {
        self.0.try_sign_message(message)
    }

    fn is_interactive(&self) -> bool {
        self.0.is_interactive()
    }
}

/// Shared Signer.
pub type SignerRef = Arc<DynSigner>;

/// Create a new shared signer.
pub fn shared_signer(signer: impl Signer + Send + Sync + 'static) -> SignerRef {
    SignerRef::new(DynSigner(Arc::new(signer)))
}
