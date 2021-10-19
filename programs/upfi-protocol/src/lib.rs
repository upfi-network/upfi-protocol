use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use spl_token::instruction::AuthorityType;
use std::convert::TryInto;

declare_id!("Cyq9SPbsdxKTKSLfaMKQN5rDfQraFmFzy1nCkf1XKfCe");

pub mod constants {
    pub const USDC_TOKEN_MINT_PUBKEY: &str = "F4AyR1rrzg1h4hgFsM9uckupsPxJ3s95bL4Z71twSBdS";
    pub const UPFI_TOKEN_MINT_PUBKEY: &str = "A8sACLUssMGSFNmcm5CMmvKEFYX6tGTGERsuD8SPcrjN";
    pub const UPS_TOKEN_MINT_PUBKEY: &str = "8BSsK1bLBzYQuw1cChwuDpN5prq22xsG6Pv6cWv1UcQP";
    pub const LP_TOKEN_MINT_PUBKEY: &str = "DY98vGpLfSDs2hYLqiajYTLmeatk4kFyFv9GakQaKxna";
}

// pub mod constants {
//     pub const USDC_TOKEN_MINT_PUBKEY: &str = "3hxtbhQjPbAJ3cjgwoTtJvjfcwKypW1DuWmNjYZHyUcS";
//     pub const UPFI_TOKEN_MINT_PUBKEY: &str = "EgRhq51CVc6VtSnH8hBqXCYz9UV56WYUYwVtA8HZfAtj";
//     pub const UPS_TOKEN_MINT_PUBKEY: &str = "CYKweyWXQ8qqTZ5S72jFYdSBRLEEgdKKuZXqYT8yFbtW";
// }

pub fn amount_mint(mut usdc_amount: u64, mut ups_amount: u64) -> (u64, u64, u64) {
    let ups_price: u64 = 30_000;
    // check rate usdc and ups
    let to_ups: u64 = usdc_amount
        .checked_mul(25)
        .unwrap()
        .checked_mul(1_000_000)
        .unwrap()
        .checked_div(ups_price)
        .unwrap()
        .checked_div(9_975)
        .unwrap()
        .try_into()
        .unwrap();

    let check: bool = to_ups > ups_amount;

    if check {
        usdc_amount = ups_amount
            .checked_mul(9_975)
            .unwrap()
            .checked_mul(ups_price)
            .unwrap()
            .checked_div(25)
            .unwrap()
            .checked_div(1_000_000)
            .unwrap()
            .try_into()
            .unwrap();
    } else {
        ups_amount = to_ups;
    }
    let mut upfi: u64 = usdc_amount
        .checked_mul(10000)
        .unwrap()
        .checked_div(9_975)
        .unwrap()
        .try_into()
        .unwrap();
    let fee: u64 = upfi
        .checked_mul(20)
        .unwrap()
        .checked_div(10_000)
        .unwrap()
        .try_into()
        .unwrap();

    upfi = upfi.checked_sub(fee).unwrap().try_into().unwrap();
    return (upfi, usdc_amount, ups_amount);
}

pub fn amount_redeem(amount_upfi: u64) -> (u64, u64) {
    let fee: u64 = amount_upfi
        .checked_mul(30)
        .unwrap()
        .checked_div(10000)
        .unwrap()
        .try_into()
        .unwrap();

    let upfi: u64 = amount_upfi.checked_sub(fee).unwrap().try_into().unwrap();

    let _usdc = upfi
        .checked_mul(9_975)
        .unwrap()
        .checked_div(10_000)
        .unwrap()
        .try_into()
        .unwrap();

    let _ups = upfi
        .checked_mul(25)
        .unwrap()
        .checked_mul(1_000_000)
        .unwrap()
        .checked_div(10_000)
        .unwrap()
        .checked_div(30_000)
        .unwrap()
        .try_into()
        .unwrap();
    return (_usdc, _ups);
}

#[program]
pub mod upfi_mint_redeem {
    use super::*;
    pub fn initialize(_ctx: Context<Initialize>, _nonce: u8) -> ProgramResult {
        Ok(())
    }

