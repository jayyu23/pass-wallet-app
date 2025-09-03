# PassWallet App Prototype

By Jay Yu

[Paper Draft](https://cs191w.stanford.edu/projects/Yu,%20Jay_Systems%20191W.pdf)

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
# Install frontend dependencies
npm install

# Install Python enclave dependencies
cd py-kms-sim
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
cd ..
```

3. **Set up environment variables**

Create `.env`:
```
NEXT_PUBLIC_PROJECT_ID=your_walletconnect_project_id
DATABASE_URL="file:./dev.db"
```

(Optional) Create `py-kms-sim/.env`:
```
ENCLAVE_SECRET=your_secure_enclave_secret
```

4. **Initialize the database**
```bash
npx prisma db push
```

5. **Start the services**

Run backend:

See `nitro-enclave` folder for instructions to setup AWS Nitro server

For Flask simulation of enclave, run:
```bash
npm run enclave-py
```

Run frontend:
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
├── py-kms-sim/          # Python Key Management System Simulation
│   ├── enclave_api.py   # REST API endpoints
│   ├── enclave_kms.py   # Core KMS logic
│   └── venv/            # Python virtual environment
├── nitro-enclave/       # AWS Nitro Enclave implementation - see `nitro-enclave/README.md` for more details
└── prisma/              # Database schema
```

### Key Components

- **Frontend**: Next.js + RainbowKit for wallet connections and UI
- **Backend**: Next.js API routes for business logic
- **Enclave**: AWS Nitro Enclave (nitro-enclave) or Python service simulating a TEE for key operations (py-kms-sim)
- **Database**: SQLite via Prisma for wallet metadata


## Security Considerations

> ⚠️ **Warning**: This implementation is for demonstration purposes only.

For production use:

- Use a TEE with strong hardware security (AWS Nitro Enclaves, Intel SGX) rather than a simulated TEE.
- Implement key rotation and secure backup procedures
- Add comprehensive access controls and audit logging
- Use secure channels between components
- Regular security audits

## License

This project is licensed under the MIT License.

## Acknowledgments

This project is inspired by the [Liquefaction](https://github.com/key-encumbrance/liquefaction) paper by IC3 researchers James Austgen, Andrés Fábrega, Mahimna Kelkar, Dani Vilardell, Sarah Allen, Kushal Babel, Jay Yu, and Ari Juels.

Built with:
- [RainbowKit](https://rainbowkit.com) - Wallet connection UI
- [wagmi](https://wagmi.sh) - React Hooks for Ethereum
- [Next.js](https://nextjs.org) - React Framework
- [WalletConnect v2](https://walletconnect.com) - Web3 Messaging Protocol
- [eth-account](https://github.com/ethereum/eth-account) - Ethereum Key Management
