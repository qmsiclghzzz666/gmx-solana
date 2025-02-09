use dyn_clone::{clone_trait_object, DynClone};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{Signer, SignerError},
};

/// Boxed Signer.
pub type BoxSigner = Box<dyn Signer>;

/// Boxed Clonable Signer.
#[derive(Clone)]
pub struct BoxClonableSigner<'a>(pub Box<dyn CloneableSigner + 'a>);

impl<'a> BoxClonableSigner<'a> {
    /// Create from `impl Signer`.
    pub fn new(signer: impl Signer + Clone + 'a) -> Self {
        Self(Box::new(signer))
    }
}

/// Clonable Signer.
pub trait CloneableSigner: Signer + DynClone {}

impl<T: Signer + Clone> CloneableSigner for T {}

clone_trait_object!(CloneableSigner);

impl Signer for BoxClonableSigner<'_> {
    fn pubkey(&self) -> Pubkey {
        self.0.pubkey()
    }

    fn sign_message(&self, message: &[u8]) -> Signature {
        self.0.sign_message(message)
    }

    fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
        self.0.try_pubkey()
    }

    fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
        self.0.try_sign_message(message)
    }

    fn is_interactive(&self) -> bool {
        self.0.is_interactive()
    }
}
