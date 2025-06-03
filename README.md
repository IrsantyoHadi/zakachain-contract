# ZakaChain Smart Contract Documentation

## Overview
ZakaChain is a Solana-based smart contract for managing Zakat (Islamic almsgiving) collection and distribution. It provides a secure, transparent, and automated way to handle Zakat funds while ensuring proper distribution according to Islamic principles.

## Key Features

### 1. Program Initialization
- Only the program deployer can initialize the contract
- Sets up the Amil (Zakat administrator) and their fee percentage
- Maximum Amil fee is capped at 12.5%

### 2. Zakat Collection
- Accepts Zakat payments in USDC
- Automatically calculates and transfers Amil fees
- Tracks total Zakat collected and Amil fees
- Emits events for transparency

### 3. Mustahik Management
- Amil can add Mustahiks (eligible recipients)
- Maximum of 100 Mustahiks can be registered
- Each Mustahik must have a valid Solana wallet address

### 4. Zakat Distribution
- Automated distribution to registered Mustahiks
- Manual distribution option for special cases
- Tracks total distributed amount

### 5. Manual Withdrawal System
- Amil can withdraw funds for manual distribution
- Security features:
  - Only amil can receive withdrawn funds
  - Unique ID required for each withdrawal
  - Withdrawal tracking and counting
  - Event emission for transparency

## Account Structure

### State Account
```rust
pub struct ZakaChainState {
    pub amil: Pubkey,                    // Amil's wallet address
    pub amil_fee_percentage: u8,         // Fee percentage (max 12.5%)
    pub total_zakat_collected: u64,      // Total Zakat received
    pub total_zakat_distributed: u64,    // Total Zakat distributed
    pub total_amil_fees_collected: u64,  // Total fees collected by Amil
    pub mustahiks: Vec<Pubkey>,          // List of registered Mustahiks
    pub is_initialized: bool,            // Initialization status
    pub last_withdrawal_timestamp: i64,  // Last manual withdrawal time
    pub manual_withdrawal_count: u32,    // Total manual withdrawals
}
```

## Instructions

### 1. Initialize
```rust
pub fn initialize(
    ctx: Context<Initialize>,
    amil_fee_percentage: u8,
) -> Result<()>
```
- Initializes the program
- Sets up Amil and fee percentage
- Can only be called by program deployer

### 2. Receive Zakat
```rust
pub fn receive_zakat(
    ctx: Context<ReceiveZakat>,
    amount: u64,
) -> Result<()>
```
- Accepts Zakat payments
- Calculates and transfers Amil fees
- Updates total collected amounts

### 3. Add Mustahik
```rust
pub fn add_mustahik(
    ctx: Context<AddMustahik>,
    mustahik_address: Pubkey,
) -> Result<()>
```
- Registers a new Mustahik
- Can only be called by Amil
- Maximum 100 Mustahiks

### 4. Distribute to Mustahik
```rust
pub fn distribute_to_mustahik(
    ctx: Context<DistributeToMustahik>,
    amount: u64,
) -> Result<()>
```
- Distributes Zakat to a registered Mustahik
- Can only be called by Amil
- Mustahik must be registered

### 5. Withdraw Zakat Manual
```rust
pub fn withdraw_zakat_manual(
    ctx: Context<WithdrawZakatManual>,
    amount: u64,
    unique_id: String,
) -> Result<()>
```
- Withdraws funds for manual distribution
- Only amil can receive withdrawn funds
- Requires unique ID for tracking
- Emits withdrawal event

## Events

### 1. ZakatReceived
```rust
pub struct ZakatReceived {
    pub payer: Pubkey,
    pub amount: u64,
    pub amil_fee: u64,
    pub timestamp: i64,
}
```

### 2. MustahikAdded
```rust
pub struct MustahikAdded {
    pub mustahik: Pubkey,
    pub timestamp: i64,
}
```

### 3. ZakatDistributed
```rust
pub struct ZakatDistributed {
    pub mustahik: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
```

### 4. ZakatWithdrawn
```rust
pub struct ZakatWithdrawn {
    pub amount: u64,
    pub unique_id: String,
    pub timestamp: i64,
    pub withdrawal_count: u32,
}
```

## Error Types

1. `NotInitialized`: Contract not initialized
2. `InvalidFeePercentage`: Fee percentage exceeds 12.5%
3. `Unauthorized`: Unauthorized access attempt
4. `InvalidMustahik`: Invalid Mustahik address
5. `InvalidTokenMint`: Token mint mismatch
6. `InvalidTokenAccount`: Invalid token account
7. `InsufficientFunds`: Insufficient funds for withdrawal
8. `InvalidRecipient`: Recipient is not the amil
9. `UniqueIdTooLong`: Unique ID exceeds 200 characters

## Security Features

1. **Access Control**
   - Program deployer only initialization
   - Amil-only operations for sensitive functions
   - PDA-based state account
   - Amil-only recipient for withdrawals

2. **Amount Limits**
   - Maximum Amil fee: 12.5%
   - Withdrawal limits based on available funds

3. **Validation**
   - Token account ownership checks
   - Token mint verification
   - Unique ID length limits
   - Mustahik registration limits

4. **Transparency**
   - Event emission for all operations
   - Withdrawal tracking and counting
   - Timestamp recording

## Usage Examples

### Initializing the Program
```typescript
await program.methods
    .initialize(50) // 5% fee
    .accounts({
        state: stateAccount,
        amil: amilWallet.publicKey,
        systemProgram: SystemProgram.programId,
    })
    .rpc();
```

### Receiving Zakat
```typescript
await program.methods
    .receiveZakat(new BN(1000000)) // 1 USDC
    .accounts({
        state: stateAccount,
        payer: payerWallet.publicKey,
        payerTokenAccount: payerTokenAccount,
        programTokenAccount: programTokenAccount,
        amilTokenAccount: amilTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
```

### Manual Withdrawal
```typescript
await program.methods
    .withdrawZakatManual(
        new BN(100000000), // 100 USDC
        "WITHDRAWAL-001"
    )
    .accounts({
        state: stateAccount,
        amil: amilWallet.publicKey,
        programTokenAccount: programTokenAccount,
        recipientTokenAccount: amilTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();
```

## Future Improvements

1. **Enhanced Distribution**
   - Automated distribution algorithms
   - Priority-based distribution
   - Distribution scheduling

2. **Additional Security**
   - Multi-signature requirements
   - Emergency pause functionality
   - Enhanced audit trails

3. **Reporting**
   - Detailed analytics
   - Distribution reports
   - Financial statements

4. **Integration**
   - Web interface
   - Mobile application
   - API endpoints

## Contributing
Contributions are welcome! Please read our contributing guidelines before submitting pull requests.

## License
This project is licensed under the MIT License - see the LICENSE file for details. 