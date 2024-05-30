use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct TokenRegister {
    #[max_len(10)]
    pub token_name: String,
    pub token_mint_account: Pubkey,
    pub settle_time: u64,
    pub settle_duration: u64,
}