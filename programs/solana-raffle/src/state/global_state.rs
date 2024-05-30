use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub signer: Pubkey,

    pub fee_account: Pubkey,
    pub pledge_rate: u128,
    pub cancel_rate: u128,
    pub blackout_rate: u128,
}
