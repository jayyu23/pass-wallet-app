# PASS Wallet HTTP API Documentation

## Overview

The PASS Wallet HTTP API provides RESTful endpoints for managing multiple PASS wallets within the Nitro Enclave. The API supports wallet creation, asset management, subaccount management, deposits, withdrawals, transfers, and signing operations.

## Base URL

The HTTP server runs on port 5000 by default (configurable via `HTTP_PORT` environment variable).

## Authentication

All requests communicate with the Nitro Enclave via VSOCK. The enclave provides cryptographic security and key management.

## Endpoints

### Original KMS Endpoints

#### Generate Ethereum Account
```
POST /generate
```

**Request Body:**
```json
{}
```

**Response:**
```json
{
  "address": "0x1234567890abcdef1234567890abcdef12345678",
  "message": "Account generated and stored in enclave"
}
```

#### List Addresses
```
GET /addresses
```

**Response:**
```json
[
  "0x1234567890abcdef1234567890abcdef12345678",
  "0xabcdef1234567890abcdef1234567890abcdef12"
]
```

#### Sign Message
```
POST /sign
```

**Request Body:**
```json
{
  "address": "0x1234567890abcdef1234567890abcdef12345678",
  "message": "Hello, world!"
}
```

**Response:**
```json
{
  "signature": "0x1234567890abcdef..."
}
```

### PASS Wallet Endpoints

#### Create PASS Wallet
```
POST /pass/wallets
```

**Request Body:**
```json
{
  "name": "My PASS Wallet",
  "owner": "alice"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "name": "My PASS Wallet",
  "owner": "alice",
  "state": {
    "address": "0x1234567890abcdef1234567890abcdef12345678",
    "name": "My PASS Wallet",
    "owner": "alice",
    "nonce": 0,
    "inbox_count": 0,
    "outbox_count": 0,
    "assets_count": 0,
    "subaccounts_count": 0,
    "history_count": 0,
    "created_at": 1698123456
  }
}
```

#### List PASS Wallets
```
GET /pass/wallets
```

**Response:**
```json
{
  "wallets": [
    "0x1234567890abcdef1234567890abcdef12345678",
    "0xabcdef1234567890abcdef1234567890abcdef12"
  ]
}
```

#### Get Wallet State
```
POST /pass/wallets/state
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678"
}
```

**Response:**
```json
{
  "address": "0x1234567890abcdef1234567890abcdef12345678",
  "name": "My PASS Wallet",
  "owner": "alice",
  "nonce": 0,
  "inbox_count": 0,
  "outbox_count": 0,
  "assets_count": 1,
  "subaccounts_count": 2,
  "history_count": 5,
  "created_at": 1698123456
}
```

#### Add Asset
```
POST /pass/wallets/assets
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
          "asset_id": "eth",
  "token_type": "ETH",
  "contract_address": null,
  "token_id": null,
  "symbol": "ETH",
  "name": "Ethereum",
  "decimals": 18
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
          "asset_id": "eth",
  "symbol": "ETH",
  "name": "Ethereum"
}
```

#### Get Assets
```
POST /pass/wallets/assets/list
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "assets": {
    "eth_mainnet": {
      "token_type": "ETH",
      "contract_address": null,
      "token_id": null,
      "symbol": "ETH",
      "name": "Ethereum",
      "decimals": 18,
      "total_balance": 5000000000000000000,
      "subaccount_balances": {
        "main_account": 3000000000000000000,
        "trading_account": 2000000000000000000
      }
    },
            "usdc": {
      "token_type": "ERC20",
      "contract_address": "0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2",
      "token_id": null,
      "symbol": "USDC",
      "name": "USD Coin",
      "decimals": 6,
      "total_balance": 1000000000,
      "subaccount_balances": {
        "main_account": 1000000000
      }
    },
    "nft_collection": {
      "token_type": "ERC721",
      "contract_address": "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d",
      "token_id": "1234",
      "symbol": "BAYC",
      "name": "Bored Ape Yacht Club",
      "decimals": 0,
      "total_balance": 1,
      "subaccount_balances": {
        "collectibles_account": 1
      }
    }
  }
}
```

#### Add Subaccount
```
POST /pass/wallets/subaccounts
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "subaccount_id": "main_account",
  "label": "Main Account",
  "address": "0x1234567890abcdef1234567890abcdef12345678"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "subaccount_id": "main_account",
  "label": "Main Account",
  "address": "0x1234567890abcdef1234567890abcdef12345678"
}
```

