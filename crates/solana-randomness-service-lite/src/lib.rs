#![cfg_attr(doc_cfg, feature(doc_cfg))]

//!  The Solana Randomness Service uses a Switchboard SGX enabled oracle to provide randomness to any Solana program using a callback instruction.
//!
//!  **Program ID:** `RANDMo5gFnqnXJW5Z52KNmd24sAo95KAd5VbiCtq5Rh`
//!
//! See the crate `solana-randomness-service` for the full CPI interface.
//!
//!  # Example Program
//!
//! ```
//! use anchor_lang::prelude::*;
//! use solana_randomness_service_lite::{SimpleRandomnessV1Request, ID as SolanaRandomnessServiceID};
//!
//! #[program]
//! pub mod solana_randomness_consumer {
//!     use super::*;
//!
//!     pub fn request_randomness(ctx: Context<RequestRandomness>) -> anchor_lang::prelude::Result<()> {
//!         msg!("Requesting randomness...");
//!
//!         let request = SimpleRandomnessV1Request {
//!             request: ctx.accounts.randomness_request.to_account_info(),
//!             escrow: ctx.accounts.randomness_escrow.to_account_info(),
//!             state: ctx.accounts.randomness_state.to_account_info(),
//!             mint: ctx.accounts.randomness_mint.to_account_info(),
//!             payer: ctx.accounts.payer.to_account_info(),
//!             system_program: ctx.accounts.system_program.to_account_info(),
//!             token_program: ctx.accounts.token_program.to_account_info(),
//!             associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
//!         };
//!         request.invoke(
//!             ctx.accounts.randomness_service.to_account_info(),
//!             8, // Request 8 bytes of randomness
//!             &solana_randomness_service_lite::Callback::new(
//!                 ID,
//!                 vec![
//!                     AccountMeta::new_readonly(ctx.accounts.randomness_state.key(), true).into(),
//!                     AccountMeta::new_readonly(ctx.accounts.randomness_request.key(), false).into(),
//!                 ],
//!                 [190, 217, 49, 162, 99, 26, 73, 234].to_vec(), // Our callback ixn discriminator. The oracle will append the randomness bytes to the end
//!             ),
//!             &Some(solana_randomness_service_lite::TransactionOptions {
//!                 compute_units: Some(1_000_000),
//!                 compute_unit_price: Some(100),
//!             }),
//!         )?;
//!
//!         // Here we can emit some event to index our requests
//!
//!         Ok(())
//!     }
//! }
//!
//! #[derive(Accounts)]
//! pub struct RequestRandomness<'info> {
//!     /// CHECK: manually check programID and executable status
//!     #[account(
//!         constraint = randomness_service.key() == SolanaRandomnessServiceID,
//!         constraint = randomness_service.executable,
//!     )]
//!     pub randomness_service: AccountInfo<'info>,
//!
//!     /// The account that will be created on-chain to hold the randomness request.
//!     /// Used by the off-chain oracle to pickup the request and fulfill it.
//!     /// CHECK: todo
//!     #[account(
//!         mut,
//!         signer,
//!         owner = system_program.key(),
//!         constraint = randomness_request.data_len() == 0 && randomness_request.lamports() == 0,
//!     )]
//!     pub randomness_request: AccountInfo<'info>,
//!
//!     /// The TokenAccount that will store the funds for the randomness request.
//!     /// CHECK: todo
//!     #[account(
//!         mut,
//!         owner = system_program.key(),
//!         constraint = randomness_escrow.data_len() == 0 && randomness_escrow.lamports() == 0,
//!     )]
//!     pub randomness_escrow: AccountInfo<'info>,
//!
//!     /// The randomness service's state account. Responsible for storing the
//!     /// reward escrow and the cost per random byte.
//!     #[account(
//!         seeds = [b"STATE"],
//!         bump = randomness_state.bump,
//!         seeds::program = randomness_service.key(),
//!     )]
//!     pub randomness_state: Box<Account<'info, solana_randomness_service::State>>,
//!
//!     /// The token mint to use for paying for randomness requests.
//!     #[account(address = NativeMint::ID)]
//!     pub randomness_mint: Account<'info, Mint>,
//!
//!     /// The account that will pay for the randomness request.
//!     #[account(mut)]
//!     pub payer: Signer<'info>,
//!
//!     /// The Solana System program. Used to allocate space on-chain for the randomness_request account.
//!     pub system_program: Program<'info, System>,
//!
//!     /// The Solana Token program. Used to transfer funds to the randomness escrow.
//!     pub token_program: Program<'info, Token>,
//!
//!     /// The Solana Associated Token program. Used to create the TokenAccount for the randomness escrow.
//!     pub associated_token_program: Program<'info, AssociatedToken>,
//! }
//! ```
use borsh::{BorshDeserialize, BorshSerialize};
pub use solana_program::account_info::AccountInfo;
pub use solana_program::instruction::AccountMeta;
pub use solana_program::program_error::ProgramError;
pub use solana_program::pubkey::Pubkey;
use solana_program::{declare_id, instruction::Instruction};
use solana_program::{program::invoke, program::invoke_signed, pubkey};

