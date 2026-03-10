# Solana Escrow Program

A secure, trustless escrow smart contract built with the Anchor framework for peer-to-peer token exchanges on Solana.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Solana](https://img.shields.io/badge/Solana-black?style=flat&logo=solana&logoColor=14F195)
![Anchor](https://img.shields.io/badge/Anchor-purple?style=flat)
![License](https://img.shields.io/badge/License-MIT-green)

## Overview

This program enables trustless token swaps between two parties without requiring an intermediary. Party A deposits tokens into an escrow vault, and Party B can complete the exchange by depositing their tokens, triggering an atomic swap.

## Features

- Trustless P2P token exchanges
- Atomic swap execution
- Time-locked escrows with expiration
- Partial fills support
- Cancel and refund mechanism
- Multi-token support (any SPL token)
- Comprehensive test coverage

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Party A   │────▶│   Escrow    │◀────│   Party B   │
│  (Maker)    │     │   Vault     │     │  (Taker)    │
└─────────────┘     └─────────────┘     └─────────────┘
      │                   │                   │
      │ deposit_a         │                   │ deposit_b
      ▼                   ▼                   ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Token A    │     │  Escrow     │     │  Token B    │
│  Account    │     │  State      │     │  Account    │
└─────────────┘     └─────────────┘     └─────────────┘
```

## Program Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize` | Create new escrow with terms |
| `deposit` | Maker deposits tokens into vault |
| `exchange` | Taker completes the swap |
| `cancel` | Maker cancels and withdraws |
| `expire` | Anyone can close expired escrow |

## Usage

### Prerequisites

- Rust 1.70+
- Solana CLI 1.17+
- Anchor CLI 0.29+
- Node.js 18+

### Build

```bash
anchor build
```

### Test

```bash
anchor test
```

### Deploy

```bash
# Devnet
anchor deploy --provider.cluster devnet

# Mainnet
anchor deploy --provider.cluster mainnet
```

## State Accounts

### Escrow

```rust
#[account]
pub struct Escrow {
    /// Maker (initiator) of the escrow
    pub maker: Pubkey,
    /// Token mint that maker is offering
    pub mint_a: Pubkey,
    /// Token mint that maker wants to receive
    pub mint_b: Pubkey,
    /// Amount of token A being offered
    pub amount_a: u64,
    /// Amount of token B requested
    pub amount_b: u64,
    /// Escrow vault holding token A
    pub vault: Pubkey,
    /// Expiration timestamp (unix)
    pub expiration: i64,
    /// Current state
    pub state: EscrowState,
    /// Bump seed for PDA
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum EscrowState {
    Initialized,
    Active,
    Completed,
    Cancelled,
    Expired,
}
```

## Client SDK

### TypeScript

```typescript
import { EscrowClient } from './app/src';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';

const connection = new Connection('https://api.devnet.solana.com');
const wallet = Keypair.generate();
const client = new EscrowClient(connection, wallet);

// Create escrow
const escrowId = await client.createEscrow({
  mintA: new PublicKey('...'), // Token you're offering
  mintB: new PublicKey('...'), // Token you want
  amountA: 1000000, // Amount offering (with decimals)
  amountB: 500000,  // Amount requesting
  expirationSeconds: 3600, // 1 hour
});

// Take escrow (as taker)
await client.takeEscrow(escrowId);

// Cancel escrow (as maker)
await client.cancelEscrow(escrowId);
```

## Security Considerations

1. **Atomic Execution**: All transfers happen in a single transaction
2. **PDA Authority**: Vault is controlled by program-derived address
3. **Time Locks**: Escrows can be set to expire
4. **Access Control**: Only maker can cancel, anyone can take
5. **Reentrancy Safe**: CPI guards prevent reentrancy attacks

## Testing

The program includes comprehensive tests:

```
tests/
├── escrow.ts           # Main escrow flow tests
├── edge-cases.ts       # Error handling tests
└── integration.ts      # Full integration tests
```

Run tests:

```bash
# All tests
anchor test

# Specific test file
anchor test --skip-build tests/escrow.ts
```

## Project Structure

```
solana-escrow-program/
├── programs/
│   └── escrow/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs           # Program entry point
│           ├── instructions/    # Instruction handlers
│           ├── state/          # Account structures
│           └── errors.rs       # Custom errors
├── tests/
│   └── escrow.ts
├── app/
│   └── src/
│       └── index.ts            # TypeScript client
├── Anchor.toml
└── README.md
```

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 6000 | InvalidAmount | Amount must be greater than 0 |
| 6001 | InvalidExpiration | Expiration must be in the future |
| 6002 | EscrowNotActive | Escrow is not in active state |
| 6003 | EscrowExpired | Escrow has expired |
| 6004 | UnauthorizedCancellation | Only maker can cancel |
| 6005 | InsufficientFunds | Insufficient token balance |

## Roadmap

- [x] Basic escrow functionality
- [x] Time-locked escrows
- [x] TypeScript client SDK
- [ ] Partial fills
- [ ] Multi-party escrows
- [ ] Fee mechanism
- [ ] Frontend UI

## Contributing

Contributions welcome! Please read the contributing guidelines.

## License

MIT License - see LICENSE for details.

---

Built by [@Sugusdaddy](https://github.com/Sugusdaddy)
