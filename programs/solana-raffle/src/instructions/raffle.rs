use std::mem::size_of;

use crate::state::{GlobalState, TokenRegister};
use anchor_lang::solana_program::{hash::hash, sysvar::instructions as instructions_sysvar_module};
use anchor_spl::token::{self, Mint, TokenAccount, Transfer};

use anchor_lang::solana_program::ed25519_program::ID as ED25519_PROGRAM_ID;
use {anchor_lang::prelude::*, anchor_spl::token::Token};

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Debug)]
pub struct SettleOrderPaymentArgs {
    pub token: String,
    pub order_id: u128,
    pub nonce: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Debug)]
pub struct SettleFullOrder {
    pub order_id: u128,
    pub token: String,
    pub nonce: u128,
    pub settle_amount: u64,
    pub sell_order_value: u64,
    pub side: String,
    pub customer: Pubkey,
    pub order_ids: Vec<u128>,
    pub sellet_amounts: Vec<u128>,
}

#[account]
pub struct NonceAccount {
    pub nonce: u128,
}


#[derive(Accounts)]
#[instruction(ix_args: SettleOrderPaymentArgs)]
pub struct SettleOrderPayment<'info> {
    #[account(mut)]
    pub customer: Signer<'info>,

    #[account(seeds = [b"global"], bump)]
    pub global_state: Account<'info, GlobalState>,


    #[account(
        init,
        payer = customer,
        seeds = [&hash(format!("nonce_{}", ix_args.nonce).as_bytes()).to_bytes()],
        bump,
        space = 8 + size_of::<NonceAccount>()
    )]
    pub nonce_account: Account<'info, NonceAccount>,


    // #[account(
    //     seeds = [b"token_register", ix_args.token.as_bytes().as_ref()],
    //     bump,
    // )]
    // pub token_register: Account<'info, TokenRegister>,

    #[account(mut)]
    pub spl_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub stake_account_ata: Account<'info, TokenAccount>, 

    #[account(
        mut,
        associated_token::mint = spl_token_mint,
        associated_token::authority = customer
    )]
    pub customer_account_ata: Box<Account<'info, TokenAccount>>,

    #[account(address = instructions_sysvar_module::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

const EXPECTED_PUBLIC_KEY_OFFSET: usize = 16;
const EXPECTED_PUBLIC_KEY_RANGE: std::ops::Range<usize> =
    EXPECTED_PUBLIC_KEY_OFFSET..(EXPECTED_PUBLIC_KEY_OFFSET + 32);

fn validate_ed25519_ix(ix: &anchor_lang::solana_program::instruction::Instruction) -> bool {
    if ix.program_id != ED25519_PROGRAM_ID || ix.accounts.len() != 0 {
        return false;
    }
    let ix_data = &ix.data;
    let public_key_offset = &ix_data[6..=7];
    let exp_public_key_offset = u16::try_from(EXPECTED_PUBLIC_KEY_OFFSET)
        .unwrap()
        .to_le_bytes();
    let expected_num_signatures: u8 = 1;
    return public_key_offset       == &exp_public_key_offset                        && // pulic_key in expected offset (16)
        &[ix_data[0]]           == &expected_num_signatures.to_le_bytes()        && // num_signatures is 1
        &[ix_data[1]]           == &[0]                                          && // padding is 0
        &ix_data[4..=5]         == &u16::MAX.to_le_bytes()                       && // signature_instruction_index is not defined by user (default value)
        &ix_data[8..=9]         == &u16::MAX.to_le_bytes()                       && // public_key_instruction_index is not defined by user (default value)
        &ix_data[14..=15]       == &u16::MAX.to_le_bytes(); // message_instruction_index is not defined by user (default value)
}

#[event]
pub struct NewSettleEvent {
    #[index]
    pub order_id: u128,
    pub user_account: Pubkey,
    pub nonce: u128,
    pub settle_amount: u64,
    pub side: String,
    pub token: String,
    pub timestamp: i64,
    pub order_ids: Vec<u128>,
    pub sellet_amounts: Vec<u128>,
}