pub mod types;
pub use types::*;

declare_id!("RANDMo5gFnqnXJW5Z52KNmd24sAo95KAd5VbiCtq5Rh");

pub const SWITCHBOARD_PROGRAM_ID: Pubkey = pubkey!("sbattyXrzedoNATfc4L31wC9Mhxsi1BmFhTiN8gDshx");

pub const RANDOMNESS_SERVICE_STATE: Pubkey =
    pubkey!("889J3BcnDDBMA651BoZNnuKrhQtXkLRzDuKhnJkWUfKA");
pub const RANDOMNESS_SERVICE_REWARD_WALLET: Pubkey =
    pubkey!("3X7Jy3dc7eRSP9ECWvxiiQWG8gfwmYq1zENfWpzygt6D");
pub const RANDOMNESS_SERVICE_REWARD_MINT: Pubkey =
    pubkey!("So11111111111111111111111111111111111111112");

pub const MAINNET_SWITCHBOARD_FUNCTION: Pubkey =
    pubkey!("yxvdQ9D6eovAQqacSyAL9vYhXXtdtnmgCABfaz8cg2W");
pub const MAINNET_SWITCHBOARD_SERVICE: Pubkey =
    pubkey!("3gGs95XHv47gY6aUvSPhQmVrWYt3M6Lz3nxE9eMS3bot");

pub const DEVNET_SWITCHBOARD_FUNCTION: Pubkey =
    pubkey!("AHV7ygefHZQ5extiZ4GbseGANg3AwBWgSUfnUktTrxjd");
pub const DEVNET_SWITCHBOARD_SERVICE: Pubkey =
    pubkey!("2fpdEbugwThMjRQ728Ne4zwGsrjFcCtmYDnwGtzScfnL");

///
pub struct SimpleRandomnessV1Request<'info> {
    pub request: AccountInfo<'info>,
    pub escrow: AccountInfo<'info>,
    pub state: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
}

impl<'info> SimpleRandomnessV1Request<'info> {
    pub const DISCRIMINATOR: [u8; 8] = [179, 249, 152, 75, 164, 218, 230, 36];

    pub fn discriminator() -> [u8; 8] {
        Self::DISCRIMINATOR
    }

    pub fn get_instruction(
        &self,
        program_id: Pubkey,
        num_bytes: u8,
        callback: &Callback,
        options: &Option<TransactionOptions>,
    ) -> Result<Instruction, ProgramError> {
        let accounts = self.to_account_metas();

        let mut data: Vec<u8> = Self::discriminator().to_vec();
        data.push(num_bytes);
        data.append(&mut callback.to_vec()?);
        data.append(&mut TransactionOptions::to_opt_vec(options)?);

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }

