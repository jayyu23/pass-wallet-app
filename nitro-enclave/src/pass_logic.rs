use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use crate::key_manager::EnclaveKMS;
// Helper function to convert string address to bytes
fn parse_address(addr_str: &str) -> Result<Vec<u8>> {
    let clean_addr = addr_str.strip_prefix("0x").unwrap_or(addr_str);
    if clean_addr.len() != 40 {
        return Err(anyhow!("Invalid address length"));
    }
    hex::decode(clean_addr).map_err(|e| anyhow!("Invalid address hex: {}", e))
}

// Helper function to convert u64 to big-endian bytes (removing leading zeros)
fn u64_to_be_bytes_minimal(value: u64) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }
    let bytes = value.to_be_bytes();
    let mut start = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        if byte != 0 {
            start = i;
            break;
        }
    }
    bytes[start..].to_vec()
}

/// Asset type identifier (e.g., "ETH", "USDC", etc.)
pub type AssetType = String;

/// User ID in the system
pub type UserId = String;

/// Deposit ID for tracking external deposits
pub type DepositId = String;

/// External destination address for withdrawals
pub type ExternalDestination = String;

/// Wallet address (unique identifier for each PASS wallet)
pub type WalletAddress = String;

/// Token type matching the database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    ETH,
    ERC20,
    ERC721,
    ERC1155,
}

/// Asset information matching the database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub token_type: TokenType,
    pub contract_address: Option<String>,
    pub token_id: Option<String>,
    pub symbol: String,
    pub name: String,
    pub decimals: u32,
}

/// Subaccount within a wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subaccount {
    pub id: String,
    pub label: String,
    pub address: String,
}

/// Balance for a subaccount-asset pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaccountBalance {
    pub subaccount_id: String,
    pub asset_id: String,
    pub amount: u64,
}

/// Deposit entry in the inbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub asset_id: String,
    pub amount: u64,
    pub deposit_id: DepositId,
    pub transaction_hash: String,
    pub block_number: String,
    pub from_address: String,
    pub to_address: String,
}

/// Outbox entry for withdrawals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEntry {
    pub asset_id: String,
    pub amount: u64,
    pub external_destination: ExternalDestination,
    pub nonce: u64,
}

/// Transaction operation types for provenance history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionOperation {
    Claim { asset_id: String, amount: u64, deposit_id: DepositId, subaccount_id: String },
    Transfer { asset_id: String, amount: u64, from_subaccount: String, to_subaccount: String },
    Withdraw { asset_id: String, amount: u64, subaccount_id: String, destination: ExternalDestination, signed_raw_transaction: String, nonce: u64, gas_price: u64, gas_limit: u64 },
}

/// Provenance history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub operation: TransactionOperation,
    pub timestamp: u64,
    pub block_number: Option<u64>,
}

/// Individual PASS wallet state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassWalletState {
    pub address: WalletAddress,
    pub name: String,
    pub owner: String,
    pub nonce: u64,
    pub inbox: Vec<Deposit>,
    pub outbox: VecDeque<OutboxEntry>,
    pub assets: HashMap<String, Asset>,
    pub subaccounts: HashMap<String, Subaccount>,
    pub balances: HashMap<String, u64>, // subaccount_id:asset_id -> amount
    pub history: Vec<ProvenanceRecord>,
    pub created_at: u64,
}

impl PassWalletState {
    /// Create a new PASS wallet state
    pub fn new(address: WalletAddress, name: String, owner: String) -> Self {
        PassWalletState {
            address,
            name,
            owner,
            nonce: 0,
            inbox: Vec::new(),
            outbox: VecDeque::new(),
            assets: HashMap::new(),
            subaccounts: HashMap::new(),
            balances: HashMap::new(),
            history: Vec::new(),
            created_at: Self::get_timestamp(),
        }
    }

    /// Add an asset to the wallet
    pub fn add_asset(&mut self, asset_id: String, asset: Asset) {
        self.assets.insert(asset_id, asset);
    }

    /// Add a subaccount to the wallet
    pub fn add_subaccount(&mut self, subaccount: Subaccount) {
        self.subaccounts.insert(subaccount.id.clone(), subaccount);
    }