pub fn seller_settle<'info>(
    ctx: Context<'_, '_, '_, 'info, SettleOrderPayment<'info>>,
    ix_args: SettleOrderPaymentArgs,
) -> Result<()> {
   
    msg!("seller_settle");
    msg!("token {}", ix_args.token);

    let token_program = &ctx.accounts.token_program;
    let ix = instructions_sysvar_module::get_instruction_relative(
        -1,
        &ctx.accounts.instructions_sysvar,
    )?;
    if !validate_ed25519_ix(&ix) {
        return err!(ErrorCode::InstructionMissing);
    }

    if ctx.accounts.nonce_account.nonce >0 {
        return err!(ErrorCode::IdlAccountNotEmpty);
    }

    let pub_key: Pubkey = Pubkey::new(&ix.data[EXPECTED_PUBLIC_KEY_RANGE]);


    if pub_key != ctx.accounts.global_state.signer {
        return err!(ErrorCode::ConstraintSigner);
    }

    let order_data = &ix.data[112..];
    let order = deserialize_order(&order_data.to_vec());
    msg!("order {:?}", order);

    match &order {
        Ok(order_value) => {
            if order_value.order_id != ix_args.order_id || order_value.token != ix_args.token {
                return err!(ErrorCode::ConstraintSigner);
            }
        }
        Err(_e) => return err!(ErrorCode::ConstraintSigner),
    }

    let _order = order.as_ref();
    let cpi_accounts = Transfer {
        from: ctx.accounts.customer_account_ata.to_account_info().clone(),
        to: ctx.accounts.stake_account_ata.to_account_info().clone(),
        authority: ctx.accounts.customer.to_account_info().clone(),
    };

    let cpi_program = token_program.to_account_info();
    token::transfer(
        CpiContext::new(cpi_program, cpi_accounts),
        _order.unwrap().settle_amount,
    )?;

    ctx.accounts.nonce_account.nonce = ix_args.nonce;

    emit!(NewSettleEvent {
        order_id: ix_args.order_id,
        token: ix_args.token,
        nonce: _order.unwrap().nonce,
        settle_amount: _order.unwrap().settle_amount,
        user_account: _order.unwrap().customer,
        side: _order.unwrap().side.clone(),
        order_ids: _order.unwrap().order_ids.clone(),
        sellet_amounts: _order.unwrap().sellet_amounts.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn buyer_settlee<'info>(
    ctx: Context<'_, '_, '_, 'info, SettleOrderPayment<'info>>,
    ix_args: SettleOrderPaymentArgs,
) -> Result<()> {
    msg!("buyer_settle");
    msg!("token {}", ix_args.token);
    let token_program = &ctx.accounts.token_program;
    let ix = instructions_sysvar_module::get_instruction_relative(
        -1,
        &ctx.accounts.instructions_sysvar,
    )?;
    if !validate_ed25519_ix(&ix) {
        return err!(ErrorCode::InstructionMissing);
    }

    if ctx.accounts.nonce_account.nonce >0 {
        return err!(ErrorCode::IdlAccountNotEmpty);
    }

    let pub_key: Pubkey = Pubkey::new(&ix.data[EXPECTED_PUBLIC_KEY_RANGE]);


    if pub_key != ctx.accounts.global_state.signer {
        return err!(ErrorCode::ConstraintSigner);
    }

    let order_data = &ix.data[112..];
    let order = deserialize_order(&order_data.to_vec());
    msg!("order {:?}", order);

    match &order {
        Ok(order_value) => {
            if order_value.order_id != ix_args.order_id || order_value.token != ix_args.token {
                return err!(ErrorCode::ConstraintSigner);
            }
        }
        Err(_e) => return err!(ErrorCode::ConstraintSigner),
    }

    let _order = order.as_ref();
    let cpi_accounts = Transfer {
        from: ctx.accounts.stake_account_ata.to_account_info().clone(),
        to: ctx.accounts.customer_account_ata.to_account_info().clone(),
        authority: ctx.accounts.global_state.to_account_info().clone(),
    };

    let bump = ctx.bumps.global_state;
    let seeds = &["global".as_bytes(), &[bump]];
    let signer = &[&seeds[..]];

    let cpi_program = token_program.to_account_info();
    token::transfer(
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
        _order.unwrap().settle_amount,
    )?;
    ctx.accounts.nonce_account.nonce = ix_args.nonce;
    emit!(NewSettleEvent {
        order_id: ix_args.order_id,
        token: ix_args.token,
        nonce: _order.unwrap().nonce,
        settle_amount: _order.unwrap().settle_amount,
        user_account: _order.unwrap().customer,
        side: _order.unwrap().side.clone(),
        order_ids: _order.unwrap().order_ids.clone(),
        sellet_amounts: _order.unwrap().sellet_amounts.clone(),
        timestamp: Clock::get()?.unix_timestamp
    });

    Ok(())
}

pub fn deserialize_order(order_payload: &Vec<u8>) -> Result<SettleFullOrder> {
    match SettleFullOrder::try_from_slice(order_payload) {
        Ok(order) => Ok(order),
        Err(_) => err!(ErrorCode::InstructionMissing),
    }
}
