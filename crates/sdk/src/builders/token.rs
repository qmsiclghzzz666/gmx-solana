use std::collections::HashSet;

use typed_builder::TypedBuilder;

use crate::{utils::serde::StringPubkey, AtomicInstructionGroup, IntoAtomicInstructionGroup};

use super::utils::prepare_ata;

/// Prepares token accounts for the owner.
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypedBuilder)]
pub struct PrepareTokenAccounts {
    /// Payer.
    #[builder(setter(into))]
    pub payer: StringPubkey,
    /// Owner.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option, into))]
    pub owner: Option<StringPubkey>,
    /// Tokens.
    #[builder(setter(into))]
    pub tokens: HashSet<StringPubkey>,
    /// Token Program ID.
    #[builder(default = StringPubkey(anchor_spl::token::ID), setter(into))]
    pub token_program: StringPubkey,
}

impl IntoAtomicInstructionGroup for PrepareTokenAccounts {
    type Hint = ();

    fn into_atomic_instruction_group(
        self,
        _hint: &Self::Hint,
    ) -> crate::Result<AtomicInstructionGroup> {
        let payer = self.payer.0;
        let owner = self.owner.as_deref().copied().unwrap_or(payer);
        let insts = self
            .tokens
            .iter()
            .map(|token| {
                prepare_ata(&payer, &owner, Some(token), &self.token_program)
                    .unwrap()
                    .1
            })
            .collect();
        Ok(insts)
    }
}

/// Wraps the native token into its corresponding associated token account
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypedBuilder)]
pub struct WrapNative {
    /// Owner.
    #[builder(setter(into))]
    pub owner: StringPubkey,
    /// Lamports.
    pub lamports: u64,
}

impl IntoAtomicInstructionGroup for WrapNative {
    type Hint = ();

    fn into_atomic_instruction_group(
        self,
        _hint: &Self::Hint,
    ) -> crate::Result<AtomicInstructionGroup> {
        use anchor_spl::token::spl_token::{
            instruction::sync_native, native_mint::ID as NATIVE_MINT,
        };
        use anchor_spl::token::ID;
        use gmsol_programs::anchor_lang::solana_program::system_instruction::transfer;

        let owner = self.owner.0;
        let (ata, prepare) = prepare_ata(&owner, &owner, Some(&NATIVE_MINT), &ID).unwrap();
        let transfer = transfer(&owner, &ata, self.lamports);
        let sync = sync_native(&ID, &ata).unwrap();

        Ok([prepare, transfer, sync].into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use super::*;

    #[test]
    fn prepare_token_accounts() {
        let tokens = [Pubkey::new_unique().into(), Pubkey::new_unique().into()];
        let insts = PrepareTokenAccounts::builder()
            .payer(Pubkey::new_unique())
            .tokens(tokens)
            .build()
            .into_atomic_instruction_group(&())
            .unwrap();
        assert_eq!(insts.len(), tokens.len());
    }

    #[test]
    fn wrap_native() {
        WrapNative::builder()
            .owner(Pubkey::new_unique())
            .lamports(1_000_000_000)
            .build()
            .into_atomic_instruction_group(&())
            .unwrap();
    }
}