    /// Get balance for a subaccount-asset pair
    pub fn get_balance(&self, subaccount_id: &str, asset_id: &str) -> u64 {
        let balance_key = format!("{}:{}", subaccount_id, asset_id);
        self.balances.get(&balance_key).copied().unwrap_or(0)
    }

    /// Set balance for a subaccount-asset pair
    fn set_balance(&mut self, subaccount_id: &str, asset_id: &str, amount: u64) {
        let balance_key = format!("{}:{}", subaccount_id, asset_id);
        self.balances.insert(balance_key, amount);
    }

    /// Add external deposit to inbox
    pub fn inbox_deposit(&mut self, deposit: Deposit) -> Result<()> {
        // Check if deposit ID already exists
        if self.inbox.iter().any(|d| d.deposit_id == deposit.deposit_id) {
            return Err(anyhow!("Deposit ID already exists"));
        }
        
        self.inbox.push(deposit);
        Ok(())
    }

    /// Claim deposit from inbox
    pub fn claim_inbox(&mut self, deposit_id: &str, subaccount_id: &str) -> Result<()> {
        // Find and remove the deposit from inbox
        let deposit_index = self.inbox.iter().position(|d| d.deposit_id == deposit_id)
            .ok_or_else(|| anyhow!("Deposit not found in inbox"))?;
        
        let deposit = self.inbox.remove(deposit_index);
        
        // Update balance
        let current_balance = self.get_balance(subaccount_id, &deposit.asset_id);
        self.set_balance(subaccount_id, &deposit.asset_id, current_balance + deposit.amount);
        
        // Add to provenance history
        self.history.push(ProvenanceRecord {
            operation: TransactionOperation::Claim {
                asset_id: deposit.asset_id,
                amount: deposit.amount,
                deposit_id: deposit.deposit_id,
                subaccount_id: subaccount_id.to_string(),
            },
            timestamp: Self::get_timestamp(),
            block_number: None,
        });
        
        Ok(())
    }

    /// Check if a subaccount is allowed to perform a transaction
    pub fn check_allow(&self, subaccount_id: &str, asset_id: &str, amount: u64) -> bool {
        self.get_balance(subaccount_id, asset_id) >= amount
    }

    /// Internal transfer between subaccounts
    pub fn internal_transfer(&mut self, asset_id: &str, amount: u64, from_subaccount: &str, to_subaccount: &str) -> Result<()> {
        // Check if sender has sufficient balance
        if !self.check_allow(from_subaccount, asset_id, amount) {
            return Err(anyhow!("Insufficient balance"));
        }
        
        // Update balances
        let from_balance = self.get_balance(from_subaccount, asset_id);
        let to_balance = self.get_balance(to_subaccount, asset_id);
        
        self.set_balance(from_subaccount, asset_id, from_balance - amount);
        self.set_balance(to_subaccount, asset_id, to_balance + amount);
        
        // Add to provenance history
        self.history.push(ProvenanceRecord {
            operation: TransactionOperation::Transfer {
                asset_id: asset_id.to_string(),
                amount,
                from_subaccount: from_subaccount.to_string(),
                to_subaccount: to_subaccount.to_string(),
            },
            timestamp: Self::get_timestamp(),
            block_number: None,
        });
        
        Ok(())
    }

    /// Withdraw to external destination
    pub fn withdraw(&mut self, asset_id: &str, amount: u64, subaccount_id: &str, external_destination: &str) -> Result<()> {
        // Check if subaccount has sufficient balance
        if !self.check_allow(subaccount_id, asset_id, amount) {
            return Err(anyhow!("Insufficient balance"));
        }
        
        // Update balance
        let current_balance = self.get_balance(subaccount_id, asset_id);
        self.set_balance(subaccount_id, asset_id, current_balance - amount);
        
        // Add to outbox
        self.outbox.push_back(OutboxEntry {
            asset_id: asset_id.to_string(),
            amount,
            external_destination: external_destination.to_string(),
            nonce: self.nonce,
        });
        
        // Add to provenance history
        self.history.push(ProvenanceRecord {
            operation: TransactionOperation::Withdraw {
                asset_id: asset_id.to_string(),
                amount,
                subaccount_id: subaccount_id.to_string(),
                destination: external_destination.to_string(),
                signed_raw_transaction: "pending".to_string(), // Will be updated when transaction is signed
                nonce: 0, // Will be updated when transaction is signed
                gas_price: 0, // Will be updated when transaction is signed
                gas_limit: 0, // Will be updated when transaction is signed
            },
            timestamp: Self::get_timestamp(),
            block_number: None,
        });
        
        Ok(())
    }