#### Inbox Deposit
```
POST /pass/wallets/deposits
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
          "asset_id": "eth",
  "amount": 1000000000000000000,
  "deposit_id": "deposit_12345",
  "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "block_number": "12345678",
  "from_address": "0x9876543210fedcba9876543210fedcba98765432",
  "to_address": "0x1234567890abcdef1234567890abcdef12345678"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
          "asset_id": "eth",
  "amount": 1000000000000000000,
  "deposit_id": "deposit_12345",
  "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
}
```

#### Claim Inbox
```
POST /pass/wallets/claims
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "deposit_id": "deposit_12345",
  "subaccount_id": "main_account"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "deposit_id": "deposit_12345",
  "subaccount_id": "main_account",
  "state": {
    "address": "0x1234567890abcdef1234567890abcdef12345678",
    "name": "My PASS Wallet",
    "owner": "alice",
    "nonce": 0,
    "inbox_count": 0,
    "outbox_count": 0,
    "assets_count": 1,
    "subaccounts_count": 1,
    "history_count": 1,
    "created_at": 1698123456
  }
}
```

#### Internal Transfer
```
POST /pass/wallets/transfers
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
          "asset_id": "eth",
  "amount": 500000000000000000,
  "from_subaccount": "main_account",
  "to_subaccount": "trading_account"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
          "asset_id": "eth",
  "amount": 500000000000000000,
  "from_subaccount": "main_account",
  "to_subaccount": "trading_account",
  "from_balance": 500000000000000000,
  "to_balance": 500000000000000000
}
```

#### Withdraw
```
POST /pass/wallets/withdrawals
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
          "asset_id": "eth",
  "amount": 100000000000000000,
  "subaccount_id": "main_account",
  "destination": "0x9876543210fedcba9876543210fedcba98765432"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
          "asset_id": "eth",
  "amount": 100000000000000000,
  "subaccount_id": "main_account",
  "destination": "0x9876543210fedcba9876543210fedcba98765432",
  "remaining_balance": 400000000000000000
}
```

#### Process Outbox
```
POST /pass/wallets/outbox
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "processed_items": [
    {
              "asset_id": "eth",
      "amount": 100000000000000000,
      "external_destination": "0x9876543210fedcba9876543210fedcba98765432",
      "nonce": 0
    }
  ],
  "count": 1
}
```

#### Get Balance
```
POST /pass/wallets/balance
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "subaccount_id": "main_account",
  "asset_id": "eth_mainnet"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "subaccount_id": "main_account",
          "asset_id": "eth",
  "balance": 500000000000000000
}
```

#### Get Subaccount Balances
```
POST /pass/wallets/balances
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "subaccount_id": "main_account"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "subaccount_id": "main_account",
  "balances": {
                "eth": 500000000000000000,
            "usdc": 1000000000
  }
}
```

#### Sign GSM Message
```
POST /pass/wallets/sign
```

