use crate::constants::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock, program_option::COption, sysvar};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::convert::Into;
use std::convert::TryInto;

declare_id!("2zu8SFickvWcfMWLVGAWi8nmXbCYpJ53rfcqpN2sk2Ci");

mod contants {
    pub const DEPOSIT_REQUIREMENT: u64 = 10_000_000_000_000;
    pub const DURATION: u64 = 1;
}

#[program]
pub mod goblin_stake {
    use super::*;
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
      if amount != constants.DEPOSIT_REQUIREMENT {
        return Err(ErrCode: InvalidAmount.into());
      }
      let pool = &mut ctx.accounts.pool;

      pool.stakes.push(StakeInfo {
        last_update_time = clock.unix_timestamp.try_into().unwrap(),
        nft: ctx.to_account_info().key,
        owner: ctx.accounts.owner.to_account_info().key
      })
      // Transfer tokens into the stake vault.
      {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_from_account.to_account_info(),
                to: ctx.accounts.staking_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(), //todo use user account as signer
            },
        );
        token::transfer(cpi_ctx, amount)?;
      }

      // Transfer nft into the stake vault.
      {
          cpi_ctx = CpiContext::new(
              ctx.accounts.nft_program.to_account_info(),
              token::Transfer {
                  from: ctx.accounts.nft_from_account.to_account_info(),
                  to: ctx.accounts.staking_vault.to_account_info(),
                  authority: ctx.accounts.owner.to_account_info(), //todo use user account as signer
              },
          );
          token::transfer(cpi_ctx, 1)?;
      }
      Ok(())
    }

    pub fn unstake(ctx: Context<Stake>, stake_id: u128) -> Result<()> {
      let pool = &mut ctx.accounts.pool;
      let owner = ctx.accounts.owner;
      if pool.stakes[stake_id as usize].owner == owner.to_account_info().key {
        return Err(ErrorCode::NoNFTOwner.into());
      }
      let amount = pool.stakes[stake_id as usize].amount;
      pool.stakes.remove(stake_id);
      // Transfer tokens and nft the pool vault to user vault
      {
        let seeds = &[
            ctx.accounts.pool.to_account_info().key.as_ref(),
            &[ctx.accounts.pool.nonce],
        ];
        let pool_signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.staking_vault.to_account_info(),
                to: ctx.accounts.token_from_account.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(cpi_ctx, amount)?;
        cpi_ctx = CpiContext::new_with_signer(
          ctx.accounts.token_program.to_account_info(),
          token::Transfer {
              from: ctx.accounts.staking_vault.to_account_info(),
              to: ctx.accounts.nft_from_account.to_account_info(),
              authority: ctx.accounts.pool_signer.to_account_info(),
          },
          pool_signer,
      );
      token::transfer(cpi_ctx, 1)?;
      }

      Ok(())
    }

    pub fn claim_nft(
      ctx: Context<ClaimNFT>,
      stake_id: u128,
    ) -> Result<()> {
      let pool = &mut ctx.accounts.pool;
      let owner = ctx.accounts.owner;
      if pool.stakes[stake_id as usize].owner == owner.to_account_info().key {
        return Err(ErrorCode::NoNFTOwner.into());
      }
      if pool.stakes[stake_id as usize].last_update_time + constants.DURATION < clock.unix_timestamp.try_into().unwrap() {
        return Err(ErrorCode::InvalidTime.into());
      }

      let seeds = &[
        ctx.accounts.pool.to_account_info().key.as_ref(),
        &[ctx.accounts.pool.nonce],
      ];

      let pool_signer = &[&seeds[..]];
      let nft_id = pool.nfts.iter().positiion(|&nft| nft == nft_program.to_account_info().key);

      let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.receive_account.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        },
      );
      token::transfer(cpi_ctx, 1)?;
      // update rank
      if (pool.nfts[nft_id].rank < 8) {
        pool.nfts[nft_id].rank += 1;
      }
      Ok(())
    }

    pub fn add_nft_for_sale(
        ctx: Context<ADDNFT>,
        price: u128,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let cpi_ctx = CpiContext::new(
            ctx.accounts.nft_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.staking_vault.to_account_info(),
                authority: ctx.accounts.funder.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, 1)?;

        pool.nftsForSale.push(NFT {
            nft_mint: ctx.accounts.staking_mint.key(),
            nft_vault: ctx.accounts.staking_vault.key(),
            price: price,
            redeemed: false
        });
        Ok(())
    }

    pub fn buy_nft(
        ctx: Context<BuyNFT>,
        nft_id: u8,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        if pool.nfts[nft_id as usize].redeemed == true {
            return Err(ErrorCode::NFTRedeemed.into());
        }
        let sees = &[
            ctx.accounts.pool.to_account_info().key.as_ref(),
            &[ctx.accounts.pool.nonce],
        ];
        let pool_signer = &[&seeds[..]];
        let amount = pool.nftsForSale[nft_id as usize].price;
        let fee = amount * 5 / 100;
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_from_account.to_account_info(),
                to: ctx.accounts.staking_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(), //todo use user account as signer
            },
        );
        token::transfer(cpi_ctx, amount - fee)?;
        cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_from_account.to_account_info(),
                to: ctx.pool.devAddr,
                authority: ctx.accounts.owner.to_account_info(), //todo use user account as signer
            },
        );
        token::transfer(cpi_ctx, fee)?;
        cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.staking_vault.to_account_info(),
                to: ctx.accounts.receive_account.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, 1)?;
    }
}


