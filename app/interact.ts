import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Zakachain } from "../target/types/zakachain";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Connection, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
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

console.log("Program ID:", programId.toBase58());

async function main() {
    try {
        // Derive the PDA for the state account
        const [statePda, stateBump] = await PublicKey.findProgramAddress(
            [Buffer.from("state")],
            programId
        );
        console.log("State PDA:", statePda.toBase58(), "Bump:", stateBump);

        // Example: Initialize the program
        console.log("Initializing program...");
        const tx = await program.methods
            .initialize(5) // 5% amil fee
            .accounts({
                state: statePda,
                amil: provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();
        
        console.log("Transaction signature:", tx);
        console.log("Program initialized successfully!");

    } catch (error) {
        console.error("Error:", error);
    }
}

main(); 