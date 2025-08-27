use axum::{
    extract::Json,
    http::{Method, StatusCode},
    response::Json as JsonResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use std::convert::TryInto;
use crate::{vsock_connect, protocol_helpers::{send_loop, send_u64, recv_loop, recv_u64}};
use std::os::unix::io::AsRawFd;
use crate::server_logic::Response;

const BUF_MAX_LEN: usize = 8192;

// Existing request/response structures
#[derive(Deserialize)]
struct SignRequest {
    address: String,
    message: String,
}

#[derive(Serialize)]
struct GenerateResponse {
    address: String,
    message: String,
}

#[derive(Serialize)]
struct SignResponse {
    signature: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// PASS Wallet request/response structures
#[derive(Deserialize)]
struct CreatePassWalletRequest {
    name: String,
    owner: String,
}

#[derive(Deserialize)]
struct AddAssetRequest {
    wallet_address: String,
    asset_id: String,
    token_type: String,
    contract_address: Option<String>,
    token_id: Option<String>,
    symbol: String,
    name: String,
    decimals: u32,
}

#[derive(Deserialize)]
struct AddSubaccountRequest {
    wallet_address: String,
    subaccount_id: String,
    label: String,
    address: String,
}

#[derive(Deserialize)]
struct InboxDepositRequest {
    wallet_address: String,
    asset_id: String,
    amount: u64,
    deposit_id: String,
    transaction_hash: String,
    block_number: String,
    from_address: String,
    to_address: String,
}

#[derive(Deserialize)]
struct ClaimInboxRequest {
    wallet_address: String,
    deposit_id: String,
    subaccount_id: String,
}

#[derive(Deserialize)]
struct InternalTransferRequest {
    wallet_address: String,
    asset_id: String,
    amount: u64,
    from_subaccount: String,
    to_subaccount: String,
}

#[derive(Deserialize)]
struct WithdrawRequest {
    wallet_address: String,
    asset_id: String,
    amount: u64,
    subaccount_id: String,
    destination: String,
}

#[derive(Deserialize)]
struct GetBalanceRequest {
    wallet_address: String,
    subaccount_id: String,
    asset_id: String,
}

#[derive(Deserialize)]
struct GetSubaccountBalancesRequest {
    wallet_address: String,
    subaccount_id: String,
}

#[derive(Deserialize)]
struct SignGSMRequest {
    wallet_address: String,
    domain: String,
    message: String,
}

#[derive(Deserialize)]
struct ProcessOutboxRequest {
    wallet_address: String,
}

#[derive(Deserialize)]
struct GetWalletStateRequest {
    wallet_address: String,
}

#[derive(Deserialize)]
struct GetAssetsRequest {
    wallet_address: String,
}

#[derive(Deserialize)]
struct GetProvenanceLogRequest {
    wallet_address: String,
}

#[derive(Deserialize)]
struct GetProvenanceByAssetRequest {
    wallet_address: String,
    asset_id: String,
}

#[derive(Deserialize)]
struct GetProvenanceBySubaccountRequest {
    wallet_address: String,
    subaccount_id: String,
}

// Withdrawal request structures
#[derive(Deserialize)]
struct WithdrawToExternalRequest {
    wallet_address: String,
    subaccount_id: String,
    asset_id: String,
    amount: u64,
    destination: String,
    gas_price: Option<u64>,
    gas_limit: Option<u64>,
    chain_id: u64,
    override_nonce: Option<u64>,
}

#[derive(Deserialize)]
struct RemoveFromOutboxRequest {
    nonce: u64,
}

// Command: Generate account
async fn generate_handler(Json(_args): Json<Option<serde_json::Value>>) -> Result<JsonResponse<GenerateResponse>, (StatusCode, JsonResponse<ErrorResponse>)> {
    println!("Generating account");
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "Keygen": null
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if let Some(data) = response.data {
                if let Some(address) = data.get("address").and_then(|v| v.as_str()) {
                    return Ok(JsonResponse(GenerateResponse {
                        address: address.to_string(),
                        message: "Account generated and stored in enclave".to_string(),
                    }));
                }
            }
            Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                error: "Failed to generate account".to_string(),
            })))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Command: List addresses
async fn addresses_handler() -> Result<JsonResponse<Vec<String>>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "List": null
    });
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if let Some(data) = response.data {

                if let Some(addresses) = data.as_array() {
                    println!("Addresses: {:?}", addresses);
                    let address_list: Vec<String> = addresses.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    return Ok(JsonResponse(address_list));
                }
            }
            Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                error: "Failed to list addresses".to_string(),
            })))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

