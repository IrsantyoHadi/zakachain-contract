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

        // Derive the PDA for the mustahik account
        const [mustahikAccountPda] = await PublicKey.findProgramAddress(
            [Buffer.from("mustahik"), mustahikWallet.publicKey.toBuffer()],
            programId
        );
        console.log("Mustahik Account PDA:", mustahikAccountPda.toBase58());

        // Add mustahik
        console.log("\nAdding mustahik...");
        const tx = await program.methods
            .addMustahik("MUSTAHIK-001", "Mustahik 1")
            .accounts({
                state: statePda,
                amil: wallet.publicKey,
                mustahik: mustahikWallet.publicKey,
                mustahikAccount: mustahikAccountPda,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .rpc();
        
        console.log("Transaction signature:", tx);
        console.log("Mustahik added successfully!");

        // Check mustahik status using view_mustahik_status
        console.log("\nChecking mustahik status...");
        const mustahikStatus = await program.methods
            .viewMustahikStatus()
            .accounts({
                mustahik: mustahikWallet.publicKey,
                mustahikAccount: mustahikAccountPda,
            })
            .view();

        console.log("Mustahik Status:", mustahikStatus);

        // Fetch the mustahik account to display the data
        const mustahikAccount = await program.account.mustahikAccount.fetch(mustahikAccountPda);
        console.log("Mustahik Status:", {
            address: mustahikAccount.address.toBase58(),
            name: mustahikAccount.name,
            uniqueId: mustahikAccount.uniqueId,
            isActive: mustahikAccount.isActive,
            createdAt: new Date(mustahikAccount.createdAt.toNumber() * 1000).toISOString(),
        });

    } catch (error) {
        console.error("Error:", error);
    }
}

main(); 