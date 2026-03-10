import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;
  
  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;
  let makerTokenA: anchor.web3.PublicKey;
  let makerTokenB: anchor.web3.PublicKey;
  let takerTokenA: anchor.web3.PublicKey;
  let takerTokenB: anchor.web3.PublicKey;
  let escrowPda: anchor.web3.PublicKey;
  let vault: anchor.web3.Keypair;
  
  const maker = anchor.web3.Keypair.generate();
  const taker = anchor.web3.Keypair.generate();
  
  const AMOUNT_A = 1000000; // 1 token A
  const AMOUNT_B = 500000;  // 0.5 token B

  before(async () => {
    // Airdrop SOL to maker and taker
    await provider.connection.requestAirdrop(
      maker.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      taker.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );

    // Wait for confirmation
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Create mints
    mintA = await createMint(
      provider.connection,
      maker,
      maker.publicKey,
      null,
      6
    );

    mintB = await createMint(
      provider.connection,
      maker,
      maker.publicKey,
      null,
      6
    );

    // Create token accounts
    makerTokenA = await createAccount(
      provider.connection,
      maker,
      mintA,
      maker.publicKey
    );

    makerTokenB = await createAccount(
      provider.connection,
      maker,
      mintB,
      maker.publicKey
    );

    takerTokenA = await createAccount(
      provider.connection,
      taker,
      mintA,
      taker.publicKey
    );

    takerTokenB = await createAccount(
      provider.connection,
      taker,
      mintB,
      taker.publicKey
    );

    // Mint tokens
    await mintTo(
      provider.connection,
      maker,
      mintA,
      makerTokenA,
      maker.publicKey,
      AMOUNT_A * 10
    );

    await mintTo(
      provider.connection,
      maker,
      mintB,
      takerTokenB,
      maker.publicKey,
      AMOUNT_B * 10
    );

    // Derive escrow PDA
    [escrowPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.publicKey.toBuffer()],
      program.programId
    );

    vault = anchor.web3.Keypair.generate();
  });

  it("Initializes escrow", async () => {
    const expiration = Math.floor(Date.now() / 1000) + 3600; // 1 hour from now

    await program.methods
      .initialize(new anchor.BN(AMOUNT_A), new anchor.BN(AMOUNT_B), new anchor.BN(expiration))
      .accounts({
        maker: maker.publicKey,
        mintA,
        mintB,
        escrow: escrowPda,
        vault: vault.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([maker, vault])
      .rpc();

    const escrow = await program.account.escrow.fetch(escrowPda);
    assert.ok(escrow.maker.equals(maker.publicKey));
    assert.ok(escrow.mintA.equals(mintA));
    assert.ok(escrow.mintB.equals(mintB));
    assert.equal(escrow.amountA.toNumber(), AMOUNT_A);
    assert.equal(escrow.amountB.toNumber(), AMOUNT_B);
  });

  it("Deposits tokens into escrow", async () => {
    await program.methods
      .deposit()
      .accounts({
        maker: maker.publicKey,
        escrow: escrowPda,
        makerTokenA,
        vault: vault.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc();

    const vaultAccount = await getAccount(provider.connection, vault.publicKey);
    assert.equal(Number(vaultAccount.amount), AMOUNT_A);

    const escrow = await program.account.escrow.fetch(escrowPda);
    assert.deepEqual(escrow.state, { active: {} });
  });

  it("Completes exchange", async () => {
    const makerTokenBBefore = await getAccount(provider.connection, makerTokenB);
    const takerTokenABefore = await getAccount(provider.connection, takerTokenA);

    await program.methods
      .exchange()
      .accounts({
        taker: taker.publicKey,
        escrow: escrowPda,
        maker: maker.publicKey,
        takerTokenA,
        takerTokenB,
        makerTokenB,
        vault: vault.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();

    const makerTokenBAfter = await getAccount(provider.connection, makerTokenB);
    const takerTokenAAfter = await getAccount(provider.connection, takerTokenA);

    assert.equal(
      Number(makerTokenBAfter.amount) - Number(makerTokenBBefore.amount),
      AMOUNT_B
    );
    assert.equal(
      Number(takerTokenAAfter.amount) - Number(takerTokenABefore.amount),
      AMOUNT_A
    );

    const escrow = await program.account.escrow.fetch(escrowPda);
    assert.deepEqual(escrow.state, { completed: {} });
  });

  it("Cannot exchange completed escrow", async () => {
    try {
      await program.methods
        .exchange()
        .accounts({
          taker: taker.publicKey,
          escrow: escrowPda,
          maker: maker.publicKey,
          takerTokenA,
          takerTokenB,
          makerTokenB,
          vault: vault.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([taker])
        .rpc();
      assert.fail("Should have thrown error");
    } catch (err) {
      assert.include(err.message, "EscrowNotActive");
    }
  });
});