#[derive(Accounts)]
pub struct Stake<'info> {
    owner: Signer<'info>,
    #[account(mut)]
    token_from_account: Box<'info, TokenAccount>>,
    nft_from_account: Box<'info, TokenAccount>>,
    // Misc
    token_program: Program<'info, Token>,
    nft_program: Program<'info, Token>,
    pool: Box<Account<'info, Pool>>,
    staking_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = staking_vault.mint == staking_mint.key(),
        constraint = staking_vault.owner == pool_signer.key(),
        constraint = staking_vault.close_authority == COption::None,
    )]
    staking_vault: Box<Account<'info, TokenAccount>>,
    pool_signer: UncheckedAccount<'info>,
}


#[derive(Accounts)]
pub struct ClaimNFT<'info> {
  owner: Signer<'info>,
  nft_program: Program<'info, Token>,
  pool: Box<Account<'info, Pool>>,
  staking_vault: Box<Account<'info, TokenAccount>>,
  receive_account: Box<Account<'info, TokenAccount>>,
  pool_signer: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct AddNFT<'info> {
    #[account(mut)]
    pool: Box<Account<'info, Pool>>,
    staking_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = staking_vault.mint == staking_mint.key(),
        constraint = staking_vault.owner == pool_signer.key(),
        constraint = staking_vault.close_authority == COption::None,
    )]
    staking_vault: Box<Account<'info, TokenAccount>>,
    funder: Signer<'info>,
    #[account(mut)]
    from: Box<Account<'info, TokenAccount>>,
    nft_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BuyNFT<'info> {
    #[account(mut)]
    pool: Box<Account<'info, Pool>>,
    staking_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = staking_vault.mint == staking_mint.key(),
        constraint = staking_vault.owner == pool_signer.key(),
        constraint = staking_vault.close_authority == COption::None,
    )]
    staking_vault: Box<Account<'info, TokenAccount>>,
    receive_account: Box<Account<'info. TokenAccount>>,
    funder: Signer<'info>,
    #[account(mut)]
    from: Box<Account<'info, TokenAccount>>,
    nft_program: Program<'info, Token>,
    token_program: Program<'info, Token>,
}

#[account]
pub struct Pool {
    pub stakes: Vec<StakeInfo>,
    pub nfts: Vec<NFTInfo>,
    pub nonce: u8,
    pub devAddr: Pubkey,
    pub nftsForSale: Vec<NFT>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct StakeInfo {
    pub nft: Pubkey,
    pub last_update_time: u128,
    pub owner: Pubkey,
    pub token_amount: u128,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct NFTInfo {
    pub nft: Pubkey,
    pub rank: u8,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct NFT {
    pub nft_mint: Pubkey,
    pub nft_vault: Pubkey,
    pub price: u128,
    pub redeemed: bool,
}


#[error]
pub enum ErrorCode {
  #[msg("Amount is not amount to stake.")]
  InvalidAmount,
  #[msg("You are not nft owner.")]
  NoNFTOwner,
  #[msg("You can not claim yet.")]
  InvalidTime,
  #[msg("NFT was already sold out")]
  NFTRedeemed,
}