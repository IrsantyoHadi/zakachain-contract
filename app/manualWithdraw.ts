import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddress, getAccount, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { readFileSync } from "fs";

const PROGRAM_ID = new PublicKey("3EJSTPJYM3BaNBvL7haWnhXoNh5GvmsQfwL1QQ2am3GJ");
const USDC_MINT = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = new anchor.Program(
    JSON.parse(readFileSync("target/idl/zakachain.json", "utf8")),
    PROGRAM_ID,
    provider
  );

  const wallet = provider.wallet;
  const amil = wallet.publicKey;

  // Derive state PDA
  const [statePda] = await PublicKey.findProgramAddress([
    Buffer.from("state")
  ], PROGRAM_ID);

  // Program's token account (PDA)
  const programTokenAccount = await getAssociatedTokenAddress(
    USDC_MINT,
    statePda,
    true
  );

  // Amil's token account
  const amilTokenAccount = await getAssociatedTokenAddress(
    USDC_MINT,
    amil
  );

  // Withdraw 0.1 USDC (assuming 6 decimals)
  const withdrawalAmount = new anchor.BN(100_000); // 0.1 USDC
  const uniqueId = "WITHDRAWAL-TEST-001";

  // Call withdraw_zakat_manual
  const tx = await program.methods
    .withdrawZakatManual(withdrawalAmount, uniqueId)
    .accounts({
      state: statePda,
      amil,
      programTokenAccount,
      recipientTokenAccount: amilTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("Manual withdrawal transaction signature:", tx);

  // Print balances
  const programTokenBalance = await getAccount(provider.connection, programTokenAccount);
  const amilTokenBalance = await getAccount(provider.connection, amilTokenAccount);
  console.log("Program token account balance:", Number(programTokenBalance.amount) / 1e6, "USDC");
  console.log("Amil token account balance:", Number(amilTokenBalance.amount) / 1e6, "USDC");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
}); 