    /// Process outbox (periodic or on-demand)
    pub fn process_outbox(&mut self) -> Result<Vec<OutboxEntry>> {
        let mut processed = Vec::new();
        
        while let Some(entry) = self.outbox.pop_front() {
            processed.push(entry);
            self.nonce += 1;
        }
        
        Ok(processed)
    }

    /// Get all balances for a subaccount
    pub fn get_subaccount_balances(&self, subaccount_id: &str) -> HashMap<String, u64> {
        self.balances.iter()
            .filter_map(|(key, amount)| {
                if let Some((sub_id, asset_id)) = key.split_once(':') {
                    if sub_id == subaccount_id {
                        Some((asset_id.to_string(), *amount))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get wallet state summary
    pub fn get_state_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "address": self.address,
            "name": self.name,
            "owner": self.owner,
            "nonce": self.nonce,
            "inbox_count": self.inbox.len(),
            "outbox_count": self.outbox.len(),
            "assets_count": self.assets.len(),
            "subaccounts_count": self.subaccounts.len(),
            "history_count": self.history.len(),
            "created_at": self.created_at
        })
    }

    /// Helper function to get current timestamp
    fn get_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Pending withdrawal transaction with signed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingWithdrawal {
    pub wallet_address: WalletAddress,
    pub subaccount_id: String,
    pub asset_id: String,
    pub amount: u64,
    pub destination: String,
    pub nonce: u64,
    pub signed_raw_transaction: String,
    pub created_at: u64,
}

/// PASS Wallet Manager - manages multiple PASS wallets
pub struct PassWalletManager {
    kms: Arc<Mutex<EnclaveKMS>>,
    wallets: Arc<Mutex<HashMap<WalletAddress, PassWalletState>>>,
    /// Global nonce counter for transaction sequencing
    global_nonce: Arc<Mutex<u64>>,
    /// Outbox queue for pending withdrawal transactions
    outbox_queue: Arc<Mutex<VecDeque<PendingWithdrawal>>>,
}

impl PassWalletManager {
    /// Create a new PASS wallet manager
    pub fn new(kms: Arc<Mutex<EnclaveKMS>>) -> Self {
        PassWalletManager {
            kms,
            wallets: Arc::new(Mutex::new(HashMap::new())),
            global_nonce: Arc::new(Mutex::new(0)),
            outbox_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Create a new PASS wallet
    pub fn create_wallet(&self, name: String, owner: String) -> Result<WalletAddress> {
        // Generate a new Ethereum account using the existing KMS
        let account = {
            let mut kms = self.kms.lock().unwrap();
            kms.handle_keygen()?
        };

        let address = account.address.clone();
        let wallet_state = PassWalletState::new(address.clone(), name, owner);

        // Store the wallet
        {
            let mut wallets = self.wallets.lock().unwrap();
            wallets.insert(address.clone(), wallet_state);
        }

        Ok(address)
    }

    /// Get a wallet by address
    pub fn get_wallet(&self, address: &str) -> Option<PassWalletState> {
        let wallets = self.wallets.lock().unwrap();
        wallets.get(address).cloned()
    }

    /// Update a wallet
    pub fn update_wallet(&self, address: &str, wallet_state: PassWalletState) -> Result<()> {
        let mut wallets = self.wallets.lock().unwrap();
        if wallets.contains_key(address) {
            wallets.insert(address.to_string(), wallet_state);
            Ok(())
        } else {
            Err(anyhow!("Wallet not found"))
        }
    }

    /// List all wallet addresses
    pub fn list_wallets(&self) -> Vec<WalletAddress> {
        let wallets = self.wallets.lock().unwrap();
        wallets.keys().cloned().collect()
    }

    /// Sign a message using a wallet's private key
    pub fn sign_message(&self, wallet_address: &str, domain: &str, message: &str) -> Result<String> {
        // Use the existing KMS to sign the message
        let kms = self.kms.lock().unwrap();
        let full_message = format!("{}:{}", domain, message);
        
        match kms.sign_message(&full_message, wallet_address)? {
            Some(signature) => Ok(signature),
            None => Err(anyhow!("Failed to sign message - wallet not found")),
        }
    }

    /// Execute inbox deposit
    pub fn inbox_deposit(&self, wallet_address: &str, deposit: Deposit) -> Result<()> {
        let mut wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        wallet_state.inbox_deposit(deposit)?;
        self.update_wallet(wallet_address, wallet_state)?;
        Ok(())
    }

    /// Execute claim inbox
    pub fn claim_inbox(&self, wallet_address: &str, deposit_id: &str, subaccount_id: &str) -> Result<()> {
        let mut wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        wallet_state.claim_inbox(deposit_id, subaccount_id)?;
        self.update_wallet(wallet_address, wallet_state)?;
        Ok(())
    }

    /// Execute internal transfer
    pub fn internal_transfer(&self, wallet_address: &str, asset_id: &str, amount: u64, from_subaccount: &str, to_subaccount: &str) -> Result<()> {
        let mut wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        wallet_state.internal_transfer(asset_id, amount, from_subaccount, to_subaccount)?;
        self.update_wallet(wallet_address, wallet_state)?;
        Ok(())
    }

    /// Execute withdrawal
    pub fn withdraw(&self, wallet_address: &str, asset_id: &str, amount: u64, subaccount_id: &str, destination: &str) -> Result<()> {
        let mut wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        wallet_state.withdraw(asset_id, amount, subaccount_id, destination)?;
        self.update_wallet(wallet_address, wallet_state)?;
        Ok(())
    }

    /// Process outbox
    pub fn process_outbox(&self, wallet_address: &str) -> Result<Vec<OutboxEntry>> {
        let mut wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        let processed = wallet_state.process_outbox()?;
        self.update_wallet(wallet_address, wallet_state)?;
        Ok(processed)
    }

    /// Add asset to wallet
    pub fn add_asset(&self, wallet_address: &str, asset_id: String, asset: Asset) -> Result<()> {
        let mut wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        wallet_state.add_asset(asset_id, asset);
        self.update_wallet(wallet_address, wallet_state)?;
        Ok(())
    }

    /// Add subaccount to wallet
    pub fn add_subaccount(&self, wallet_address: &str, subaccount: Subaccount) -> Result<()> {
        let mut wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        wallet_state.add_subaccount(subaccount);
        self.update_wallet(wallet_address, wallet_state)?;
        Ok(())
    }

    /// Get balance for a subaccount
    pub fn get_balance(&self, wallet_address: &str, subaccount_id: &str, asset_id: &str) -> Result<u64> {
        let wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        Ok(wallet_state.get_balance(subaccount_id, asset_id))
    }

    /// Get all balances for a subaccount
    pub fn get_subaccount_balances(&self, wallet_address: &str, subaccount_id: &str) -> Result<HashMap<String, u64>> {
        let wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        Ok(wallet_state.get_subaccount_balances(subaccount_id))
    }

    /// Get wallet state summary
    pub fn get_wallet_state(&self, wallet_address: &str) -> Result<serde_json::Value> {
        let wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        Ok(wallet_state.get_state_summary())
    }

    /// Get all assets from a wallet's asset ledger with total balances across all subaccounts
    pub fn get_wallet_assets(&self, wallet_address: &str) -> Result<serde_json::Value> {
        let wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        let mut assets_with_balances = serde_json::Map::new();
        
        for (asset_id, asset) in &wallet_state.assets {
            // Calculate total balance for this asset across all subaccounts
            let total_balance: u64 = wallet_state.balances
                .iter()
                .filter_map(|(balance_key, amount)| {
                    if let Some((_subaccount_id, balance_asset_id)) = balance_key.split_once(':') {
                        if balance_asset_id == asset_id {
                            Some(*amount)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .sum();

            // Get per-subaccount balances for this asset
            let mut subaccount_balances = serde_json::Map::new();
            for (balance_key, amount) in &wallet_state.balances {
                if let Some((subaccount_id, balance_asset_id)) = balance_key.split_once(':') {
                    if balance_asset_id == asset_id && *amount > 0 {
                        subaccount_balances.insert(subaccount_id.to_string(), serde_json::Value::Number(serde_json::Number::from(*amount)));
                    }
                }
            }

            assets_with_balances.insert(asset_id.clone(), serde_json::json!({
                "token_type": asset.token_type,
                "contract_address": asset.contract_address,
                "token_id": asset.token_id,
                "symbol": asset.symbol,
                "name": asset.name,
                "decimals": asset.decimals,
                "total_balance": total_balance,
                "subaccount_balances": subaccount_balances
            }));
        }
        
        Ok(serde_json::json!({
            "wallet_address": wallet_address,
            "assets": assets_with_balances
        }))
    }

    /// Get full provenance log for a wallet
    pub fn get_provenance_log(&self, wallet_address: &str) -> Result<serde_json::Value> {
        let wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        Ok(serde_json::json!({
            "wallet_address": wallet_address,
            "provenance_records": wallet_state.history
        }))
    }

    /// Get provenance log filtered by asset
    pub fn get_provenance_by_asset(&self, wallet_address: &str, asset_id: &str) -> Result<serde_json::Value> {
        let wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        let filtered_records: Vec<&ProvenanceRecord> = wallet_state.history.iter()
            .filter(|record| {
                match &record.operation {
                    TransactionOperation::Claim { asset_id: a, .. } => a == asset_id,
                    TransactionOperation::Transfer { asset_id: a, .. } => a == asset_id,
                    TransactionOperation::Withdraw { asset_id: a, .. } => a == asset_id,
                }
            })
            .collect();
        
        Ok(serde_json::json!({
            "wallet_address": wallet_address,
            "asset_id": asset_id,
            "provenance_records": filtered_records
        }))
    }

    /// Get provenance log filtered by subaccount
    pub fn get_provenance_by_subaccount(&self, wallet_address: &str, subaccount_id: &str) -> Result<serde_json::Value> {
        let wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        let filtered_records: Vec<&ProvenanceRecord> = wallet_state.history.iter()
            .filter(|record| {
                match &record.operation {
                    TransactionOperation::Claim { subaccount_id: s, .. } => s == subaccount_id,
                    TransactionOperation::Transfer { from_subaccount, to_subaccount, .. } => {
                        from_subaccount == subaccount_id || to_subaccount == subaccount_id
                    },
                    TransactionOperation::Withdraw { subaccount_id: s, .. } => s == subaccount_id,
                }
            })
            .collect();
        
        Ok(serde_json::json!({
            "wallet_address": wallet_address,
            "subaccount_id": subaccount_id,
            "provenance_records": filtered_records
        }))
    }

    /// Withdraw assets to external address - builds and signs transaction
    pub fn withdraw_to_external(&self, 
        wallet_address: &str, 
        subaccount_id: &str, 
        asset_id: &str, 
        amount: u64, 
        destination: &str,
        gas_price: Option<u64>,
        gas_limit: Option<u64>,
        chain_id: u64,
        override_nonce: Option<u64>
    ) -> Result<(String, u64, u64, u64)> {
        // Parse destination address first (no locks needed)
        let to_address = parse_address(destination)?;
        
        // CRITICAL: Lock the entire withdrawal process to ensure atomicity and sequencing
        let mut global_nonce = self.global_nonce.lock().unwrap();
        
        // Get and validate wallet state
        let mut wallet_state = self.get_wallet(wallet_address)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        // Get asset info
        let asset = wallet_state.assets.get(asset_id)
            .ok_or_else(|| anyhow!("Asset not found"))?;

        // Check sufficient balance for the asset being withdrawn
        let current_balance = wallet_state.get_balance(subaccount_id, asset_id);
        if current_balance < amount {
            return Err(anyhow!("Insufficient balance: {} available, {} requested", current_balance, amount));
        }

        // Calculate estimated gas cost first
        let estimated_gas_price = gas_price.unwrap_or(5_000_000_000); // 5 gwei default for Sepolia
        let estimated_gas_limit = match asset.token_type {
            TokenType::ETH => gas_limit.unwrap_or(21_000),
            TokenType::ERC20 => gas_limit.unwrap_or(60_000),
            _ => {
                return Err(anyhow!("Withdrawal not supported for asset type: {:?}", asset.token_type));
            }
        };
        
        let total_gas_cost = estimated_gas_price * estimated_gas_limit;
        
        // Find ETH asset for gas fee calculation
        let eth_asset_id = wallet_state.assets.iter()
            .find(|(_, asset)| matches!(asset.token_type, TokenType::ETH))
            .map(|(id, _)| id.clone())
            .ok_or_else(|| anyhow!("No ETH asset found in wallet for gas fee calculation"))?;
        
        let current_eth_balance = wallet_state.get_balance(subaccount_id, &eth_asset_id);
        
        // Check gas fee requirements
        if asset_id == &eth_asset_id {
            // Withdrawing ETH: need amount + gas fees
            let total_eth_needed = amount + total_gas_cost;
            if current_eth_balance < total_eth_needed {
                return Err(anyhow!(
                    "Insufficient ETH for withdrawal + gas fees: {} wei available, {} wei needed ({} withdrawal + {} gas)", 
                    current_eth_balance, total_eth_needed, amount, total_gas_cost
                ));
            }
        } else {
            // Withdrawing other asset: need ETH for gas fees only
            if current_eth_balance < total_gas_cost {
                return Err(anyhow!(
                    "Insufficient ETH for gas fees: {} wei available, {} wei needed for gas", 
                    current_eth_balance, total_gas_cost
                ));
            }
        }
        
        // Use provided nonce or fallback to internal counter
        let wallet_nonce = match override_nonce {
            Some(nonce) => nonce,
            None => {
                // Fallback to internal nonce management (for backward compatibility)
                wallet_state.nonce += 1;
                wallet_state.nonce
            }
        };
        
        // Get global nonce for transaction ordering
        *global_nonce += 1;
        let tx_nonce = *global_nonce;
        drop(global_nonce);
        
        // Build transaction based on asset type
        let (raw_transaction, actual_gas_price, actual_gas_limit) = match asset.token_type {
            TokenType::ETH => {
                // Use provided gas price or reasonable default
                let gas_price_final = gas_price.unwrap_or(5_000_000_000); // 5 gwei default for Sepolia
                let gas_limit_final = gas_limit.unwrap_or(21_000); // standard ETH transfer
                let tx = self.build_eth_transaction(
                    to_address,
                    amount,
                    asset.decimals,
                    wallet_nonce,
                    gas_price_final,
                    gas_limit_final,
                    chain_id,
                    wallet_address,
                )?;
                (tx, gas_price_final, gas_limit_final)
            },
            TokenType::ERC20 => {
                let contract_address = parse_address(
                    asset.contract_address
                        .as_ref()
                        .ok_or_else(|| anyhow!("ERC20 contract address not found"))?
                )?;
                
                // Use provided gas price or reasonable default
                let gas_price_final = gas_price.unwrap_or(5_000_000_000); // 5 gwei default for Sepolia
                let gas_limit_final = gas_limit.unwrap_or(60_000); // standard ERC20 transfer
                let tx = self.build_erc20_transaction(
                    contract_address,
                    to_address.clone(),
                    amount,
                    wallet_nonce,
                    gas_price_final,
                    gas_limit_final,
                    chain_id,
                    wallet_address,
                )?;
                (tx, gas_price_final, gas_limit_final)
            },
            _ => {
                return Err(anyhow!("Withdrawal not supported for asset type: {:?}", asset.token_type));
            }
        };
        
        // Update wallet balances
        wallet_state.set_balance(subaccount_id, asset_id, current_balance - amount);
        
        // For ETH withdrawals, also deduct gas fees from ETH balance
        // For other assets, deduct gas fees from ETH balance
        if asset_id == &eth_asset_id {
            // ETH withdrawal: balance already reduced by withdrawal amount above, 
            // now also deduct gas fees from the same balance
            let updated_eth_balance = wallet_state.get_balance(subaccount_id, &eth_asset_id);
            wallet_state.set_balance(subaccount_id, &eth_asset_id, updated_eth_balance - total_gas_cost);
        } else {
            // Other asset withdrawal: deduct gas fees from ETH balance
            let current_eth_balance = wallet_state.get_balance(subaccount_id, &eth_asset_id);
            wallet_state.set_balance(subaccount_id, &eth_asset_id, current_eth_balance - total_gas_cost);
        }
        
        // Add to provenance history
        wallet_state.history.push(ProvenanceRecord {
            operation: TransactionOperation::Withdraw {
                asset_id: asset_id.to_string(),
                amount,
                subaccount_id: subaccount_id.to_string(),
                destination: destination.to_string(),
                signed_raw_transaction: raw_transaction.clone(),
                nonce: tx_nonce,
                gas_price: actual_gas_price,
                gas_limit: actual_gas_limit,
            },
            timestamp: PassWalletState::get_timestamp(),
            block_number: None, // Will be filled when transaction is mined
        });
        
        // Save updated wallet state
        self.update_wallet(wallet_address, wallet_state)?;
        
        // Create pending withdrawal record
        let pending_withdrawal = PendingWithdrawal {
            wallet_address: wallet_address.to_string(),
            subaccount_id: subaccount_id.to_string(),
            asset_id: asset_id.to_string(),
            amount,
            destination: destination.to_string(),
            nonce: tx_nonce,
            signed_raw_transaction: raw_transaction.clone(),
            created_at: PassWalletState::get_timestamp(),
        };
        
        // Add to outbox queue (FIFO)
        {
            let mut outbox = self.outbox_queue.lock().unwrap();
            outbox.push_back(pending_withdrawal);
        }
        
        Ok((raw_transaction, tx_nonce, actual_gas_price, actual_gas_limit))
    }
    
    /// Build and sign ETH transaction
    fn build_eth_transaction(
        &self,
        to: Vec<u8>,
        amount: u64,
        _decimals: u32,
        nonce: u64,
        gas_price: u64,
        gas_limit: u64,
        chain_id: u64,
        wallet_address: &str,
    ) -> Result<String> {
        // Build transaction struct
        let tx = crate::key_manager::LegacyTransaction {
            nonce,
            gas_price: u64_to_be_bytes_minimal(gas_price),
            gas_limit: u64_to_be_bytes_minimal(gas_limit),
            to: Some(to),
            value: u64_to_be_bytes_minimal(amount),
            data: Vec::new(),
        };
        
        // Sign transaction using KMS
        let signed_tx = {
            let mut kms = self.kms.lock().unwrap();
            kms.sign_transaction(wallet_address, &tx, chain_id)?
        };
        
        Ok(signed_tx)
    }
    
    /// Build and sign ERC20 transaction
    fn build_erc20_transaction(
        &self,
        token_contract: Vec<u8>,
        to: Vec<u8>,
        amount: u64,
        nonce: u64,
        gas_price: u64,
        gas_limit: u64,
        chain_id: u64,
        wallet_address: &str,
    ) -> Result<String> {
        // ERC20 transfer function signature: transfer(address,uint256)
        let transfer_selector = [0xa9, 0x05, 0x9c, 0xbb]; // keccak256("transfer(address,uint256)")[0:4]
        
        // Encode function call data
        let mut call_data = Vec::new();
        call_data.extend_from_slice(&transfer_selector);
        
        // Encode address (32 bytes, left-padded)
        let mut addr_bytes = [0u8; 32];
        addr_bytes[12..32].copy_from_slice(&to);
        call_data.extend_from_slice(&addr_bytes);
        
        // Encode amount (32 bytes, big-endian)
        let mut amount_bytes = [0u8; 32];
        let amount_be = amount.to_be_bytes();
        amount_bytes[24..].copy_from_slice(&amount_be);
        call_data.extend_from_slice(&amount_bytes);
        
        // Build transaction struct
        let tx = crate::key_manager::LegacyTransaction {
            nonce,
            gas_price: u64_to_be_bytes_minimal(gas_price),
            gas_limit: u64_to_be_bytes_minimal(gas_limit),
            to: Some(token_contract),
            value: vec![0], // Zero value for ERC20 transfers
            data: call_data,
        };
        
        // Sign transaction using KMS
        let signed_tx = {
            let mut kms = self.kms.lock().unwrap();
            kms.sign_transaction(wallet_address, &tx, chain_id)?
        };
        
        Ok(signed_tx)
    }
    
    /// Get pending withdrawals from outbox queue
    pub fn get_outbox_queue(&self) -> Result<Vec<PendingWithdrawal>> {
        let outbox = self.outbox_queue.lock().unwrap();
        Ok(outbox.iter().cloned().collect())
    }
    
    /// Remove processed withdrawal from outbox queue
    pub fn remove_from_outbox(&self, nonce: u64) -> Result<()> {
        let mut outbox = self.outbox_queue.lock().unwrap();
        outbox.retain(|w| w.nonce != nonce);
        Ok(())
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_manager::EnclaveKMS;

    #[test]
    fn test_create_wallet_manager() {
        let kms = Arc::new(Mutex::new(EnclaveKMS::new("test_secret").unwrap()));
        let manager = PassWalletManager::new(kms);
        
        let wallet_address = manager.create_wallet("Test Wallet".to_string(), "alice".to_string()).unwrap();
        assert!(!wallet_address.is_empty());
        
        let wallet_state = manager.get_wallet(&wallet_address).unwrap();
        assert_eq!(wallet_state.name, "Test Wallet");
        assert_eq!(wallet_state.owner, "alice");
    }

    #[test]
    fn test_multiple_wallets() {
        let kms = Arc::new(Mutex::new(EnclaveKMS::new("test_secret").unwrap()));
        let manager = PassWalletManager::new(kms);
        
        let wallet1 = manager.create_wallet("Wallet 1".to_string(), "alice".to_string()).unwrap();
        let wallet2 = manager.create_wallet("Wallet 2".to_string(), "bob".to_string()).unwrap();
        
        assert_ne!(wallet1, wallet2);
        
        let wallets = manager.list_wallets();
        assert_eq!(wallets.len(), 2);
        assert!(wallets.contains(&wallet1));
        assert!(wallets.contains(&wallet2));
    }

    #[test]
    fn test_wallet_operations() {
        let kms = Arc::new(Mutex::new(EnclaveKMS::new("test_secret").unwrap()));
        let manager = PassWalletManager::new(kms);
        
        let wallet_address = manager.create_wallet("Test Wallet".to_string(), "alice".to_string()).unwrap();
        
        // Add asset
        let asset = Asset {
            token_type: TokenType::ETH,
            contract_address: None,
            token_id: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
        };
        manager.add_asset(&wallet_address, "eth".to_string(), asset).unwrap();
        
        // Add subaccount
        let subaccount = Subaccount {
            id: "sub1".to_string(),
            label: "Main Account".to_string(),
            address: "0x123...".to_string(),
        };
        manager.add_subaccount(&wallet_address, subaccount).unwrap();
        
        // Test deposit
        let deposit = Deposit {
            asset_id: "eth".to_string(),
            amount: 1000,
            deposit_id: "deposit1".to_string(),
            transaction_hash: "0xabc...".to_string(),
            block_number: "12345".to_string(),
            from_address: "0x456...".to_string(),
            to_address: wallet_address.clone(),
        };
        manager.inbox_deposit(&wallet_address, deposit).unwrap();
        
        // Test claim
        manager.claim_inbox(&wallet_address, "deposit1", "sub1").unwrap();
        
        // Check balance
        let balance = manager.get_balance(&wallet_address, "sub1", "eth").unwrap();
        assert_eq!(balance, 1000);
    }
}
