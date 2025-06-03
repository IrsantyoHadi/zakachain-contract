# ZakaChain

A Solana-based smart contract for managing zakat collection and distribution.

## Program ID

- **ZakaChain:** 3EJSTPJYM3BaNBvL7haWnhXoNh5GvmsQfwL1QQ2am3GJ

## Features

- **Initialize:** Set up the ZakaChain program with an amil and fee percentage.
- **Update Authority:** Change the amil (authority) of the program.
- **Update Fee Percentage:** Modify the amil fee percentage.
- **Add Mustahik:** Register a new mustahik (recipient) for zakat distribution.
- **Remove Mustahik:** Remove a mustahik from the list of recipients.
- **Collect Zakat:** Collect zakat from a payer and distribute fees to the amil.
- **Distribute to Mustahik:** Distribute collected zakat to a registered mustahik.
- **Withdraw Amil Fees:** Allow the amil to withdraw collected fees.

## Instructions

### Initialize

```typescript
await program.methods
    .initialize(new anchor.BN(500)) // 0.5% fee
    .accounts({
        state: statePda,
        amil: wallet.publicKey,
        systemProgram: SystemProgram.programId,
    })
    .rpc();
```

### Update Authority

```typescript
await program.methods
    .updateAuthority(newAuthority)
    .accounts({
        state: statePda,
        amil: wallet.publicKey,
    })
    .rpc();
```

### Update Fee Percentage

```typescript
await program.methods
    .updateFeePercentage(new anchor.BN(1000)) // 1% fee
    .accounts({
        state: statePda,
        amil: wallet.publicKey,
    })
    .rpc();
```

### Add Mustahik

```typescript
await program.methods
    .addMustahik(mustahikWallet.publicKey)
    .accounts({
        state: statePda,
        amil: wallet.publicKey,
    })
    .rpc();
```

### Remove Mustahik

```typescript
await program.methods
    .removeMustahik(mustahikWallet.publicKey)
    .accounts({
        state: statePda,
        amil: wallet.publicKey,
    })
    .rpc();
```

### Collect Zakat

```typescript
await program.methods
    .collectZakat(new anchor.BN(1000000)) // 1 USDC
    .accounts({
        state: statePda,
        payer: wallet.publicKey,
        payerTokenAccount: payerTokenAccount,
        programTokenAccount: programTokenAccount,
        amilTokenAccount: amilTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
```

### Distribute to Mustahik

```typescript
await program.methods
    .distributeToMustahik(mustahikWallet.publicKey, new anchor.BN(500000)) // 0.5 USDC
    .accounts({
        state: statePda,
        amil: wallet.publicKey,
        mustahik: mustahikWallet.publicKey,
        mustahikTokenAccount: mustahikTokenAccount,
        programTokenAccount: programTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
```

### Withdraw Amil Fees

```typescript
await program.methods
    .withdrawAmilFees()
    .accounts({
        state: statePda,
        amil: wallet.publicKey,
        amilTokenAccount: amilTokenAccount,
        programTokenAccount: programTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
```

## Testing

Run the tests using:

```bash
anchor test
```

## Deployment

Deploy the program to devnet using:

```bash
anchor deploy --provider.cluster devnet
```

## License

MIT 