    pub fn mint(
        _ctx: Context<MintToken>,
        nonce: u8,
        amount_usdc: u64,
        amount_ups: u64,
    ) -> ProgramResult {
        let usdc_pubkey = _ctx.accounts.usdc_token.key();
        // will add arbitrary salt for more secure
        let seeds_usdc = &[usdc_pubkey.as_ref(), &[nonce]];
        let signer = [&seeds_usdc[..]];

        let (_upfi, _usdc, _ups) = amount_mint(amount_usdc, amount_ups);

        // mint upfi to user
        let cpi_ctx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: _ctx.accounts.upfi_token.to_account_info(),
                to: _ctx.accounts.upfi_token_to.to_account_info(),
                authority: _ctx.accounts.token_vault.to_account_info(),
            },
            &signer,
        );
        token::mint_to(cpi_ctx, _upfi)?;

        // // send usdc to vault
        let cpi_ctx_transfer_usdc = CpiContext::new(
            _ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: _ctx.accounts.usdc_token_from.to_account_info(),
                to: _ctx.accounts.token_vault.to_account_info(),
                authority: _ctx.accounts.caller_signer.to_account_info(),
            },
        );
        token::transfer(cpi_ctx_transfer_usdc, _usdc)?;

        // burn ups
        let cpi_ctx_burn_ups = CpiContext::new(
            _ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: _ctx.accounts.ups_token.to_account_info(),
                to: _ctx.accounts.ups_token_from.to_account_info(),
                authority: _ctx.accounts.caller_signer.to_account_info(),
            },
        );
        token::burn(cpi_ctx_burn_ups, amount_ups)?;

        (&mut _ctx.accounts.upfi_token_to).reload()?;
        (&mut _ctx.accounts.usdc_token_from).reload()?;
        (&mut _ctx.accounts.ups_token_from).reload()?;

        Ok(())
    }

    pub fn redeem(_ctx: Context<RedeemToken>, nonce: u8, amount_upfi: u64) -> ProgramResult {
        let usdc_pubkey = _ctx.accounts.usdc_token.key();
        // will add arbitrary salt for more secure
        let seeds_usdc = &[usdc_pubkey.as_ref(), &[nonce]];
        let signer = [&seeds_usdc[..]];

        let (_usdc, _ups) = amount_redeem(amount_upfi);

        // let user_address = ctx.accounts.token_program

        // burn upfi
        let cpi_ctx_burn_upfi = CpiContext::new(
            _ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: _ctx.accounts.upfi_token.to_account_info(),
                to: _ctx.accounts.upfi_token_from.to_account_info(),
                authority: _ctx.accounts.caller_signer.to_account_info(),
            },
        );
        token::burn(cpi_ctx_burn_upfi, amount_upfi)?;

        // // mint ups to user
        let cpi_ctx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: _ctx.accounts.ups_token.to_account_info(),
                to: _ctx.accounts.ups_token_to.to_account_info(),
                authority: _ctx.accounts.token_vault.to_account_info(),
            },
            &signer,
        );
        token::mint_to(cpi_ctx, _ups)?;

        // // send usdc to user
        let cpi_ctx_transfer_usdc = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: _ctx.accounts.token_vault.to_account_info(),
                to: _ctx.accounts.usdc_token_to.to_account_info(),
                authority: _ctx.accounts.token_vault.to_account_info(),
            },
            &signer,
        );
        token::transfer(cpi_ctx_transfer_usdc, _usdc)?;

        (&mut _ctx.accounts.upfi_token_from).reload()?;
        (&mut _ctx.accounts.ups_token_to).reload()?;
        (&mut _ctx.accounts.usdc_token_to).reload()?;
        (&mut _ctx.accounts.token_vault).reload()?;

        Ok(())
    }

    pub fn reclaim_mint_upfi_authority(
        ctx: Context<ReclaimMintUpfiAuthority>,
        nonce: u8,
    ) -> ProgramResult {
        let usdc_token_key = ctx.accounts.usdc_token.key();
        // will add arbitrary salt for more secure
        let seeds = &[usdc_token_key.as_ref(), &[nonce]];
        let signer = [&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::SetAuthority {
                current_authority: ctx.accounts.token_vault.to_account_info(),
                account_or_mint: ctx.accounts.upfi_token.to_account_info(),
            },
            &signer,
        );
        token::set_authority(
            cpi_ctx,
            AuthorityType::MintTokens,
            Some(ctx.accounts.usdc_token.mint_authority.unwrap()),
        )?;
        Ok(())
    }

    pub fn reclaim_mint_ups_authority(
        ctx: Context<ReclaimMintUpsAuthority>,
        nonce: u8,
    ) -> ProgramResult {
        let usdc_token_key = ctx.accounts.usdc_token.key();
        // will add arbitrary salt for more secure
        let seeds = &[usdc_token_key.as_ref(), &[nonce]];
        let signer = [&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::SetAuthority {
                current_authority: ctx.accounts.token_vault.to_account_info(),
                account_or_mint: ctx.accounts.ups_token.to_account_info(),
            },
            &signer,
        );
        token::set_authority(
            cpi_ctx,
            AuthorityType::MintTokens,
            Some(ctx.accounts.authority.key()),
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_nonce: u8)]
pub struct Initialize<'info> {
    #[account(
        address = constants::USDC_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub usdc_token: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = initializer,
        token::mint = usdc_token,
        token::authority = token_vault, //the PDA address is both the vault account and the authority (and event the mint authority)
        seeds = [ constants::USDC_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap().as_ref() ],
        bump = _nonce,
    )]
    ///the not-yet-created, derived token vault pubkey
    pub token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    ///pays rent on the initializing accounts
    pub initializer: Signer<'info>,

    ///used by anchor for init of the token
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct MintToken<'info> {
    #[account(
        address = constants::UPS_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    #[account(mut)]
    pub ups_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = constants::UPFI_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub upfi_token: Box<Account<'info, Mint>>,

    #[account(
        address = constants::USDC_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub usdc_token: Box<Account<'info, Mint>>,

    //the token account uscd - where program will transfer
    #[account(mut)]
    pub usdc_token_from: Box<Account<'info, TokenAccount>>,

    //the token account ups - where program will transfer
    #[account(mut)]
    pub ups_token_from: Box<Account<'info, TokenAccount>>,

    //the authority allowed to transfer from ups, usdc
    // #[account(mut)]
    // pub authority_tranfer_token: Box<Account<'info, TokenAccount>>,
    pub caller_signer: Signer<'info>,

    // pubkey allowed mint upfi and receive usdc
    #[account(
        mut,
        seeds = [ usdc_token.key().as_ref() ],
        bump = nonce,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    // The token account to send upfi
    #[account(mut)]
    pub upfi_token_to: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct RedeemToken<'info> {
    #[account(
        address = constants::UPS_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    #[account(mut)]
    pub ups_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = constants::UPFI_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub upfi_token: Box<Account<'info, Mint>>,

    #[account(
        address = constants::USDC_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub usdc_token: Box<Account<'info, Mint>>,

    //the token account upfi - where program will transfer
    #[account(mut)]
    pub upfi_token_from: Box<Account<'info, TokenAccount>>,

    //the authority allowed to transfer from upfi
    pub caller_signer: Signer<'info>,

    // pubkey allowed mint upfi and receive usdc
    #[account(
        mut,
        seeds = [ usdc_token.key().as_ref() ],
        bump = nonce,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    // The token account to send upfi
    #[account(mut)]
    pub ups_token_to: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub usdc_token_to: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct ReclaimMintUpfiAuthority<'info> {
    #[account(
        address = constants::USDC_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub usdc_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = constants::UPFI_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub upfi_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [ usdc_token.key().as_ref() ],
        bump = nonce,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        //only STEP's token authority can sign for this action
        address = usdc_token.mint_authority.unwrap(),
    )]
    ///the mint authority of the step token
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct ReclaimMintUpsAuthority<'info> {
    #[account(
        address = constants::USDC_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub usdc_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = constants::UPS_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub ups_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [ usdc_token.key().as_ref() ],
        bump = nonce,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        // address = usdc_token.mint_authority.unwrap(),
    )]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
