use dyn_clone::{clone_trait_object, DynClone};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{Signer, SignerError},
};

/// Boxed Signer.
pub type BoxSigner = Box<dyn Signer>;

/// Boxed Clonable Signer.
pub type BoxClonableSigner = Box<dyn CloneableSigner>;

/// Clonable Signer.
pub trait CloneableSigner: Signer + DynClone {}

impl<T: Signer + Clone> CloneableSigner for T {}

clone_trait_object!(CloneableSigner);

impl Signer for BoxClonableSigner {
    fn pubkey(&self) -> Pubkey {
        (**self).pubkey()
    }

    fn sign_message(&self, message: &[u8]) -> Signature {
        (**self).sign_message(message)
    }

    fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
        (**self).try_pubkey()
    }

    fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
        (**self).try_sign_message(message)
    }

    fn is_interactive(&self) -> bool {
        (**self).is_interactive()
    }
}
