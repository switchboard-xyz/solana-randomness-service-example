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
    console.log(`Request: ${requestKeypair.publicKey.toBase58()}`);

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
    console.log(`[TX] requestRandomness: ${signature}`);

    // Await the response from the Switchboard Service
    const [settledRandomnessEvent, settledSlot] =
      await settledRandomnessEventPromise;

    console.log(
      `[EVENT] SimpleRandomnessV1SettledEvent\n${JSON.stringify(
        {
          ...settledRandomnessEvent,

          // why is anchor.BN so annoying with hex strings?
          requestSlot: settledRandomnessEvent.requestSlot.toNumber(),
          settledSlot: settledRandomnessEvent.settledSlot.toNumber(),
          randomness: `[${new Uint8Array(settledRandomnessEvent.randomness)}]`,
        },
        undefined,
        2
      )}`
    );

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

    const latency = settledRandomnessEvent.settledSlot
      .sub(settledRandomnessEvent.requestSlot)
      .toNumber();
    console.log(
      `\nRandomness: [${new Uint8Array(
        settledRandomnessEvent.randomness
      )}]\nRequest completed in ${latency} slots!\n`
    );
  });
});
