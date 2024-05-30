use crate::state::GlobalState;
use crate::state::TokenRegister;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [b"global"],
        bump,
        payer = payer
    )]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Debug)]
pub struct RegisterTokenArgs {
    pub token_name: String,
    pub token_mint_account: Pubkey,
    pub settle_time: u64,
    pub settle_duration: u64,
}

#[derive(Accounts)]
#[instruction(ix: RegisterTokenArgs)]
pub struct ListToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(seeds = [b"global"], bump)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        space = 8 + TokenRegister::INIT_SPACE,
        seeds = [b"token_register", ix.token_name.as_bytes()],
        bump,
        payer = payer
    )]
    pub token_register: Account<'info, TokenRegister>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct UpdateSigner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, seeds = [b"global"], bump)]
    pub global_state: Account<'info, GlobalState>,
}

pub fn initialize(ctx: Context<Initialize>, signer: Pubkey) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    global_state.signer = signer;
    msg!("Initialization completed");
    Ok(())
}

pub fn update_signer(ctx: Context<UpdateSigner>, signer: Pubkey) -> Result<()> {
    if ctx.accounts.global_state.signer != ctx.accounts.payer.key() {
        return err!(ErrorCode::ConstraintSigner);
    }
    let global_state = &mut ctx.accounts.global_state;
    global_state.signer = signer;
    Ok(())
}

pub fn list_token(ctx: Context<ListToken>, ix_args: RegisterTokenArgs) -> Result<()> {
    if ctx.accounts.global_state.signer != ctx.accounts.payer.key() {
        return err!(ErrorCode::ConstraintSigner);
    }
    ctx.accounts.token_register.token_name = ix_args.token_name;
    ctx.accounts.token_register.token_mint_account = ix_args.token_mint_account;
    ctx.accounts.token_register.settle_duration = ix_args.settle_duration;
    ctx.accounts.token_register.settle_time = ix_args.settle_time;
    msg!(
        "list_token {:?} completed",
        ctx.accounts.token_register.token_name
    );
    Ok(())
}