async fn sign_handler(Json(request): Json<SignRequest>) -> Result<JsonResponse<SignResponse>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "Sign": {
            "address": request.address,
            "message": request.message
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if let Some(data) = response.data {
                if let Some(signature) = data.get("signature").and_then(|v| v.as_str()) {
                    return Ok(JsonResponse(SignResponse {
                        signature: signature.to_string(),
                    }));
                }
            }
            Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                error: "Failed to sign message".to_string(),
            })))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// ------------ PASS Wallet HTTP handlers ------------

// Create PASS wallet
async fn create_pass_wallet_handler(Json(request): Json<CreatePassWalletRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "CreatePassWallet": {
            "name": request.name,
            "owner": request.owner
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// List PASS wallets
async fn list_pass_wallets_handler() -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "ListPassWallets": null
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Get wallet state
async fn get_wallet_state_handler(Json(request): Json<GetWalletStateRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "GetPassWalletState": {
            "wallet_address": request.wallet_address
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Get assets from wallet ledger
async fn get_assets_handler(Json(request): Json<GetAssetsRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "GetAssets": {
            "wallet_address": request.wallet_address
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Get full provenance log from wallet
async fn get_provenance_log_handler(Json(request): Json<GetProvenanceLogRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "GetProvenanceLog": {
            "wallet_address": request.wallet_address
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Get provenance log filtered by asset
async fn get_provenance_by_asset_handler(Json(request): Json<GetProvenanceByAssetRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "GetProvenanceByAsset": {
            "wallet_address": request.wallet_address,
            "asset_id": request.asset_id
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Get provenance log filtered by subaccount
async fn get_provenance_by_subaccount_handler(Json(request): Json<GetProvenanceBySubaccountRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "GetProvenanceBySubaccount": {
            "wallet_address": request.wallet_address,
            "subaccount_id": request.subaccount_id
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Add asset
async fn add_asset_handler(Json(request): Json<AddAssetRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "AddAsset": {
            "wallet_address": request.wallet_address,
            "asset_id": request.asset_id,
            "token_type": request.token_type,
            "contract_address": request.contract_address,
            "token_id": request.token_id,
            "symbol": request.symbol,
            "name": request.name,
            "decimals": request.decimals
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Add subaccount
async fn add_subaccount_handler(Json(request): Json<AddSubaccountRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "AddSubaccount": {
            "wallet_address": request.wallet_address,
            "subaccount_id": request.subaccount_id,
            "label": request.label,
            "address": request.address
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Inbox deposit
async fn inbox_deposit_handler(Json(request): Json<InboxDepositRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "InboxDeposit": {
            "wallet_address": request.wallet_address,
            "asset_id": request.asset_id,
            "amount": request.amount,
            "deposit_id": request.deposit_id,
            "transaction_hash": request.transaction_hash,
            "block_number": request.block_number,
            "from_address": request.from_address,
            "to_address": request.to_address
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Claim inbox
async fn claim_inbox_handler(Json(request): Json<ClaimInboxRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "ClaimInbox": {
            "wallet_address": request.wallet_address,
            "deposit_id": request.deposit_id,
            "subaccount_id": request.subaccount_id
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Internal transfer
async fn internal_transfer_handler(Json(request): Json<InternalTransferRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "InternalTransfer": {
            "wallet_address": request.wallet_address,
            "asset_id": request.asset_id,
            "amount": request.amount,
            "from_subaccount": request.from_subaccount,
            "to_subaccount": request.to_subaccount
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Withdraw
async fn withdraw_handler(Json(request): Json<WithdrawRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "Withdraw": {
            "wallet_address": request.wallet_address,
            "asset_id": request.asset_id,
            "amount": request.amount,
            "subaccount_id": request.subaccount_id,
            "destination": request.destination
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Process outbox
async fn process_outbox_handler(Json(request): Json<ProcessOutboxRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "ProcessOutbox": {
            "wallet_address": request.wallet_address
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Get balance
async fn get_balance_handler(Json(request): Json<GetBalanceRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "GetBalance": {
            "wallet_address": request.wallet_address,
            "subaccount_id": request.subaccount_id,
            "asset_id": request.asset_id
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Get subaccount balances
async fn get_subaccount_balances_handler(Json(request): Json<GetSubaccountBalancesRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "GetSubaccountBalances": {
            "wallet_address": request.wallet_address,
            "subaccount_id": request.subaccount_id
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Sign GSM
async fn sign_gsm_handler(Json(request): Json<SignGSMRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "SignGSM": {
            "wallet_address": request.wallet_address,
            "domain": request.domain,
            "message": request.message
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Withdraw to external address handler
async fn withdraw_to_external_handler(Json(request): Json<WithdrawToExternalRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "WithdrawToExternal": {
            "wallet_address": request.wallet_address,
            "subaccount_id": request.subaccount_id,
            "asset_id": request.asset_id,
            "amount": request.amount,
            "destination": request.destination,
            "gas_price": request.gas_price,
            "gas_limit": request.gas_limit,
            "chain_id": request.chain_id,
            "override_nonce": request.override_nonce
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Get outbox queue handler
async fn get_outbox_queue_handler() -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "GetOutboxQueue": {}
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Remove from outbox handler
async fn remove_from_outbox_handler(Json(request): Json<RemoveFromOutboxRequest>) -> Result<JsonResponse<serde_json::Value>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let cid = std::env::var("ENCLAVE_CID").unwrap_or_else(|_| "19".to_string()).parse::<u32>()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: "Invalid ENCLAVE_CID".to_string(),
        })))?;
    
    let port = 7777u32;
    
    let command = serde_json::json!({
        "RemoveFromOutbox": {
            "nonce": request.nonce
        }
    });
    
    match send_command_to_enclave(cid, port, &command.to_string()).await {
        Ok(response) => {
            if response.success {
                Ok(JsonResponse(response.data.unwrap_or(serde_json::json!({}))))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
                    error: response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(ErrorResponse {
            error: format!("Enclave communication error: {}", e),
        })))
    }
}

// Send command to enclave and receive response
async fn send_command_to_enclave(cid: u32, port: u32, command: &str) -> Result<Response, String> {
    let vsocket = vsock_connect(cid, port)?;
    let fd = vsocket.as_raw_fd();

    println!("Sending command to enclave: {}", command);

    // Send command to enclave
    let buf = command.as_bytes();
    let len: u64 = buf.len().try_into().map_err(|err| format!("{:?}", err))?;
    send_u64(fd, len)?;
    send_loop(fd, buf, len)?;

    // Receive response from enclave
    let mut response_buf = [0u8; BUF_MAX_LEN];
    let response_len = recv_u64(fd)?;
    recv_loop(fd, &mut response_buf, response_len)?;
    
    let response_str = String::from_utf8(response_buf[..response_len as usize].to_vec())
        .map_err(|err| format!("The received bytes are not UTF-8: {:?}", err))?;
    
    let response: Response = serde_json::from_str(&response_str)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(response)
}

pub async fn run_http_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        // Original KMS endpoints
        .route("/generate", post(generate_handler))
        .route("/addresses", get(addresses_handler))
        .route("/sign", post(sign_handler))
        
        // PASS Wallet endpoints
        .route("/pass/wallets", post(create_pass_wallet_handler))
        .route("/pass/wallets", get(list_pass_wallets_handler))
        .route("/pass/wallets/state", post(get_wallet_state_handler))
        .route("/pass/wallets/assets", post(add_asset_handler))
        .route("/pass/wallets/assets/list", post(get_assets_handler))
        .route("/pass/wallets/subaccounts", post(add_subaccount_handler))
        .route("/pass/wallets/deposits", post(inbox_deposit_handler))
        .route("/pass/wallets/claims", post(claim_inbox_handler))
        .route("/pass/wallets/transfers", post(internal_transfer_handler))
        .route("/pass/wallets/withdrawals", post(withdraw_handler))
        .route("/pass/wallets/withdrawals/external", post(withdraw_to_external_handler))
        .route("/pass/wallets/outbox", get(get_outbox_queue_handler))
        .route("/pass/wallets/outbox/remove", post(remove_from_outbox_handler))
        .route("/pass/wallets/balance", post(get_balance_handler))
        .route("/pass/wallets/balances", post(get_subaccount_balances_handler))
        .route("/pass/wallets/sign", post(sign_gsm_handler))
        
        // Provenance endpoints
        .route("/pass/wallets/provenance", post(get_provenance_log_handler))
        .route("/pass/wallets/provenance/asset", post(get_provenance_by_asset_handler))
        .route("/pass/wallets/provenance/subaccount", post(get_provenance_by_subaccount_handler))
        
        .layer(cors);

    println!("HTTP server listening on port {}", port);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
} 