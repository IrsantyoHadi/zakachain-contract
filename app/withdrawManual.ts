import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import * as fs from 'fs';

// Configure the client to use the devnet cluster
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

// Load wallet from Solana config
const walletKeypair = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(process.env.HOME + '/.config/solana/id.json', 'utf-8')))
);
const wallet = new anchor.Wallet(walletKeypair);

const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
});
anchor.setProvider(provider);

// Program ID from your Anchor.toml
const programId = new PublicKey("GAvHenbgHgjax1zxam7qqi61qU7gxjAkmg3Ryy4jY5oP");

// Load the IDL
const idl = JSON.parse(fs.readFileSync("./target/idl/zakachain.json", "utf8"));
const program = new Program(idl, programId, provider);

// State account public key
const stateAccount = new PublicKey("7JBnuMFKCapRbPgYfw5CsYUnnV9gPNu4gwmuoU2gozoa");

// USDC devnet mint address
const usdcMint = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");

async function main() {
    try {
        console.log("Preparing to withdraw zakat for manual distribution...");
        
        // Get the program's token account
        const programTokenAccount = await getAssociatedTokenAddress(usdcMint, programId, true);
        console.log("Program Token Account:", programTokenAccount.toBase58());

        // Get the amil's operational token account
        const amilOperationalAccount = await getAssociatedTokenAddress(usdcMint, wallet.publicKey);
        console.log("Amil Operational Account:", amilOperationalAccount.toBase58());

        // Withdraw 0.5 USDC for manual distribution
        const amount = new anchor.BN(500000); // 0.5 USDC (6 decimals)
        const description = "Manual distribution for emergency cases";

        console.log("\nWithdrawing", amount.toNumber() / 1e6, "USDC for manual distribution...");
        
        const tx = await program.methods
            .withdrawForManualDistribution(amount, description)
            .accounts({
                state: stateAccount,
                amil: wallet.publicKey,
                programTokenAccount: programTokenAccount,
                amilOperationalAccount: amilOperationalAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc();
        
        console.log("Transaction signature:", tx);
        console.log("Withdrawal successful!");

    } catch (error) {
        console.error("Error:", error);
    }
}

main(); 