# AWS Nitro Enclave with HTTP-Vsock proxy server

TEE Backend for PASS Wallet, written for AWS Nitro Enclave architecture.

- `lib.rs` contains entrypoint code for default Vsock client/server.
- `http_main` and `http_client` contain code for the HTTP to Vsock proxy server.
- Start HTTP proxy server: `cargo run --bin http-server`
- Proxy Server created using Axum
- Use JSON strings to parse in commands
- Default Nitro Vsock example documentation and deployment in [NITRO_VSOCK.md](NITRO_VSOCK.md)

# Quick Start

## Step 1: Begin Nitro Enclave Server

Create EIF Image and Deploy to Enclave
```bash
make enclave
```

Deploy EIF Image to Enclave, get Enclave ID ("i-xxxx")
```
nitro-cli run-enclave --eif-path vsock_sample_server.eif --cpu-count 2 --memory 256 --debug-mode
```

Check Enclave Console
```
nitro-cli console --enclave-id <ENCLAVE_ID>
```

## Step 2 (Optional): Launch Test Client Code

Client code in `lib.rs` will send a single keygen command
```bash
make run-client
```

## Step 3: Launch HTTP Server
```bash
export ENCLAVE_CID=16 # Replace with actual Enclave CID
cargo run --bin http-server
```

HTTP server provides these endpoints:
POST /generate - Generate a new wallet address
GET /addresses - List all stored addresses
POST /sign - Sign a message with a specific address

```bash
# Generate a new wallet
curl -X POST http://localhost:5000/generate \
-H "Content-Type: application/json" \
  -d '{}'

# List addresses
curl http://localhost:5000/addresses

# Sign a message
curl -X POST http://localhost:5000/sign \
  -H "Content-Type: application/json" \
  -d '{"address": "0x...", "message": "Hello World"}'
```

## View and Stop Enclave
```bash
nitro-cli describe-enclaves
nitro-cli console --enclave-id $ENCLAVE_ID
nitro-cli terminate-enclave --enclave-id $ENCLAVE_ID
```


# Architecture Flow

```
HTTP Client → HTTP Server (Port 5000) → Vsock → Enclave Server (Port 7777)
     ↑                                                      ↓
HTTP Response ← HTTP Server ← Vsock ← Enclave Response
```
