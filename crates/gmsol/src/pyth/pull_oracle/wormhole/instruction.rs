use anchor_client::anchor_lang::{
    prelude::{borsh, AnchorSerialize},
    Discriminator, InstructionData,
};

#[derive(AnchorSerialize)]
pub(super) struct InitEncodedVaa {}

impl InstructionData for InitEncodedVaa {}

impl Discriminator for InitEncodedVaa {
    const DISCRIMINATOR: [u8; 8] = [209, 193, 173, 25, 91, 202, 181, 218];
}
