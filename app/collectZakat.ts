import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Zakachain } from "../target/types/zakachain";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Connection, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, createMint, createAssociatedTokenAccount, mintTo, getAccount, getAssociatedTokenAddress, createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import * as fs from 'fs';

// Configure the client to use the devnet cluster
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

// Load wallet from Solana config
const walletKeypair = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(process.env.HOME + '/.config/solana/id.json', 'utf-8')))
);
const wallet = new anchor.Wallet(walletKeypair);

console.log("Wallet:", wallet.publicKey.toBase58());

const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
});
anchor.setProvider(provider);

// Program ID from your Anchor.toml
const programId = new PublicKey("3EJSTPJYM3BaNBvL7haWnhXoNh5GvmsQfwL1QQ2am3GJ");
const program = anchor.workspace.Zakachain as Program<Zakachain>;

// USDC mint address on devnet
const usdcMint = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");

async function main() {
    try {
        // Derive the PDA for the state account
        const [statePda] = await PublicKey.findProgramAddress(
            [Buffer.from("state")],
            programId
        );
        console.log("State PDA:", statePda.toBase58());

        // Create token accounts
        console.log("Creating token accounts...");
        
        // Payer's token account (using existing USDC token account)
        const payerTokenAccount = await getAssociatedTokenAddress(
            usdcMint,
            wallet.publicKey
        );
        console.log("Payer Token Account:", payerTokenAccount.toBase58());

        // Program's token account (PDA)
        const programTokenAccount = await getAssociatedTokenAddress(
            usdcMint,
            statePda,
            true // allowOwnerOffCurve
        );
        // Create the associated token account for the PDA if it doesn't exist
        const programTokenAccountInfo = await connection.getAccountInfo(programTokenAccount);
        if (!programTokenAccountInfo) {
            const ataIx = createAssociatedTokenAccountInstruction(
                wallet.publicKey, // payer
                programTokenAccount, // ata
                statePda, // owner (PDA)
                usdcMint
            );
            const tx = new anchor.web3.Transaction().add(ataIx);
            await provider.sendAndConfirm(tx, []);
            console.log("Created program's token account for PDA:", programTokenAccount.toBase58());
        } else {
            console.log("Program Token Account already exists:", programTokenAccount.toBase58());
        }

        // Amil's token account
        const amilTokenAccount = await getAssociatedTokenAddress(
            usdcMint,
            wallet.publicKey
        );
        console.log("Amil Token Account:", amilTokenAccount.toBase58());

        // Check payer's balance
        const payerAccount = await getAccount(connection, payerTokenAccount);
        console.log("Payer USDC Balance:", Number(payerAccount.amount) / 1e6, "USDC");

        // Collect zakat (1 USDC)
        console.log("\nCollecting zakat...");
        const zakatAmount = 1 * 1e6; // 1 USDC with 6 decimals
        const tx = await program.methods
            .collectZakat(new anchor.BN(zakatAmount))
            .accounts({
                state: statePda,
                payer: wallet.publicKey,
                payerTokenAccount: payerTokenAccount,
                programTokenAccount: programTokenAccount,
                amilTokenAccount: amilTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc();
        
        console.log("Transaction signature:", tx);
        console.log("Zakat collected successfully!");

        // Check final balances
        const finalPayerAccount = await getAccount(connection, payerTokenAccount);
        const finalProgramAccount = await getAccount(connection, programTokenAccount);
        const finalAmilAccount = await getAccount(connection, amilTokenAccount);

        console.log("\nFinal Balances:");
        console.log("Payer:", Number(finalPayerAccount.amount) / 1e6, "USDC");
        console.log("Program:", Number(finalProgramAccount.amount) / 1e6, "USDC");
        console.log("Amil:", Number(finalAmilAccount.amount) / 1e6, "USDC");

    } catch (error) {
        console.error("Error:", error);
    }
}

main(); 