    pub fn invoke(
        &self,
        program: AccountInfo<'info>,
        num_bytes: u8,
        callback: &Callback,
        options: &Option<TransactionOptions>,
    ) -> Result<(), solana_program::program_error::ProgramError> {
        let instruction = self.get_instruction(*program.key, num_bytes, callback, options)?;
        let account_infos = self.to_account_infos();

        invoke(&instruction, &account_infos[..])
    }

    pub fn invoke_signed(
        &self,
        program: AccountInfo<'info>,
        num_bytes: u8,
        callback: &Callback,
        options: &Option<TransactionOptions>,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<(), solana_program::program_error::ProgramError> {
        let instruction = self.get_instruction(*program.key, num_bytes, callback, options)?;
        let account_infos = self.to_account_infos();

        invoke_signed(&instruction, &account_infos[..], signer_seeds)
    }

    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![
            self.request.clone(),
            self.escrow.clone(),
            self.state.clone(),
            self.mint.clone(),
            self.payer.clone(),
            self.system_program.clone(),
            self.token_program.clone(),
            self.associated_token_program.clone(),
        ]
    }

    fn to_account_metas(&self) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new(*self.request.key, true),
            AccountMeta::new(*self.escrow.key, false),
            AccountMeta::new_readonly(*self.state.key, false),
            AccountMeta::new_readonly(*self.mint.key, false),
            AccountMeta::new(*self.payer.key, true),
            AccountMeta::new_readonly(*self.system_program.key, false),
            AccountMeta::new_readonly(*self.token_program.key, false),
            AccountMeta::new_readonly(*self.associated_token_program.key, false),
        ]
    }
}

#[derive(Clone, Default, BorshDeserialize, BorshSerialize)]
pub struct SimpleRandomnessV1Account {
    pub is_completed: u8,
    pub num_bytes: u8,
    pub user: Pubkey,
    pub escrow: Pubkey,
    pub request_slot: u64,
    pub callback: Callback,
    pub compute_units: u32,
    pub priority_fee_micro_lamports: u64,
    pub error_message: String,
}
impl SimpleRandomnessV1Account {
    pub const DISCRIMINATOR: [u8; 8] = [45, 236, 206, 109, 194, 21, 241, 154];

    pub fn discriminator() -> [u8; 8] {
        Self::DISCRIMINATOR
    }

    pub fn owner() -> Pubkey {
        ID
    }

    pub fn try_deserialize(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        if buf.len() < Self::DISCRIMINATOR.len() {
            return Err(ProgramError::InvalidAccountData);
        }
        let given_disc = &buf[..8];
        if Self::DISCRIMINATOR != given_disc {
            return Err(ProgramError::InvalidAccountData);
        }
        Self::try_deserialize_unchecked(buf)
    }

    pub fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        let mut data: &[u8] = &buf[8..];
        Self::deserialize(&mut data).map_err(|_| ProgramError::InvalidAccountData)
    }
}

#[derive(Clone, Default, BorshDeserialize, BorshSerialize)]
pub struct State {
    pub is_completed: u8,
    pub num_bytes: u8,
    pub user: Pubkey,
    pub escrow: Pubkey,
    pub request_slot: u64,
    pub callback: Callback,
    pub compute_units: u32,
    pub priority_fee_micro_lamports: u64,
    pub error_message: String,
}
impl State {
    pub const DISCRIMINATOR: [u8; 8] = [216, 146, 107, 94, 104, 75, 182, 177];

    pub fn discriminator() -> [u8; 8] {
        Self::DISCRIMINATOR
    }

    pub fn owner() -> Pubkey {
        ID
    }

    pub fn try_deserialize(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        if buf.len() < Self::DISCRIMINATOR.len() {
            return Err(ProgramError::InvalidAccountData);
        }
        let given_disc = &buf[..8];
        if Self::DISCRIMINATOR != given_disc {
            return Err(ProgramError::InvalidAccountData);
        }
        Self::try_deserialize_unchecked(buf)
    }

    pub fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        let mut data: &[u8] = &buf[8..];
        Self::deserialize(&mut data).map_err(|_| ProgramError::InvalidAccountData)
    }
}
