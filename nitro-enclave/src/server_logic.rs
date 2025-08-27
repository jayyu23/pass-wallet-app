// Backend logic for the server - parse commands and call the appropriate functions

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use anyhow::Result;

use crate::key_manager::EnclaveKMS;
use crate::pass_logic::{PassWalletManager, Asset, Subaccount, Deposit, TokenType};



// Global KMS instance
lazy_static::lazy_static! {
    static ref KMS: Arc<Mutex<EnclaveKMS>> = {
        let secret = std::env::var("ENCLAVE_SECRET").unwrap_or_else(|_| "test_secret".to_string());
        Arc::new(Mutex::new(EnclaveKMS::new(&secret).expect("Failed to initialize KMS")))
    };
}

// Global PASS Wallet Manager instance
lazy_static::lazy_static! {
    static ref PASS_WALLET_MANAGER: PassWalletManager = {
        PassWalletManager::new(KMS.clone())
    };
}

#[derive(Serialize, Deserialize)]
pub enum Command {
    // Existing KMS commands
    Keygen,
    Sign { address: String, message: String },
    List,
    Verify { address: String, message: String, signature: String },
    
    // PASS Wallet commands
    CreatePassWallet { name: String, owner: String },
    ListPassWallets,
    GetPassWalletState { wallet_address: String },
    
    // Asset management
    AddAsset { 
        wallet_address: String, 
        asset_id: String, 
        token_type: String,
        contract_address: Option<String>,
        token_id: Option<String>,
        symbol: String,
        name: String,
        decimals: u32,
    },
    
    // Subaccount management
    AddSubaccount { 
        wallet_address: String, 
        subaccount_id: String,
        label: String,
        address: String,
    },
    
    // Deposit and withdrawal operations
    InboxDeposit { 
        wallet_address: String,
        asset_id: String,
        amount: u64,
        deposit_id: String,
        transaction_hash: String,
        block_number: String,
        from_address: String,
        to_address: String,
    },
    ClaimInbox { 
        wallet_address: String,
        deposit_id: String,
        subaccount_id: String,
    },
    
    // Transfer operations
    InternalTransfer { 
        wallet_address: String,
        asset_id: String,
        amount: u64,
        from_subaccount: String,
        to_subaccount: String,
    },
    Withdraw { 
        wallet_address: String,
        asset_id: String,
        amount: u64,
        subaccount_id: String,
        destination: String,
    },
    
    // Utility operations
    ProcessOutbox { wallet_address: String },
    GetBalance { 
        wallet_address: String,
        subaccount_id: String,
        asset_id: String,
    },
    GetSubaccountBalances { 
        wallet_address: String,
        subaccount_id: String,
    },
    
    // Signing operations
    SignGSM { 
        wallet_address: String,
        domain: String,
        message: String,
    },
    
    // Asset operations
    GetAssets { 
        wallet_address: String,
    },
    
