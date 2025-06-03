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
- Each Mustahik has their own PDA account with detailed information
- Each Mustahik must have a valid Solana wallet address
- View mustahik status using the `view_mustahik_status` function
- Mustahik information includes:
  - Unique ID for tracking
  - Name
  - Active status
  - Creation and update timestamps

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
    pub is_initialized: bool,            // Initialization status
    pub last_withdrawal_timestamp: i64,  // Last manual withdrawal time
    pub manual_withdrawal_count: u32,    // Total manual withdrawals
    pub withdrawal_count: u32,           // Total withdrawals
}
```

### Mustahik Account
```rust
pub struct MustahikAccount {
    pub address: Pubkey,                 // Mustahik's wallet address
    pub unique_id: String,               // Unique identifier
    pub name: String,                    // Mustahik's name
    pub is_active: bool,                 // Active status
    pub created_at: i64,                 // Creation timestamp
    pub updated_at: i64,                 // Last update timestamp
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

### 2. Update Authority
```rust
pub fn update_authority(
    ctx: Context<UpdateAuthority>,
    new_authority: Pubkey,
) -> Result<()>
```
- Updates the Amil (authority) of the program
- Can only be called by current Amil

### 3. Update Fee Percentage
```rust
pub fn update_fee_percentage(
    ctx: Context<UpdateFeePercentage>,
    new_fee_percentage: u8,
) -> Result<()>
```
- Updates the Amil fee percentage
- Can only be called by Amil
- Maximum fee is 12.5%

### 4. Collect Zakat
```rust
pub fn collect_zakat(
    ctx: Context<CollectZakat>,
    amount: u64,
) -> Result<()>
```
- Accepts Zakat payments
- Calculates and transfers Amil fees
- Updates total collected amounts

### 5. Add Mustahik
```rust
pub fn add_mustahik(
    ctx: Context<AddMustahik>,
    unique_id: String,
    name: String,
) -> Result<()>
```
- Registers a new Mustahik
- Can only be called by Amil
- Each Mustahik has their own PDA account

### 6. Remove Mustahik
```rust
pub fn remove_mustahik(
    ctx: Context<RemoveMustahik>,
) -> Result<()>
```
- Deactivates a Mustahik
- Can only be called by Amil
- Mustahik must be active

### 7. Distribute to Mustahik
```rust
pub fn distribute_to_mustahik(
    ctx: Context<DistributeToMustahik>,
    amount: u64,
) -> Result<()>
```
- Distributes Zakat to a registered Mustahik
- Can only be called by Amil
- Mustahik must be active

### 8. Withdraw Amil Fees
```rust
pub fn withdraw_amil_fees(
    ctx: Context<WithdrawAmilFees>,
    amount: u64,
) -> Result<()>
```
- Allows Amil to withdraw collected fees
- Can only be called by Amil
- Amount must not exceed collected fees

### 9. Withdraw Zakat Manual
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

### 10. View Mustahik Status
```rust
pub fn view_mustahik_status(
    ctx: Context<GetMustahikStatus>,
) -> Result<MustahikStatusView>
```
- Returns the status of a mustahik
- Can be called by anyone
- Returns a `MustahikStatusView` struct with mustahik details

## Events

### 1. ZakatCollected
```rust
pub struct ZakatCollected {
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
    pub mustahik_account: Pubkey,
    pub timestamp: i64,
}
```

### 3. MustahikRemoved
```rust
pub struct MustahikRemoved {
    pub mustahik: Pubkey,
    pub mustahik_account: Pubkey,
    pub timestamp: i64,
}
```

### 4. ZakatDistributed
```rust
pub struct ZakatDistributed {
    pub mustahik: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
```

### 5. AmilFeesWithdrawn
```rust
pub struct AmilFeesWithdrawn {
    pub amil: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
```

### 6. ZakatWithdrawn
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
4. `MaxMustahiksReached`: Maximum number of mustahiks reached
5. `InvalidMustahik`: Invalid Mustahik address or inactive mustahik
6. `InsufficientFunds`: Insufficient funds for withdrawal
7. `DescriptionTooLong`: Description exceeds maximum length
8. `InvalidRecipient`: Recipient is not the amil
9. `InvalidAmount`: Amount is zero or invalid
10. `UniqueIdTooLong`: Unique ID exceeds maximum length

## Constants and Limits

### Program Constants
```rust
pub const MAX_DESCRIPTION_LEN: usize = 100;    // Maximum length for descriptions and unique IDs
pub const PROGRAM_DEPLOYER: &str = "...";      // Fixed program deployer address
```

### Account Size Constants
```rust
// State Account Size
pub const STATE_ACCOUNT_SIZE: usize = 32 + 1 + 8 + 8 + 8 + 8 + 4 + 4;  // 73 bytes
// - 32 bytes for amil pubkey
// - 1 byte for fee percentage
// - 8 bytes for each u64 field (total_zakat_collected, total_zakat_distributed, etc.)
// - 4 bytes for each u32 field (manual_withdrawal_count, withdrawal_count)

// Mustahik Account Size
pub const MUSTAHIK_ACCOUNT_SIZE: usize = 32 + 100 + 100 + 1 + 8 + 8;  // 249 bytes
// - 32 bytes for address pubkey
// - 100 bytes for unique_id string
// - 100 bytes for name string
// - 1 byte for is_active boolean
// - 8 bytes for each timestamp (created_at, updated_at)
```

## Security Features

1. **Access Control**
   - Program deployer only initialization
   - Amil-only operations for sensitive functions
   - PDA-based state account
   - Amil-only recipient for withdrawals
   - Fixed program deployer address verification

2. **Amount Limits**
   - Maximum Amil fee: 12.5%
   - Withdrawal limits based on available funds
   - Maximum description/unique ID length: 100 characters

3. **Validation**
   - Token account ownership checks
   - Token mint verification
   - Unique ID length limits
   - Overflow protection for all arithmetic operations
   - Timestamp validation for all operations

4. **Transparency**
   - Event emission for all operations
   - Withdrawal tracking and counting
   - Timestamp recording
   - Detailed state tracking
   - PDA-based account derivation

5. **Account Security**
   - Fixed-size account structures
   - PDA-based account derivation
   - Explicit account validation
   - Owner checks for all accounts
   - Signer verification for sensitive operations

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

### View Mustahik Status
```typescript
const mustahikStatus = await program.methods
    .viewMustahikStatus()
    .accounts({
        mustahik: mustahikWallet.publicKey,
        mustahikAccount: mustahikAccountPda,
    })
    .view();

console.log("Mustahik Status:", mustahikStatus);
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