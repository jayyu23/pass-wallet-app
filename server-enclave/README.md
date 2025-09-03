# Server Enclave

This is a standalone server implementation of the Pass Wallet enclave, designed to run on normal servers instead of AWS Nitro Enclaves.

## Features

- **Standalone Operation**: Runs as a normal HTTP server without requiring AWS Nitro Enclave infrastructure
- **Merged Architecture**: Combines HTTP client and server functionality into a single binary
- **Same API**: Maintains full compatibility with the original nitro-enclave API
- **Key Management**: Secure key generation, storage, and signing using AES-GCM encryption
- **Pass Wallet Support**: Complete PASS wallet functionality including assets, subaccounts, and transactions

## Quick Start

### Using npm (recommended)

```bash
# Start the server (development mode)
npm run server-enclave

# Start the server (release mode)
npm run server-enclave-release
```

### Using Cargo directly

```bash
cd server-enclave

# Development mode
cargo run

# Release mode
cargo run --release
```

### Using Docker

```bash
cd server-enclave

# Build the Docker image
docker build -t server-enclave .

# Run the container
docker run -p 5001:5001 server-enclave
```

## Configuration

### Environment Variables

- `HTTP_PORT`: Port to run the server on (default: 5001)
- `ENCLAVE_SECRET`: Secret key for encrypting stored private keys (default: "test_secret")

### Example

```bash
export HTTP_PORT=8080
export ENCLAVE_SECRET=your_secure_secret_here
npm run server-enclave
```

## API Endpoints

The server provides the same API as the original nitro-enclave:

### KMS Endpoints
- `POST /generate` - Generate new Ethereum account
- `GET /addresses` - List all generated addresses
- `POST /sign` - Sign a message

### PASS Wallet Endpoints
- `POST /pass/wallets` - Create PASS wallet
- `GET /pass/wallets` - List PASS wallets
- `POST /pass/wallets/state` - Get wallet state
- `POST /pass/wallets/assets` - Add asset to wallet
- `POST /pass/wallets/assets/list` - List wallet assets
- `POST /pass/wallets/subaccounts` - Add subaccount
- `POST /pass/wallets/deposits` - Process inbox deposit
- `POST /pass/wallets/claims` - Claim from inbox
- `POST /pass/wallets/transfers` - Internal transfer
- `POST /pass/wallets/withdrawals` - Withdraw assets
- `POST /pass/wallets/withdrawals/external` - Withdraw to external address
- `GET /pass/wallets/outbox` - Get outbox queue
- `POST /pass/wallets/outbox/remove` - Remove from outbox
- `POST /pass/wallets/balance` - Get balance
- `POST /pass/wallets/balances` - Get subaccount balances
- `POST /pass/wallets/sign` - Sign GSM message

### Provenance Endpoints
- `POST /pass/wallets/provenance` - Get full provenance log
- `POST /pass/wallets/provenance/asset` - Get provenance by asset
- `POST /pass/wallets/provenance/subaccount` - Get provenance by subaccount

## Key Differences from Nitro Enclave

1. **No VSOCK**: Uses standard TCP networking instead of VSOCK communication
2. **Simplified Architecture**: Single binary instead of separate client/server binaries
3. **Standard Deployment**: Can be deployed on any server, not just AWS Nitro instances
4. **Same Security Model**: Still uses AES-GCM encryption for key storage

## Security Considerations

- **Key Storage**: Private keys are encrypted using AES-GCM before storage in memory
- **Secret Management**: Use a strong `ENCLAVE_SECRET` in production
- **Network Security**: Deploy behind appropriate firewalls and load balancers
- **Monitoring**: Consider adding monitoring and logging as needed

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Adding Features

The codebase is organized into modules:
- `key_manager.rs` - Ethereum key management and cryptographic operations
- `pass_logic.rs` - PASS wallet business logic
- `server_logic.rs` - Command parsing and response handling
- `lib.rs` - HTTP server and API endpoints
- `main.rs` - Application entry point