    // Provenance operations
    GetProvenanceLog { 
        wallet_address: String,
    },
    GetProvenanceByAsset { 
        wallet_address: String,
        asset_id: String,
    },
    GetProvenanceBySubaccount { 
        wallet_address: String,
        subaccount_id: String,
    },
    // Withdrawal operations
    WithdrawToExternal {
        wallet_address: String,
        subaccount_id: String,
        asset_id: String,
        amount: u64,
        destination: String,
        gas_price: Option<u64>,
        gas_limit: Option<u64>,
        chain_id: u64,
        override_nonce: Option<u64>,
    },
    GetOutboxQueue,
    RemoveFromOutbox {
        nonce: u64,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub fn parse_command(command: &str) -> Result<Response, String> {
    let command_data: Command = serde_json::from_str(command)
        .map_err(|e| format!("Failed to parse command: {}", e))?;
    
    match command_data {
        // Existing KMS commands
        Command::Keygen => {
            let mut kms = KMS.lock().unwrap();
            match kms.handle_keygen() {
                Ok(account) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "address": account.address,
                        "private_key": account.private_key
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to generate account: {}", e)),
                }),
            }
        }
        
        Command::Sign { address, message } => {
            let kms = KMS.lock().unwrap();
            match kms.sign_message(&message, &address) {
                Ok(Some(signature)) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "signature": signature,
                        "message": message,
                        "address": address
                    })),
                    error: None,
                }),
                Ok(None) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some("Address not found or signing failed".to_string()),
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Signing error: {}", e)),
                }),
            }
        }
        
        Command::List => {
            let kms = KMS.lock().unwrap();
            match kms.list_addresses() {
                Ok(addresses) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!(addresses)),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to list addresses: {}", e)),
                }),
            }
        }
        
        Command::Verify { address, message, signature } => {
            let kms = KMS.lock().unwrap();
            match kms.verify_message(&message, &signature, &address) {
                Ok(is_valid) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "valid": is_valid,
                        "address": address,
                        "message": message
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Verification error: {}", e)),
                }),
            }
        }
        
        // PASS Wallet commands
        Command::CreatePassWallet { name, owner } => {
            match PASS_WALLET_MANAGER.create_wallet(name.clone(), owner.clone()) {
                Ok(wallet_address) => {
                    match PASS_WALLET_MANAGER.get_wallet_state(&wallet_address) {
                        Ok(state) => Ok(Response {
                            success: true,
                            data: Some(serde_json::json!({
                                "wallet_address": wallet_address,
                                "name": name,
                                "owner": owner,
                                "state": state
                            })),
                            error: None,
                        }),
                        Err(e) => Ok(Response {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to get wallet state: {}", e)),
                        }),
                    }
                }
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to create PASS wallet: {}", e)),
                }),
            }
        }
        
        Command::ListPassWallets => {
            let wallets = PASS_WALLET_MANAGER.list_wallets();
            Ok(Response {
                success: true,
                data: Some(serde_json::json!({
                    "wallets": wallets
                })),
                error: None,
            })
        }
        
        Command::GetPassWalletState { wallet_address } => {
            match PASS_WALLET_MANAGER.get_wallet_state(&wallet_address) {
                Ok(state) => Ok(Response {
                    success: true,
                    data: Some(state),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get wallet state: {}", e)),
                }),
            }
        }
        
        Command::AddAsset { 
            wallet_address, 
            asset_id, 
            token_type,
            contract_address,
            token_id,
            symbol,
            name,
            decimals,
        } => {
            let token_type_enum = match token_type.as_str() {
                "ETH" => TokenType::ETH,
                "ERC20" => TokenType::ERC20,
                "ERC721" => TokenType::ERC721,
                "ERC1155" => TokenType::ERC1155,
                _ => return Ok(Response {
                    success: false,
                    data: None,
                    error: Some("Invalid token type".to_string()),
                }),
            };
            
            let asset = Asset {
                token_type: token_type_enum,
                contract_address,
                token_id,
                symbol: symbol.clone(),
                name: name.clone(),
                decimals,
            };
            
            match PASS_WALLET_MANAGER.add_asset(&wallet_address, asset_id.clone(), asset) {
                Ok(()) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "wallet_address": wallet_address,
                        "asset_id": asset_id,
                        "symbol": symbol,
                        "name": name
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to add asset: {}", e)),
                }),
            }
        }
        
        Command::AddSubaccount { 
            wallet_address, 
            subaccount_id,
            label,
            address,
        } => {
            let subaccount = Subaccount {
                id: subaccount_id.clone(),
                label: label.clone(),
                address: address.clone(),
            };
            
            match PASS_WALLET_MANAGER.add_subaccount(&wallet_address, subaccount) {
                Ok(()) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "wallet_address": wallet_address,
                        "subaccount_id": subaccount_id,
                        "label": label,
                        "address": address
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to add subaccount: {}", e)),
                }),
            }
        }
        
        Command::InboxDeposit { 
            wallet_address,
            asset_id,
            amount,
            deposit_id,
            transaction_hash,
            block_number,
            from_address,
            to_address,
        } => {
            let deposit = Deposit {
                asset_id: asset_id.clone(),
                amount,
                deposit_id: deposit_id.clone(),
                transaction_hash: transaction_hash.clone(),
                block_number: block_number.clone(),
                from_address: from_address.clone(),
                to_address: to_address.clone(),
            };
            
            match PASS_WALLET_MANAGER.inbox_deposit(&wallet_address, deposit) {
                Ok(()) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "wallet_address": wallet_address,
                        "asset_id": asset_id,
                        "amount": amount,
                        "deposit_id": deposit_id,
                        "transaction_hash": transaction_hash
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to process inbox deposit: {}", e)),
                }),
            }
        }
        
        Command::ClaimInbox { 
            wallet_address,
            deposit_id,
            subaccount_id,
        } => {
            match PASS_WALLET_MANAGER.claim_inbox(&wallet_address, &deposit_id, &subaccount_id) {
                Ok(()) => {
                    // Get updated balance
                    match PASS_WALLET_MANAGER.get_wallet_state(&wallet_address) {
                        Ok(state) => Ok(Response {
                            success: true,
                            data: Some(serde_json::json!({
                                "wallet_address": wallet_address,
                                "deposit_id": deposit_id,
                                "subaccount_id": subaccount_id,
                                "state": state
                            })),
                            error: None,
                        }),
                        Err(e) => Ok(Response {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to get updated state: {}", e)),
                        }),
                    }
                }
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to claim inbox: {}", e)),
                }),
            }
        }
        
        Command::InternalTransfer { 
            wallet_address,
            asset_id,
            amount,
            from_subaccount,
            to_subaccount,
        } => {
            match PASS_WALLET_MANAGER.internal_transfer(&wallet_address, &asset_id, amount, &from_subaccount, &to_subaccount) {
                Ok(()) => {
                    // Get updated balances
                    let from_balance = PASS_WALLET_MANAGER.get_balance(&wallet_address, &from_subaccount, &asset_id)
                        .unwrap_or(0);
                    let to_balance = PASS_WALLET_MANAGER.get_balance(&wallet_address, &to_subaccount, &asset_id)
                        .unwrap_or(0);
                    
                    Ok(Response {
                        success: true,
                        data: Some(serde_json::json!({
                            "wallet_address": wallet_address,
                            "asset_id": asset_id,
                            "amount": amount,
                            "from_subaccount": from_subaccount,
                            "to_subaccount": to_subaccount,
                            "from_balance": from_balance,
                            "to_balance": to_balance
                        })),
                        error: None,
                    })
                }
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to process internal transfer: {}", e)),
                }),
            }
        }
        
        Command::Withdraw { 
            wallet_address,
            asset_id,
            amount,
            subaccount_id,
            destination,
        } => {
            match PASS_WALLET_MANAGER.withdraw(&wallet_address, &asset_id, amount, &subaccount_id, &destination) {
                Ok(()) => {
                    let remaining_balance = PASS_WALLET_MANAGER.get_balance(&wallet_address, &subaccount_id, &asset_id)
                        .unwrap_or(0);
                    
                    Ok(Response {
                        success: true,
                        data: Some(serde_json::json!({
                            "wallet_address": wallet_address,
                            "asset_id": asset_id,
                            "amount": amount,
                            "subaccount_id": subaccount_id,
                            "destination": destination,
                            "remaining_balance": remaining_balance
                        })),
                        error: None,
                    })
                }
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to process withdrawal: {}", e)),
                }),
            }
        }
        
        Command::ProcessOutbox { wallet_address } => {
            match PASS_WALLET_MANAGER.process_outbox(&wallet_address) {
                Ok(processed_items) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "wallet_address": wallet_address,
                        "processed_items": processed_items,
                        "count": processed_items.len()
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to process outbox: {}", e)),
                }),
            }
        }
        
        Command::GetBalance { 
            wallet_address,
            subaccount_id,
            asset_id,
        } => {
            match PASS_WALLET_MANAGER.get_balance(&wallet_address, &subaccount_id, &asset_id) {
                Ok(balance) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "wallet_address": wallet_address,
                        "subaccount_id": subaccount_id,
                        "asset_id": asset_id,
                        "balance": balance
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get balance: {}", e)),
                }),
            }
        }
        
        Command::GetSubaccountBalances { 
            wallet_address,
            subaccount_id,
        } => {
            match PASS_WALLET_MANAGER.get_subaccount_balances(&wallet_address, &subaccount_id) {
                Ok(balances) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "wallet_address": wallet_address,
                        "subaccount_id": subaccount_id,
                        "balances": balances
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get subaccount balances: {}", e)),
                }),
            }
        }
        
        Command::SignGSM { 
            wallet_address,
            domain,
            message,
        } => {
            match PASS_WALLET_MANAGER.sign_message(&wallet_address, &domain, &message) {
                Ok(signature) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "wallet_address": wallet_address,
                        "signature": signature,
                        "domain": domain,
                        "message": message
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to sign GSM: {}", e)),
                }),
            }
        }

        Command::GetAssets { wallet_address } => {
            match PASS_WALLET_MANAGER.get_wallet_assets(&wallet_address) {
                Ok(assets_with_balances) => Ok(Response {
                    success: true,
                    data: Some(assets_with_balances),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get assets: {}", e)),
                }),
            }
        }

        Command::GetProvenanceLog { wallet_address } => {
            match PASS_WALLET_MANAGER.get_provenance_log(&wallet_address) {
                Ok(provenance_log) => Ok(Response {
                    success: true,
                    data: Some(provenance_log),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get provenance log: {}", e)),
                }),
            }
        }

        Command::GetProvenanceByAsset { wallet_address, asset_id } => {
            match PASS_WALLET_MANAGER.get_provenance_by_asset(&wallet_address, &asset_id) {
                Ok(provenance_log) => Ok(Response {
                    success: true,
                    data: Some(provenance_log),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get provenance log by asset: {}", e)),
                }),
            }
        }

        Command::GetProvenanceBySubaccount { wallet_address, subaccount_id } => {
            match PASS_WALLET_MANAGER.get_provenance_by_subaccount(&wallet_address, &subaccount_id) {
                Ok(provenance_log) => Ok(Response {
                    success: true,
                    data: Some(provenance_log),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get provenance log by subaccount: {}", e)),
                }),
            }
        }

        // Withdrawal operations
        Command::WithdrawToExternal { 
            wallet_address, 
            subaccount_id, 
            asset_id, 
            amount, 
            destination, 
            gas_price, 
            gas_limit, 
            chain_id,
            override_nonce
        } => {
            match PASS_WALLET_MANAGER.withdraw_to_external(
                &wallet_address, 
                &subaccount_id, 
                &asset_id, 
                amount, 
                &destination, 
                gas_price, 
                gas_limit, 
                chain_id,
                override_nonce
            ) {
                Ok((signed_transaction, nonce, actual_gas_price, actual_gas_limit)) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "signed_raw_transaction": signed_transaction,
                        "wallet_address": wallet_address,
                        "subaccount_id": subaccount_id,
                        "asset_id": asset_id,
                        "amount": amount,
                        "destination": destination,
                        "chain_id": chain_id,
                        "nonce": nonce,
                        "gas_price": actual_gas_price,
                        "gas_limit": actual_gas_limit
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Withdrawal failed: {}", e)),
                }),
            }
        }

        Command::GetOutboxQueue => {
            match PASS_WALLET_MANAGER.get_outbox_queue() {
                Ok(outbox_queue) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "outbox_queue": outbox_queue
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get outbox queue: {}", e)),
                }),
            }
        }

        Command::RemoveFromOutbox { nonce } => {
            match PASS_WALLET_MANAGER.remove_from_outbox(nonce) {
                Ok(()) => Ok(Response {
                    success: true,
                    data: Some(serde_json::json!({
                        "message": "Transaction removed from outbox",
                        "nonce": nonce
                    })),
                    error: None,
                }),
                Err(e) => Ok(Response {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to remove from outbox: {}", e)),
                }),
            }
        }
    }
}