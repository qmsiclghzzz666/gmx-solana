use anchor_lang::{prelude::*, solana_program::system_program};
use anchor_spl::{
    associated_token::{create, get_associated_token_address, Create},
    token::{transfer, TokenAccount, Transfer},
};
use typed_builder::TypedBuilder;

/// Check if the given address is an ATA address.
pub fn is_associated_token_account(pubkey: &Pubkey, owner: &Pubkey, mint: &Pubkey) -> bool {
    let expected = get_associated_token_address(owner, mint);
    expected == *pubkey
}

/// Return whether the token account must be uninitialized.
pub fn must_be_uninitialized<'info>(account: &impl AsRef<AccountInfo<'info>>) -> bool {
    let info = account.as_ref();
    *info.owner == system_program::ID
}

#[derive(TypedBuilder)]
pub struct TransferAllFromEscrowToATA<'a, 'info> {
    system_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    owner: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    ata: AccountInfo<'info>,
    escrow: &'a Account<'info, TokenAccount>,
    escrow_authority: AccountInfo<'info>,
    seeds: &'a [&'a [u8]],
    init_if_needed: bool,
    #[builder(default)]
    skip_owner_check: bool,
}

impl<'a, 'info> TransferAllFromEscrowToATA<'a, 'info> {
    /// Transfer all tokens from escrow account to ATA.
    ///
    /// Return `false` if the transfer is required but the ATA is not initilaized.
    pub(crate) fn execute(self) -> Result<bool> {
        let Self {
            system_program,
            token_program,
            associated_token_program,
            payer,
            owner,
            mint,
            ata,
            escrow,
            escrow_authority,
            seeds,
            init_if_needed,
            skip_owner_check,
        } = self;

        let amount = escrow.amount;
        if amount != 0 {
            if must_be_uninitialized(&ata) {
                if !init_if_needed {
                    return Ok(false);
                }
                create(CpiContext::new(
                    associated_token_program,
                    Create {
                        payer,
                        associated_token: ata.clone(),
                        authority: owner.clone(),
                        mint,
                        system_program,
                        token_program: token_program.clone(),
                    },
                ))?;
            }

            let Ok(ata_owner) = anchor_spl::token::accessor::authority(&ata) else {
                msg!("the ATA is not a valid token account, skip the transfer");
                return Ok(false);
            };

            if ata_owner != owner.key() && !skip_owner_check {
                msg!("The ATA is not owned by the owner, skip the transfer");
                return Ok(false);
            }

            transfer(
                CpiContext::new(
                    token_program,
                    Transfer {
                        from: escrow.to_account_info(),
                        to: ata.to_account_info(),
                        authority: escrow_authority,
                    },
                )
                .with_signer(&[seeds]),
                amount,
            )?;
        }
        Ok(true)
    }
}
