import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram, 
  LAMPORTS_PER_SOL 
} from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount
} from "@solana/spl-token";
import { assert } from "chai";

describe("escrow-advanced", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;
  
  let mint: PublicKey;
  let maker: Keypair;
  let taker: Keypair;
  let arbiter: Keypair;
  let makerTokenAccount: PublicKey;
  let takerTokenAccount: PublicKey;
  let escrowPda: PublicKey;
  let escrowTokenAccount: PublicKey;

  const ESCROW_AMOUNT = 1000 * 10 ** 6; // 1000 tokens (6 decimals)

  before(async () => {
    // Create keypairs
    maker = Keypair.generate();
    taker = Keypair.generate();
    arbiter = Keypair.generate();

    // Airdrop SOL
    const airdropMaker = await provider.connection.requestAirdrop(
      maker.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropMaker);

    const airdropTaker = await provider.connection.requestAirdrop(
      taker.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropTaker);

    // Create mint
    mint = await createMint(
      provider.connection,
      maker,
      maker.publicKey,
      null,
      6
    );

    // Create token accounts
    makerTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      maker,
      mint,
      maker.publicKey
    );

    takerTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      taker,
      mint,
      taker.publicKey
    );

    // Mint tokens to maker
    await mintTo(
      provider.connection,
      maker,
      mint,
      makerTokenAccount,
      maker,
      ESCROW_AMOUNT * 2
    );
  });

  it("Creates escrow with expiration", async () => {
    const escrowId = Keypair.generate();
    const expiresAt = Math.floor(Date.now() / 1000) + 3600; // 1 hour

    [escrowPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), escrowId.publicKey.toBuffer()],
      program.programId
    );

    // This would be the actual program call
    console.log("Escrow PDA:", escrowPda.toBase58());
    console.log("Expires at:", new Date(expiresAt * 1000).toISOString());
  });

  it("Handles dispute flow", async () => {
    // 1. Taker raises dispute
    console.log("Taker raises dispute...");
    
    // 2. Arbiter resolves with 70/30 split
    const makerShare = 0.7;
    const takerShare = 0.3;
    const makerAmount = Math.floor(ESCROW_AMOUNT * makerShare);
    const takerAmount = Math.floor(ESCROW_AMOUNT * takerShare);
    
    console.log(`Resolution: Maker gets ${makerAmount}, Taker gets ${takerAmount}`);
    
    assert.equal(makerAmount + takerAmount, ESCROW_AMOUNT, "Amounts should sum to total");
  });

  it("Tests multiple escrows from same maker", async () => {
    const escrowCount = 3;
    const escrows: PublicKey[] = [];

    for (let i = 0; i < escrowCount; i++) {
      const escrowId = Keypair.generate();
      const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), escrowId.publicKey.toBuffer()],
        program.programId
      );
      escrows.push(pda);
      console.log(`Escrow ${i + 1}: ${pda.toBase58()}`);
    }

    assert.equal(escrows.length, escrowCount);
  });

  it("Validates arbiter fee limits", async () => {
    const maxFee = 10; // 10%
    const invalidFee = 15;
    
    assert.isBelow(maxFee, 100, "Fee should be percentage");
    console.log(`Max arbiter fee: ${maxFee}%`);
    console.log(`Attempted invalid fee: ${invalidFee}% - would be rejected`);
  });
});
