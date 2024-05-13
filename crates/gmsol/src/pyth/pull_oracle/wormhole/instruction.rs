use anchor_client::anchor_lang::{
    prelude::{borsh, AnchorSerialize},
    Discriminator, InstructionData,
};

#[derive(AnchorSerialize)]
pub(super) struct InitEncodedVaa {}

impl Discriminator for InitEncodedVaa {
    const DISCRIMINATOR: [u8; 8] = [209, 193, 173, 25, 91, 202, 181, 218];
}

impl InstructionData for InitEncodedVaa {}

#[derive(AnchorSerialize)]
pub(super) struct WriteEncodedVaa {
    pub(super) index: u32,
    pub(super) data: Vec<u8>,
}

impl Discriminator for WriteEncodedVaa {
    const DISCRIMINATOR: [u8; 8] = [199, 208, 110, 177, 150, 76, 118, 42];
}

impl InstructionData for WriteEncodedVaa {}
