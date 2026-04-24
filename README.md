# PassWallet App Prototype

## Overview
PassWallet is a decentralized wallet application that enables secure key management through a simulated Trusted Execution Environment (TEE). It combines Next.js, RainbowKit, and WalletConnect for the frontend with a AWS Nitro Enclave TEE for key management and signing. This is a demo application based on recent work on key encumbrance techniques and TEE-based wallet platforms such as [Liquefaction](https://github.com/key-encumbrance/liquefaction).

## Features

- **Multi-Account Support**: Create and manage multiple Ethereum accounts
- **WalletConnect Integration**: Connect and interact with dApps using WalletConnect v2
- **Message Signing**: Sign messages securely through the enclave
- **Transaction History**: View transaction history and signed message records
- **Asset Management**: View and transfer assets (ETH, USDC, etc.)
- **Secure Key Management**: Key encumbrance takes place inside of an AWS Nitro Enclave, ensuring that all signing logic is handled by pre-defined rules and cannot be overwritten arbitrarily by account owner. See `nitro-enclave` folder for more details on server implementation. We also provide a Python simulation of the enclave in `py-kms-sim` folder for API testing.

## Quick Start

1. **Clone the repository**
```bash
git clone https://github.com/jayyu23/pass-wallet-app
```

2. **Install dependencies**
```bash
npm install
```

3. Run backend:

See `nitro-enclave` folder for instructions to setup AWS Nitro server

4. Run frontend:
```bash
npm run dev
```

For server deployment, see `Makefile` for deployment commands.

Visit `http://localhost:3000` to use the application.

## API Documentation

For detailed HTTP API documentation, see [`nitro-enclave/PASS_WALLET_API.md`](nitro-enclave/PASS_WALLET_API.md). The API provides RESTful endpoints for:

- **Wallet Management**: Create and list PASS wallets
- **Asset Management**: Add assets and retrieve all assets in the ledger
- **Subaccount Management**: Create and manage wallet subaccounts
- **Deposit/Withdrawal Operations**: Handle external deposits and withdrawals
- **Internal Transfers**: Transfer assets between subaccounts
- **Balance Queries**: Check individual and subaccount balances
- **Message Signing**: Sign messages using wallet keys

The HTTP server runs on port 5000 by default and communicates with the Nitro Enclave via VSOCK for secure operations.

## Development

### Architecture

```
├── src/                  # Frontend application
│   ├── components/       # React components
│   ├── pages/           # Next.js pages & API routes
│   ├── styles/          # CSS modules
│   └── types/           # TypeScript definitions
├── nitro-enclave/       # AWS Nitro Enclave implementation - see `nitro-enclave/README.md` for more details
```

### Key Components

- **Frontend**: Next.js + RainbowKit for wallet connections and UI
- **Backend**: Next.js API routes for business logic
- **Enclave**: AWS Nitro Enclave (nitro-enclave) or Python service simulating a TEE for key operations (py-kms-sim)
- **Database**: SQLite via Prisma for wallet metadata