**Request Body:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "domain": "dapp.example.com",
  "message": "Approve transaction #12345"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "signature": "0x1234567890abcdef...",
  "domain": "dapp.example.com",
  "message": "Approve transaction #12345"
}
```

## Error Responses

All endpoints return error responses in the following format:

```json
{
  "error": "Error message description"
}
```

Common HTTP status codes:
- `200 OK`: Success
- `400 Bad Request`: Invalid request format
- `500 Internal Server Error`: Server or enclave communication error

## Example Usage Flow

1. **Create a PASS wallet**
   ```bash
   curl -X POST http://localhost:5000/pass/wallets \
     -H "Content-Type: application/json" \
     -d '{"name": "My Wallet", "owner": "alice"}'
   ```

2. **Add an asset**
   ```bash
   curl -X POST http://localhost:5000/pass/wallets/assets \
     -H "Content-Type: application/json" \
     -d '{
       "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
               "asset_id": "eth",
       "token_type": "ETH",
       "contract_address": null,
       "token_id": null,
       "symbol": "ETH",
       "name": "Ethereum",
       "decimals": 18
     }'
   ```

3. **Add a subaccount**
   ```bash
   curl -X POST http://localhost:5000/pass/wallets/subaccounts \
     -H "Content-Type: application/json" \
     -d '{
       "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
       "subaccount_id": "main_account",
       "label": "Main Account",
       "address": "0x1234567890abcdef1234567890abcdef12345678"
     }'
   ```

4. **List all assets in the wallet**
   ```bash
   curl -X POST http://localhost:5000/pass/wallets/assets/list \
     -H "Content-Type: application/json" \
     -d '{
       "wallet_address": "0x1234567890abcdef1234567890abcdef12345678"
     }'
   ```

5. **Make a deposit**
   ```bash
   curl -X POST http://localhost:5000/pass/wallets/deposits \
     -H "Content-Type: application/json" \
     -d '{
       "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
               "asset_id": "eth",
       "amount": 1000000000000000000,
       "deposit_id": "deposit_12345",
       "transaction_hash": "0xabcdef...",
       "block_number": "12345678",
       "from_address": "0x9876543210fedcba9876543210fedcba98765432",
       "to_address": "0x1234567890abcdef1234567890abcdef12345678"
     }'
   ```

6. **Claim the deposit**
   ```bash
   curl -X POST http://localhost:5000/pass/wallets/claims \
     -H "Content-Type: application/json" \
     -d '{
       "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
       "deposit_id": "deposit_12345",
       "subaccount_id": "main_account"
     }'
   ```

7. **Check balance**
   ```bash
   curl -X POST http://localhost:5000/pass/wallets/balance \
     -H "Content-Type: application/json" \
     -d '{
       "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
       "subaccount_id": "main_account",
       "asset_id": "eth_mainnet"
     }'
   ```

## Token Types

Supported token types:
- `ETH`: Native Ethereum
- `ERC20`: ERC-20 tokens (requires `contract_address`)
- `ERC721`: NFTs (requires `contract_address` and `token_id`)
- `ERC1155`: Multi-token standard (requires `contract_address` and `token_id`)

## Architecture

The HTTP API server acts as a bridge between external clients and the Nitro Enclave:

```
Client → HTTP API → VSOCK → Nitro Enclave → PASS Wallet Logic
```

All cryptographic operations and key management happen within the secure enclave environment.

## Security Considerations

- All private keys are generated and stored within the Nitro Enclave
- VSOCK communication provides secure channel to the enclave
- Wallet addresses are derived from enclave-generated keys
- Transaction signing happens within the enclave using TEE-protected keys
- Complete audit trail maintained through provenance history

### Provenance Log Endpoints

#### Get Full Provenance Log

Get the complete provenance history for a wallet, showing all claims, transfers, and withdrawals.

**Endpoint:** `POST /pass/wallets/provenance`

**Request:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "provenance_records": [
    {
      "operation": {
        "Claim": {
          "asset_id": "eth",
          "amount": 1000000000000000000,
          "deposit_id": "0xabcdef...",
          "subaccount_id": "main"
        }
      },
      "timestamp": 1699200000,
      "block_number": null
    },
    {
      "operation": {
        "Transfer": {
          "asset_id": "eth",
          "amount": 500000000000000000,
          "from_subaccount": "main",
          "to_subaccount": "savings"
        }
      },
      "timestamp": 1699200100,
      "block_number": null
    }
  ]
}
```

#### Get Provenance by Asset

Get provenance history filtered by a specific asset.

**Endpoint:** `POST /pass/wallets/provenance/asset`

**Request:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "asset_id": "eth"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "asset_id": "eth",
  "provenance_records": [
    {
      "operation": {
        "Claim": {
          "asset_id": "eth",
          "amount": 1000000000000000000,
          "deposit_id": "0xabcdef...",
          "subaccount_id": "main"
        }
      },
      "timestamp": 1699200000,
      "block_number": null
    }
  ]
}
```

#### Get Provenance by Subaccount

Get provenance history filtered by a specific subaccount.

**Endpoint:** `POST /pass/wallets/provenance/subaccount`

**Request:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "subaccount_id": "main"
}
```

**Response:**
```json
{
  "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
  "subaccount_id": "main",
  "provenance_records": [
    {
      "operation": {
        "Claim": {
          "asset_id": "eth",
          "amount": 1000000000000000000,
          "deposit_id": "0xabcdef...",
          "subaccount_id": "main"
        }
      },
      "timestamp": 1699200000,
      "block_number": null
    },
    {
      "operation": {
        "Transfer": {
          "asset_id": "eth",
          "amount": 500000000000000000,
          "from_subaccount": "main",
          "to_subaccount": "savings"
        }
      },
      "timestamp": 1699200100,
      "block_number": null
    }
  ]
}
```

## Environment Variables

- `HTTP_PORT`: HTTP server port (default: 5000)
- `ENCLAVE_CID`: Enclave CID for VSOCK communication (default: 19)
- `ENCLAVE_SECRET`: Secret for KMS initialization (default: "test_secret") 