import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Zakachain } from "../target/types/zakachain";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";

describe("zakachain", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Zakachain as Program<Zakachain>;

  // Test accounts
  const amil = Keypair.generate();
  const payer = Keypair.generate();
  const mustahik = Keypair.generate();
  const amilOperational = Keypair.generate();

  // Token accounts
  let mint: PublicKey;
  let payerTokenAccount: PublicKey;
  let programTokenAccount: PublicKey;
  let amilTokenAccount: PublicKey;
  let mustahikTokenAccount: PublicKey;
  let amilOperationalTokenAccount: PublicKey;

  // Program state account
  let stateAccount: PublicKey;

  before(async () => {
    // Airdrop SOL to test accounts
    const signature1 = await provider.connection.requestAirdrop(amil.publicKey, 2 * LAMPORTS_PER_SOL);
    const signature2 = await provider.connection.requestAirdrop(payer.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature1);
    await provider.connection.confirmTransaction(signature2);

    // Create test token mint
    mint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      6
    );

    // Create token accounts
    payerTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      payer.publicKey
    );

    programTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      program.programId
    );

    amilTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      amil.publicKey
    );

    mustahikTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      mustahik.publicKey
    );

    amilOperationalTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      amilOperational.publicKey
    );

    // Mint tokens to payer
    await mintTo(
      provider.connection,
      payer,
      mint,
      payerTokenAccount,
      payer,
      1000000000 // 1000 tokens
    );
  });

  it("Initializes the contract", async () => {
    const amilFeePercentage = 25; // 2.5%

    // Find PDA for state account
    [stateAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );

    await program.methods
      .initialize(amilFeePercentage)
      .accounts({
        state: stateAccount,
        amil: amil.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([amil])
      .rpc();

    // Verify state
    const state = await program.account.zakaChainState.fetch(stateAccount);
    assert.ok(state.amil.equals(amil.publicKey));
    assert.equal(state.amilFeePercentage, amilFeePercentage);
    assert.equal(state.totalZakatCollected.toString(), "0");
    assert.equal(state.totalZakatDistributed.toString(), "0");
    assert.equal(state.totalAmilFeesCollected.toString(), "0");
    assert.ok(state.isInitialized);
  });

  it("Receives Zakat and calculates Amil fee correctly", async () => {
    const zakatAmount = new anchor.BN(1000000); // 1 token

    await program.methods
      .collectZakat(zakatAmount)
      .accounts({
        state: stateAccount,
        payer: payer.publicKey,
        payerTokenAccount: payerTokenAccount,
        programTokenAccount: programTokenAccount,
        amilTokenAccount: amilTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();

    // Verify balances
    const programTokenBalance = await getAccount(provider.connection, programTokenAccount);
    const amilTokenBalance = await getAccount(provider.connection, amilTokenAccount);

    // Expected amounts
    const expectedAmilFee = 25000; // 2.5% of 1 token
    const expectedNetAmount = 975000; // 97.5% of 1 token

    assert.equal(programTokenBalance.amount.toString(), expectedNetAmount.toString());
    assert.equal(amilTokenBalance.amount.toString(), expectedAmilFee.toString());

    // Verify state
    const state = await program.account.zakaChainState.fetch(stateAccount);
    assert.equal(state.totalZakatCollected.toString(), expectedNetAmount.toString());
    assert.equal(state.totalAmilFeesCollected.toString(), expectedAmilFee.toString());
  });

  it("Adds a mustahik", async () => {
    const uniqueId = "MUSTAHIK-001";
    const name = "Test Mustahik";

    await program.methods
      .addMustahik(uniqueId, name)
      .accounts({
        state: stateAccount,
        amil: amil.publicKey,
        mustahik: mustahik.publicKey,
        mustahikAccount: await PublicKey.findProgramAddressSync(
          [Buffer.from("mustahik"), mustahik.publicKey.toBuffer()],
          program.programId
        )[0],
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([amil])
      .rpc();

    // Verify mustahik account
    const mustahikAccount = await program.account.mustahikAccount.fetch(
      await PublicKey.findProgramAddressSync(
        [Buffer.from("mustahik"), mustahik.publicKey.toBuffer()],
        program.programId
      )[0]
    );
    assert.ok(mustahikAccount.address.equals(mustahik.publicKey));
    assert.equal(mustahikAccount.uniqueId, uniqueId);
    assert.equal(mustahikAccount.name, name);
    assert.ok(mustahikAccount.isActive);
  });

  it("Distributes Zakat to a mustahik", async () => {
    const distributionAmount = new anchor.BN(100000); // 0.1 token

    await program.methods
      .distributeToMustahik(distributionAmount)
      .accounts({
        state: stateAccount,
        amil: amil.publicKey,
        mustahik: mustahik.publicKey,
        mustahikAccount: await PublicKey.findProgramAddressSync(
          [Buffer.from("mustahik"), mustahik.publicKey.toBuffer()],
          program.programId
        )[0],
        programTokenAccount: programTokenAccount,
        mustahikTokenAccount: mustahikTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([amil])
      .rpc();

    // Verify balances
    const mustahikTokenBalance = await getAccount(provider.connection, mustahikTokenAccount);
    assert.equal(mustahikTokenBalance.amount.toString(), distributionAmount.toString());

    // Verify state
    const state = await program.account.zakaChainState.fetch(stateAccount);
    assert.equal(state.totalZakatDistributed.toString(), distributionAmount.toString());
  });

  it("Withdraws zakat manually as amil", async () => {
    const withdrawalAmount = new anchor.BN(100000); // 0.1 token
    const uniqueId = "WITHDRAWAL-001";

    await program.methods
      .withdrawZakatManual(withdrawalAmount, uniqueId)
      .accounts({
        state: stateAccount,
        amil: amil.publicKey,
        programTokenAccount: programTokenAccount,
        recipientTokenAccount: amilTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([amil])
      .rpc();

    // Verify balances
    const amilTokenBalance = await getAccount(provider.connection, amilTokenAccount);
    assert.equal(amilTokenBalance.amount.toString(), withdrawalAmount.toString());

    // Verify state
    const state = await program.account.zakaChainState.fetch(stateAccount);
    assert.equal(state.totalZakatDistributed.toString(), withdrawalAmount.toString());
    assert.equal(state.withdrawalCount, 1);
    assert.equal(state.manualWithdrawalCount, 1);
  });

  it("Fails to withdraw zakat manually with unauthorized account", async () => {
    const unauthorizedAccount = Keypair.generate();
    const withdrawalAmount = new anchor.BN(100000);
    const uniqueId = "WITHDRAWAL-002";

    try {
      await program.methods
        .withdrawZakatManual(withdrawalAmount, uniqueId)
        .accounts({
          state: stateAccount,
          amil: unauthorizedAccount.publicKey,
          programTokenAccount: programTokenAccount,
          recipientTokenAccount: amilTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([unauthorizedAccount])
        .rpc();
      assert.fail("Expected error for unauthorized withdrawal");
    } catch (error) {
      assert.include(error.message, "Unauthorized");
    }
  });

  it("Fails to withdraw zakat manually to non-amil account", async () => {
    const withdrawalAmount = new anchor.BN(100000);
    const uniqueId = "WITHDRAWAL-003";

    try {
      await program.methods
        .withdrawZakatManual(withdrawalAmount, uniqueId)
        .accounts({
          state: stateAccount,
          amil: amil.publicKey,
          programTokenAccount: programTokenAccount,
          recipientTokenAccount: mustahikTokenAccount, // Using mustahik's account instead of amil's
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([amil])
        .rpc();
      assert.fail("Expected error for invalid recipient");
    } catch (error) {
      assert.include(error.message, "Invalid recipient");
    }
  });

  it("Fails to withdraw zakat manually with insufficient funds", async () => {
    const withdrawalAmount = new anchor.BN(1000000000); // 1000 tokens (more than available)
    const uniqueId = "WITHDRAWAL-004";

    try {
      await program.methods
        .withdrawZakatManual(withdrawalAmount, uniqueId)
        .accounts({
          state: stateAccount,
          amil: amil.publicKey,
          programTokenAccount: programTokenAccount,
          recipientTokenAccount: amilTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([amil])
        .rpc();
      assert.fail("Expected error for insufficient funds");
    } catch (error) {
      assert.include(error.message, "Insufficient funds");
    }
  });

  it("Fails to withdraw zakat manually with invalid amount", async () => {
    const withdrawalAmount = new anchor.BN(0); // Invalid amount
    const uniqueId = "WITHDRAWAL-005";

    try {
      await program.methods
        .withdrawZakatManual(withdrawalAmount, uniqueId)
        .accounts({
          state: stateAccount,
          amil: amil.publicKey,
          programTokenAccount: programTokenAccount,
          recipientTokenAccount: amilTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([amil])
        .rpc();
      assert.fail("Expected error for invalid amount");
    } catch (error) {
      assert.include(error.message, "Invalid amount");
    }
  });

  it("Fails to withdraw zakat manually with too long unique ID", async () => {
    const withdrawalAmount = new anchor.BN(100000);
    const uniqueId = "WITHDRAWAL-006".repeat(20); // Too long unique ID

    try {
      await program.methods
        .withdrawZakatManual(withdrawalAmount, uniqueId)
        .accounts({
          state: stateAccount,
          amil: amil.publicKey,
          programTokenAccount: programTokenAccount,
          recipientTokenAccount: amilTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([amil])
        .rpc();
      assert.fail("Expected error for too long unique ID");
    } catch (error) {
      assert.include(error.message, "Unique ID is too long");
    }
  });

  it("Fails to initialize with invalid fee percentage", async () => {
    const invalidFeePercentage = 126; // 12.6% (above max of 12.5%)

    try {
      await program.methods
        .initialize(invalidFeePercentage)
        .accounts({
          state: stateAccount,
          amil: amil.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([amil])
        .rpc();
      assert.fail("Expected error for invalid fee percentage");
    } catch (error) {
      assert.include(error.message, "Invalid fee percentage");
    }
  });

  it("Fails to add mustahik with unauthorized account", async () => {
    const unauthorizedAccount = Keypair.generate();
    const uniqueId = "MUSTAHIK-002";
    const name = "Test Mustahik 2";

    try {
      await program.methods
        .addMustahik(uniqueId, name)
        .accounts({
          state: stateAccount,
          amil: unauthorizedAccount.publicKey,
          mustahik: mustahik.publicKey,
          mustahikAccount: await PublicKey.findProgramAddressSync(
            [Buffer.from("mustahik"), mustahik.publicKey.toBuffer()],
            program.programId
          )[0],
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([unauthorizedAccount])
        .rpc();
      assert.fail("Expected error for unauthorized account");
    } catch (error) {
      assert.include(error.message, "Unauthorized");
    }
  });
});
