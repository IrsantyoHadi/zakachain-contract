import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";
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
const programId = new PublicKey("3EJSTPJYM3BaNBvL7haWnhXoNh5GvmsQfwL1QQ2am3GJ");

// Load the IDL
const idl = JSON.parse(fs.readFileSync("./target/idl/zakachain.json", "utf8"));
const program = new Program(idl, programId, provider);

async function main() {
    try {
        console.log("Fetching ZakaChain state...");
        
        // Derive the PDA for the state account
        const [stateAccount] = await PublicKey.findProgramAddress(
            [Buffer.from("state")],
            programId
        );

        console.log("State account:", stateAccount.toBase58());
        
        // Fetch the state account
        const state = await program.account.zakaChainState.fetch(stateAccount);
        
        // Display state information
        console.log("\n=== ZakaChain State ===");
        console.log("Amil:", (state as any).amil.toBase58());
        console.log("Amil Fee Percentage:", (state as any).amilFeePercentage / 10, "%"); // Convert from basis points
        console.log("Total Zakat Collected:", (state as any).totalZakatCollected.toNumber() / 1e6, "USDC"); // Convert from lamports
        console.log("Total Zakat Distributed:", (state as any).totalZakatDistributed.toNumber() / 1e6, "USDC");
        console.log("Total Amil Fees Collected:", (state as any).totalAmilFeesCollected.toNumber() / 1e6, "USDC");
        console.log("Is Initialized:", (state as any).isInitialized);
        
        // Display mustahiks
        console.log("\n=== Mustahiks ===");
        const mustahiks = (state as any).mustahiks as PublicKey[];
        if (mustahiks.length === 0) {
            console.log("No mustahiks registered yet");
        } else {
            mustahiks.forEach((mustahik, index) => {
                console.log(`${index + 1}. ${mustahik.toBase58()}`);
            });
        }

    } catch (error) {
        console.error("Error:", error);
    }
}

main(); 