# Solana Randomness Service Example

The following repo contains some example code to get started with the Solana
Randomness Service.

The Solana Randomness Service uses a Switchboard SGX enabled oracle to provide
randomness to any Solana program using a callback instruction.

Program ID: `RANDMo5gFnqnXJW5Z52KNmd24sAo95KAd5VbiCtq5Rh`

**NOTE:** This program ID is applicable for mainnet-beta and devnet.

## Request Lifecycle

1. User's program invokes the `simple_randomness_v1` instruction with a CPI call
   along with the number of randomness bytes, the custom callback instruction,
   and the priority fee config
   - Creates a `SimpleRandomnessV1Account` account
   - Sets the custom callback
   - Wraps funds into an escrow to reward the oracle for fulfilling the request
2. Off-chain SGX enabled oracle reads the request account
   - Generates random bytes inside of the enclave
   - Builds a txn with your callback and desired priority fees
   - Simulates the txn. If successful, relays the txn on-chain. If error, relays
     an error instruction with the error message which is viewable in an
     explorer.
3. Transaction relayed on-chain
   - Oracle rewarded for fulfilling request
   - Oracle invokes the users callback instruction
   - Request account is closed and the rent-exemption is returned to the
     original payer

## Anchor Program

```rust
use solana_randomness_service::SimpleRandomnessV1Account;
use solana_randomness_service::{
    program::SolanaRandomnessService, ID as SolanaRandomnessServiceID,
};
use switchboard_solana::prelude::*;
use switchboard_solana::utils::get_ixn_discriminator;

declare_id!("39hMZgeiesFXMRFt8svuKVsdCW5geiYueSRx7dxhXN4f");

#[program]
pub mod solana_randomness_consumer {
    use solana_randomness_service::TransactionOptions;

    use super::*;

    pub fn request_randomness(ctx: Context<RequestRandomness>) -> anchor_lang::prelude::Result<()> {
        // Call the randomness service and request a new value
        solana_randomness_service::cpi::simple_randomness_v1(
            CpiContext::new(
                ctx.accounts.randomness_service.to_account_info(),
                solana_randomness_service::cpi::accounts::SimpleRandomnessV1Request {
                    request: ctx.accounts.randomness_request.to_account_info(),
                    escrow: ctx.accounts.randomness_escrow.to_account_info(),
                    state: ctx.accounts.randomness_state.to_account_info(),
                    mint: ctx.accounts.randomness_mint.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    associated_token_program: ctx
                        .accounts
                        .associated_token_program
                        .to_account_info(),
                },
            ),
            8, // Request 8 bytes of randomness
            solana_randomness_service::Callback {
                program_id: ID,
                accounts: vec![
                    AccountMeta::new_readonly(ctx.accounts.randomness_state.key(), true).into(),
                    AccountMeta::new_readonly(ctx.accounts.randomness_request.key(), false).into(),
                ],
                ix_data: get_ixn_discriminator("consume_randomness").to_vec(), // TODO: hardcode this discriminator [190,217,49,162,99,26,73,234]
            },
            Some(TransactionOptions {
                compute_units: Some(1_000_000),
                compute_unit_price: Some(100),
            }),
        )?;

        // Here we can emit some event to index our requests

        Ok(())
    }

    pub fn consume_randomness(
        _ctx: Context<ConsumeRandomness>,
        result: Vec<u8>,
    ) -> anchor_lang::prelude::Result<()> {
        msg!("Randomness received: {:?}", result);
        Ok(())
    }
}

// The request_randomness macro breaks IDL generation. So we'll manually implement.
// #[request_randomness]
#[derive(Accounts)]
pub struct RequestRandomness<'info> {
    /// The Solana Randomness Service program.
    pub randomness_service: Program<'info, SolanaRandomnessService>,

    /// The account that will be created on-chain to hold the randomness request.
    /// Used by the off-chain oracle to pickup the request and fulfill it.
    /// CHECK: todo
    #[account(
        mut,
        signer,
        owner = system_program.key(),
        constraint = randomness_request.data_len() == 0 && randomness_request.lamports() == 0,
    )]
    pub randomness_request: AccountInfo<'info>,

    /// The TokenAccount that will store the funds for the randomness request.
    /// CHECK: todo
    #[account(
        mut,
        owner = system_program.key(),
        constraint = randomness_escrow.data_len() == 0 && randomness_escrow.lamports() == 0,
    )]
    pub randomness_escrow: AccountInfo<'info>,

    /// The randomness service's state account. Responsible for storing the
    /// reward escrow and the cost per random byte.
    #[account(
        seeds = [b"STATE"],
        bump = randomness_state.bump,
        seeds::program = randomness_service.key(),
    )]
    pub randomness_state: Box<Account<'info, solana_randomness_service::State>>,

    /// The token mint to use for paying for randomness requests.
    #[account(address = NativeMint::ID)]
    pub randomness_mint: Account<'info, Mint>,

    /// The account that will pay for the randomness request.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The Solana System program. Used to allocate space on-chain for the randomness_request account.
    pub system_program: Program<'info, System>,

    /// The Solana Token program. Used to transfer funds to the randomness escrow.
    pub token_program: Program<'info, Token>,

    /// The Solana Associated Token program. Used to create the TokenAccount for the randomness escrow.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    /// We need to make sure the randomness service signed this requests so it can only be invoked by a PDA and not a user.
    #[account(
        signer,
        seeds = [b"STATE"],
        seeds::program = SolanaRandomnessServiceID,
        bump = randomness_state.bump,
    )]
    pub randomness_state: Box<Account<'info, solana_randomness_service::State>>,

    pub request: Box<Account<'info, SimpleRandomnessV1Account>>,
}

```

## Typescript Client

```typescript
import type { SolanaRandomnessConsumer } from "../target/types/solana_randomness_consumer";

import type { Program } from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import { RandomnessService } from "@switchboard-xyz/solana-randomness-service";
import assert from "assert";

describe("Solana Randomness Service Example", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .SolanaRandomnessConsumer as Program<SolanaRandomnessConsumer>;

  let randomnessService: RandomnessService;

  before(async () => {
    randomnessService = await RandomnessService.fromProvider(provider);
  });

  it("requests randomness", async () => {
    const requestKeypair = anchor.web3.Keypair.generate();

    // Start watching for the settled event before triggering the request
    const settledRandomnessEventPromise = randomnessService.awaitSettledEvent(
      requestKeypair.publicKey
    );

    // your program makes a CPI request to the RandomnessService
    const signature = await program.methods
      .requestRandomness()
      .accounts({
        randomnessService: randomnessService.programId,
        randomnessRequest: requestKeypair.publicKey,
        randomnessEscrow: anchor.utils.token.associatedAddress({
          mint: randomnessService.accounts.mint,
          owner: requestKeypair.publicKey,
        }),
        randomnessState: randomnessService.accounts.state,
        randomnessMint: randomnessService.accounts.mint,
        payer: provider.wallet.publicKey,
      })
      .signers([requestKeypair])
      .rpc();

    // Await the response from the Switchboard Service
    const [settledRandomnessEvent, settledSlot] =
      await settledRandomnessEventPromise;

    assert.equal(
      settledRandomnessEvent.user.toBase58(),
      provider.wallet.publicKey.toBase58(),
      "User should be the same as the provider wallet"
    );
    assert.equal(
      settledRandomnessEvent.request.toBase58(),
      requestKeypair.publicKey.toBase58(),
      "Request should be the same as the provided request keypair"
    );
    assert.equal(
      settledRandomnessEvent.isSuccess,
      true,
      "Request did not complete successfully"
    );
  });
});
```
