import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Zakachain } from "../target/types/zakachain";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";
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

async function main() {
    try {
        // Derive the PDA for the state account
        const [statePda] = await PublicKey.findProgramAddress(
            [Buffer.from("state")],
            programId
        );
        console.log("State PDA:", statePda.toBase58());

        // Generate a random wallet for the mustahik
        const mustahikWallet = Keypair.generate();
        console.log("Mustahik Wallet:", mustahikWallet.publicKey.toBase58());

        // Add mustahik
        console.log("\nAdding mustahik...");
        const tx = await program.methods
            .addMustahik("MUSTAHIK-001", "Mustahik 1")
            .accounts({
                state: statePda,
                amil: wallet.publicKey,
            })
            .rpc();
        
        console.log("Transaction signature:", tx);
        console.log("Mustahik added successfully!");

    } catch (error) {
        console.error("Error:", error);
    }
}

main(); 