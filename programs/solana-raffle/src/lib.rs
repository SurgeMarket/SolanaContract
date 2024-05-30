use anchor_lang::prelude::*;
use instructions::*;
pub mod instructions;
pub mod state;

declare_id!("92K7tCfDyWrhEYcCJT5TurYP4mk5Uk1MYeuVkHW1MN1J");


#[program]
pub mod solana_raffle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, signer: Pubkey) -> Result<()> {
        init::initialize(ctx, signer)
    }

    pub fn update_signer(ctx: Context<UpdateSigner>, signer: Pubkey) -> Result<()> {
        init::update_signer(ctx, signer)
    }

    pub fn seller_settle<'info>(
        ctx: Context<'_, '_, '_, 'info, SettleOrderPayment<'info>>,
        ix_args: SettleOrderPaymentArgs,
    ) -> Result<()> {
        raffle::seller_settle(ctx, ix_args)
    }
    pub fn buyer_settle<'info>(
        ctx: Context<'_, '_, '_, 'info, SettleOrderPayment<'info>>,
        ix_args: SettleOrderPaymentArgs,
    ) -> Result<()> {
        raffle::buyer_settlee(ctx, ix_args)
    }

    pub fn list_token(ctx: Context<ListToken>, ix_args: RegisterTokenArgs) -> Result<()> {
        init::list_token(ctx, ix_args)
    }
}
