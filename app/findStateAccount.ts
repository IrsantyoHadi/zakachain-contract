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

// Program ID from your Anchor.toml
const programId = new PublicKey("3EJSTPJYM3BaNBvL7haWnhXoNh5GvmsQfwL1QQ2am3GJ");

async function main() {
    try {
        console.log("Finding state accounts for program:", programId.toBase58());
        
        // Get all accounts owned by the program
        const accounts = await connection.getProgramAccounts(programId, {
            commitment: "confirmed",
        });

        console.log(`Found ${accounts.length} accounts owned by the program`);

        for (const acc of accounts) {
            console.log("Account:", acc.pubkey.toBase58());
            console.log("Size:", acc.account.data.length);
            console.log("---");
        }

        // Filter for state accounts (they will have the correct size)
        const stateAccountSize = 8 + 32 + 1 + 8 + 8 + 8 + 4 + (32 * 100) + 1; // ZakaChainState::LEN
        const stateAccounts = accounts.filter(acc => acc.account.data.length === stateAccountSize);

        console.log(`Found ${stateAccounts.length} potential state accounts:`);
        
        for (const acc of stateAccounts) {
            console.log("State Account:", acc.pubkey.toBase58());
            console.log("Size:", acc.account.data.length);
            console.log("---");
        }

    } catch (error) {
        console.error("Error:", error);
    }
}

main(); 