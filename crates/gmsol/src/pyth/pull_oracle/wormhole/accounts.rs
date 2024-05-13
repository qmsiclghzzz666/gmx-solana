use anchor_client::{
    anchor_lang::ToAccountMetas,
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey},
};

pub(super) struct InitEncodedVaa {
    pub(super) write_authority: Pubkey,
    pub(super) encoded_vaa: Pubkey,
}

impl ToAccountMetas for InitEncodedVaa {
    fn to_account_metas(&self, _is_signer: Option<bool>) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(self.write_authority, true),
            AccountMeta::new(self.encoded_vaa, false),
        ]
    }
}
