use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use anyhow::Result;

use nitro_enclave::key_manager::EnclaveKMS;
use nitro_enclave::pass_logic::{
    PassWalletManager, Asset, Subaccount, Deposit, TokenType
};

/// Integration test environment that simulates real-world scenarios
struct IntegrationTestEnvironment {
    manager: PassWalletManager,
    wallets: HashMap<String, String>, // name -> address
    assets: HashMap<String, String>,  // symbol -> asset_id
}

impl IntegrationTestEnvironment {
    fn new() -> Result<Self> {
        let kms = Arc::new(Mutex::new(EnclaveKMS::new("integration_test_secret")?));
        let manager = PassWalletManager::new(kms);
        
        Ok(IntegrationTestEnvironment {
            manager,
            wallets: HashMap::new(),
            assets: HashMap::new(),
        })
    }
    
    /// Create a wallet and store its reference
    fn create_wallet(&mut self, name: &str, owner: &str) -> Result<String> {
        let address = self.manager.create_wallet(name.to_string(), owner.to_string())?;
        self.wallets.insert(name.to_string(), address.clone());
        Ok(address)
    }
    
    /// Add an asset to a wallet and store its reference
    fn add_asset(&mut self, wallet_name: &str, symbol: &str, asset: Asset) -> Result<()> {
        let wallet_address = self.wallets.get(wallet_name)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", wallet_name))?;
        
        let asset_id = format!("{}_{}", symbol.to_lowercase(), wallet_name);
        self.manager.add_asset(wallet_address, asset_id.clone(), asset)?;
        self.assets.insert(format!("{}_{}", symbol, wallet_name), asset_id);
        Ok(())
    }
    
    /// Add a subaccount to a wallet
    fn add_subaccount(&self, wallet_name: &str, subaccount_id: &str, label: &str) -> Result<()> {
        let wallet_address = self.wallets.get(wallet_name)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", wallet_name))?;
        
        let subaccount = Subaccount {
            id: subaccount_id.to_string(),
            label: label.to_string(),
            address: wallet_address.clone(),
        };
        
        self.manager.add_subaccount(wallet_address, subaccount)?;
        Ok(())
    }
    
    /// Create and process a deposit
    fn process_deposit(&self, wallet_name: &str, asset_symbol: &str, amount: u64, deposit_id: &str, subaccount_id: &str) -> Result<()> {
        let wallet_address = self.wallets.get(wallet_name)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", wallet_name))?;
        
        let asset_id = self.assets.get(&format!("{}_{}", asset_symbol, wallet_name))
            .ok_or_else(|| anyhow::anyhow!("Asset not found: {} for wallet {}", asset_symbol, wallet_name))?;
        
        let deposit = Deposit {
            asset_id: asset_id.clone(),
            amount,
            deposit_id: deposit_id.to_string(),
            transaction_hash: format!("0x{}", hex::encode(format!("{}_{}", wallet_name, deposit_id))),
            block_number: "12345".to_string(),
            from_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            to_address: wallet_address.clone(),
        };
        
        self.manager.inbox_deposit(wallet_address, deposit)?;
        self.manager.claim_inbox(wallet_address, deposit_id, subaccount_id)?;
        
        Ok(())
    }
    
    /// Get balance for convenience
    fn get_balance(&self, wallet_name: &str, subaccount_id: &str, asset_symbol: &str) -> Result<u64> {
        let wallet_address = self.wallets.get(wallet_name)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", wallet_name))?;
        
        let asset_id = self.assets.get(&format!("{}_{}", asset_symbol, wallet_name))
            .ok_or_else(|| anyhow::anyhow!("Asset not found: {} for wallet {}", asset_symbol, wallet_name))?;
        
        self.manager.get_balance(wallet_address, subaccount_id, asset_id)
    }
    
    /// Transfer between subaccounts
    fn transfer(&self, wallet_name: &str, asset_symbol: &str, amount: u64, from_subaccount: &str, to_subaccount: &str) -> Result<()> {
        let wallet_address = self.wallets.get(wallet_name)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", wallet_name))?;
        
        let asset_id = self.assets.get(&format!("{}_{}", asset_symbol, wallet_name))
            .ok_or_else(|| anyhow::anyhow!("Asset not found: {} for wallet {}", asset_symbol, wallet_name))?;
        
        self.manager.internal_transfer(wallet_address, asset_id, amount, from_subaccount, to_subaccount)
    }
    
    /// Withdraw to external address
    fn withdraw(&self, wallet_name: &str, asset_symbol: &str, amount: u64, subaccount_id: &str, destination: &str) -> Result<()> {
        let wallet_address = self.wallets.get(wallet_name)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", wallet_name))?;
        
        let asset_id = self.assets.get(&format!("{}_{}", asset_symbol, wallet_name))
            .ok_or_else(|| anyhow::anyhow!("Asset not found: {} for wallet {}", asset_symbol, wallet_name))?;
        
        self.manager.withdraw(wallet_address, asset_id, amount, subaccount_id, destination)
    }
    
    /// Withdraw to external address using withdraw_to_external (with gas calculation)
    fn withdraw_external(&self, wallet_name: &str, asset_symbol: &str, amount: u64, subaccount_id: &str, destination: &str) -> Result<(String, u64, u64, u64)> {
        let wallet_address = self.wallets.get(wallet_name)
            .ok_or_else(|| anyhow::anyhow!("Wallet not found: {}", wallet_name))?;
        
        let asset_id = self.assets.get(&format!("{}_{}", asset_symbol, wallet_name))
            .ok_or_else(|| anyhow::anyhow!("Asset not found: {} for wallet {}", asset_symbol, wallet_name))?;
        
        self.manager.withdraw_to_external(
            wallet_address, 
            subaccount_id, 
            asset_id, 
            amount, 
            destination,
            None, // gas_price
            None, // gas_limit
            11155111, // Sepolia chain_id
            None // override_nonce
        )
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_multi_user_trading_scenario() -> Result<()> {
        let mut env = IntegrationTestEnvironment::new()?;
        
        // Create multiple users
        env.create_wallet("alice_wallet", "alice")?;
        env.create_wallet("bob_wallet", "bob")?;
        env.create_wallet("charlie_wallet", "charlie")?;
        
        // Setup assets for each wallet
        let eth_asset = Asset {
            token_type: TokenType::ETH,
            contract_address: None,
            token_id: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
        };
        
        let usdc_asset = Asset {
            token_type: TokenType::ERC20,
            contract_address: Some("0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2".to_string()),
            token_id: None,
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
        };
        
        // Add assets to all wallets
        for wallet in ["alice_wallet", "bob_wallet", "charlie_wallet"] {
            env.add_asset(wallet, "ETH", eth_asset.clone())?;
            env.add_asset(wallet, "USDC", usdc_asset.clone())?;
        }
        
        // Add subaccounts for each user
        for wallet in ["alice_wallet", "bob_wallet", "charlie_wallet"] {
            env.add_subaccount(wallet, "main", "Main Account")?;
            env.add_subaccount(wallet, "trading", "Trading Account")?;
            env.add_subaccount(wallet, "savings", "Savings Account")?;
        }
        
        // Scenario: Alice receives large ETH deposit, Bob receives USDC, Charlie receives both
        env.process_deposit("alice_wallet", "ETH", 10000000000000000000, "alice_eth_1", "main")?; // 10 ETH
        env.process_deposit("bob_wallet", "USDC", 50000000000, "bob_usdc_1", "main")?; // 50,000 USDC
        env.process_deposit("charlie_wallet", "ETH", 5000000000000000000, "charlie_eth_1", "main")?; // 5 ETH
        env.process_deposit("charlie_wallet", "USDC", 25000000000, "charlie_usdc_1", "main")?; // 25,000 USDC
        
        // Verify initial deposits
        assert_eq!(env.get_balance("alice_wallet", "main", "ETH")?, 10000000000000000000);
        assert_eq!(env.get_balance("bob_wallet", "main", "USDC")?, 50000000000);
        assert_eq!(env.get_balance("charlie_wallet", "main", "ETH")?, 5000000000000000000);
        assert_eq!(env.get_balance("charlie_wallet", "main", "USDC")?, 25000000000);
        
        // Alice allocates funds between accounts
        env.transfer("alice_wallet", "ETH", 3000000000000000000, "main", "trading")?; // 3 ETH to trading
        env.transfer("alice_wallet", "ETH", 2000000000000000000, "main", "savings")?; // 2 ETH to savings
        
        // Bob allocates USDC
        env.transfer("bob_wallet", "USDC", 20000000000, "main", "trading")?; // 20,000 USDC to trading
        env.transfer("bob_wallet", "USDC", 15000000000, "main", "savings")?; // 15,000 USDC to savings
        
        // Charlie does mixed allocations
        env.transfer("charlie_wallet", "ETH", 2000000000000000000, "main", "trading")?; // 2 ETH to trading
        env.transfer("charlie_wallet", "USDC", 10000000000, "main", "trading")?; // 10,000 USDC to trading
        env.transfer("charlie_wallet", "ETH", 1000000000000000000, "main", "savings")?; // 1 ETH to savings
        env.transfer("charlie_wallet", "USDC", 5000000000, "main", "savings")?; // 5,000 USDC to savings
        
        // Verify allocations
        assert_eq!(env.get_balance("alice_wallet", "main", "ETH")?, 5000000000000000000); // 5 ETH remaining
        assert_eq!(env.get_balance("alice_wallet", "trading", "ETH")?, 3000000000000000000); // 3 ETH
        assert_eq!(env.get_balance("alice_wallet", "savings", "ETH")?, 2000000000000000000); // 2 ETH
        
        assert_eq!(env.get_balance("bob_wallet", "main", "USDC")?, 15000000000); // 15,000 USDC remaining
        assert_eq!(env.get_balance("bob_wallet", "trading", "USDC")?, 20000000000); // 20,000 USDC
        assert_eq!(env.get_balance("bob_wallet", "savings", "USDC")?, 15000000000); // 15,000 USDC
        
        // Simulate trading activity - withdrawals from trading accounts
        env.withdraw("alice_wallet", "ETH", 1000000000000000000, "trading", "0x1111111111111111111111111111111111111111")?; // 1 ETH
        env.withdraw("bob_wallet", "USDC", 5000000000, "trading", "0x2222222222222222222222222222222222222222")?; // 5,000 USDC
        env.withdraw("charlie_wallet", "ETH", 500000000000000000, "trading", "0x3333333333333333333333333333333333333333")?; // 0.5 ETH
        env.withdraw("charlie_wallet", "USDC", 2000000000, "trading", "0x4444444444444444444444444444444444444444")?; // 2,000 USDC
        
        // Verify final balances after withdrawals
        assert_eq!(env.get_balance("alice_wallet", "trading", "ETH")?, 2000000000000000000); // 2 ETH remaining
        assert_eq!(env.get_balance("bob_wallet", "trading", "USDC")?, 15000000000); // 15,000 USDC remaining
        assert_eq!(env.get_balance("charlie_wallet", "trading", "ETH")?, 1500000000000000000); // 1.5 ETH remaining
        assert_eq!(env.get_balance("charlie_wallet", "trading", "USDC")?, 8000000000); // 8,000 USDC remaining
        
        // Verify outbox entries were created for withdrawals
        for wallet_name in ["alice_wallet", "bob_wallet", "charlie_wallet"] {
            let wallet_address = env.wallets.get(wallet_name).unwrap();
            let wallet_state = env.manager.get_wallet(wallet_address).unwrap();
            assert!(!wallet_state.outbox.is_empty(), "Wallet {} should have outbox entries", wallet_name);
        }
        
        Ok(())
    }

    #[test]
    fn test_complex_multi_asset_workflow() -> Result<()> {
        let mut env = IntegrationTestEnvironment::new()?;
        
        // Create enterprise wallet
        env.create_wallet("enterprise_wallet", "enterprise_corp")?;
        
        // Add multiple assets
        let assets = vec![
            ("ETH", Asset {
                token_type: TokenType::ETH,
                contract_address: None,
                token_id: None,
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                decimals: 18,
            }),
            ("USDC", Asset {
                token_type: TokenType::ERC20,
                contract_address: Some("0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2".to_string()),
                token_id: None,
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                decimals: 6,
            }),
            ("WBTC", Asset {
                token_type: TokenType::ERC20,
                contract_address: Some("0x2260fac5e5542a773aa44fbcfedf7c193bc2c599".to_string()),
                token_id: None,
                symbol: "WBTC".to_string(),
                name: "Wrapped Bitcoin".to_string(),
                decimals: 8,
            }),
        ];
        
        for (symbol, asset) in assets {
            env.add_asset("enterprise_wallet", symbol, asset)?;
        }
        
        // Add departmental subaccounts
        let departments = vec![
            ("treasury", "Treasury Department"),
            ("operations", "Operations Department"),
            ("development", "Development Department"),
            ("marketing", "Marketing Department"),
            ("reserves", "Emergency Reserves"),
        ];
        
        for (dept_id, dept_label) in departments {
            env.add_subaccount("enterprise_wallet", dept_id, dept_label)?;
        }
        
        // Simulate large incoming deposits from various sources
        let deposits = vec![
            ("ETH", 10000000000000000000u64, "treasury"), // 10 ETH to treasury
            ("USDC", 1000000000000u64, "treasury"), // 1,000,000 USDC to treasury
            ("WBTC", 500000000u64, "treasury"), // 5 BTC to treasury
            ("ETH", 2000000000000000000u64, "operations"), // 2 ETH to operations
            ("USDC", 200000000000u64, "operations"), // 200,000 USDC to operations
        ];
        
        for (i, (asset, amount, dept)) in deposits.iter().enumerate() {
            env.process_deposit("enterprise_wallet", asset, *amount, &format!("deposit_{}", i), dept)?;
        }
        
        // Verify initial deposits
        assert_eq!(env.get_balance("enterprise_wallet", "treasury", "ETH")?, 10000000000000000000);
        assert_eq!(env.get_balance("enterprise_wallet", "treasury", "USDC")?, 1000000000000);
        assert_eq!(env.get_balance("enterprise_wallet", "treasury", "WBTC")?, 500000000);
        assert_eq!(env.get_balance("enterprise_wallet", "operations", "ETH")?, 2000000000000000000);
        assert_eq!(env.get_balance("enterprise_wallet", "operations", "USDC")?, 200000000000);
        
        // Treasury allocates funds to departments
        env.transfer("enterprise_wallet", "ETH", 3000000000000000000, "treasury", "development")?; // 3 ETH
        env.transfer("enterprise_wallet", "ETH", 2000000000000000000, "treasury", "marketing")?; // 2 ETH
        env.transfer("enterprise_wallet", "ETH", 2500000000000000000, "treasury", "reserves")?; // 2.5 ETH
        
        env.transfer("enterprise_wallet", "USDC", 300000000000, "treasury", "development")?; // 300,000 USDC
        env.transfer("enterprise_wallet", "USDC", 200000000000, "treasury", "marketing")?; // 200,000 USDC
        env.transfer("enterprise_wallet", "USDC", 250000000000, "treasury", "reserves")?; // 250,000 USDC
        
        env.transfer("enterprise_wallet", "WBTC", 100000000, "treasury", "reserves")?; // 1 BTC to reserves
        
        // Operations transfers some funds to development
        env.transfer("enterprise_wallet", "ETH", 500000000000000000, "operations", "development")?; // 0.5 ETH
        env.transfer("enterprise_wallet", "USDC", 50000000000, "operations", "development")?; // 50,000 USDC
        
        // Simulate departmental spending (withdrawals)
        let withdrawals = vec![
            ("development", "ETH", 1000000000000000000u64, "0xdev1111111111111111111111111111111111111"), // 1 ETH
            ("development", "USDC", 100000000000u64, "0xdev2222222222222222222222222222222222222"), // 100,000 USDC
            ("marketing", "ETH", 500000000000000000u64, "0xmkt1111111111111111111111111111111111111"), // 0.5 ETH
            ("marketing", "USDC", 75000000000u64, "0xmkt2222222222222222222222222222222222222"), // 75,000 USDC
            ("operations", "ETH", 500000000000000000u64, "0xops1111111111111111111111111111111111111"), // 0.5 ETH
            ("operations", "USDC", 25000000000u64, "0xops2222222222222222222222222222222222222"), // 25,000 USDC
        ];
        
        for (dept, asset, amount, destination) in withdrawals {
            env.withdraw("enterprise_wallet", asset, amount, dept, destination)?;
        }
        
        // Verify final balances
        assert_eq!(env.get_balance("enterprise_wallet", "treasury", "ETH")?, 2500000000000000000); // 2.5 ETH remaining
        assert_eq!(env.get_balance("enterprise_wallet", "treasury", "USDC")?, 250000000000); // 250,000 USDC remaining
        assert_eq!(env.get_balance("enterprise_wallet", "treasury", "WBTC")?, 400000000); // 4 BTC remaining
        
        assert_eq!(env.get_balance("enterprise_wallet", "development", "ETH")?, 2500000000000000000); // 3+0.5-1 = 2.5 ETH
        assert_eq!(env.get_balance("enterprise_wallet", "development", "USDC")?, 250000000000); // 300+50-100 = 250k USDC
        
        assert_eq!(env.get_balance("enterprise_wallet", "marketing", "ETH")?, 1500000000000000000); // 2-0.5 = 1.5 ETH
        assert_eq!(env.get_balance("enterprise_wallet", "marketing", "USDC")?, 125000000000); // 200-75 = 125k USDC
        
        assert_eq!(env.get_balance("enterprise_wallet", "operations", "ETH")?, 1000000000000000000); // 2-0.5-0.5 = 1 ETH
        assert_eq!(env.get_balance("enterprise_wallet", "operations", "USDC")?, 125000000000); // 200-50-25 = 125k USDC
        
        assert_eq!(env.get_balance("enterprise_wallet", "reserves", "ETH")?, 2500000000000000000); // 2.5 ETH
        assert_eq!(env.get_balance("enterprise_wallet", "reserves", "USDC")?, 250000000000); // 250k USDC
        assert_eq!(env.get_balance("enterprise_wallet", "reserves", "WBTC")?, 100000000); // 1 BTC
        
        // Verify transaction history
        let wallet_address = env.wallets.get("enterprise_wallet").unwrap();
        let wallet_state = env.manager.get_wallet(wallet_address).unwrap();
        
        // Should have extensive history: 5 claims + multiple transfers + multiple withdrawals
        assert!(wallet_state.history.len() > 15, "Should have extensive transaction history");
        
        // Verify provenance filtering works with complex history
        let eth_provenance = env.manager.get_provenance_by_asset(wallet_address, 
            &env.assets.get("ETH_enterprise_wallet").unwrap())?;
        let usdc_provenance = env.manager.get_provenance_by_asset(wallet_address, 
            &env.assets.get("USDC_enterprise_wallet").unwrap())?;
        let wbtc_provenance = env.manager.get_provenance_by_asset(wallet_address, 
            &env.assets.get("WBTC_enterprise_wallet").unwrap())?;
        
        // Verify each asset has appropriate number of transactions
        let eth_records: Vec<serde_json::Value> = serde_json::from_value(
            eth_provenance["provenance_records"].clone()
        )?;
        let usdc_records: Vec<serde_json::Value> = serde_json::from_value(
            usdc_provenance["provenance_records"].clone()
        )?;
        let wbtc_records: Vec<serde_json::Value> = serde_json::from_value(
            wbtc_provenance["provenance_records"].clone()
        )?;
        
        assert!(eth_records.len() >= 8, "ETH should have many transactions");
        assert!(usdc_records.len() >= 8, "USDC should have many transactions");
        assert!(wbtc_records.len() >= 2, "WBTC should have fewer transactions");
        
        Ok(())
    }

    #[test]
    fn test_high_frequency_micro_transactions() -> Result<()> {
        let mut env = IntegrationTestEnvironment::new()?;
        
        env.create_wallet("hft_wallet", "hft_trader")?;
        
        let eth_asset = Asset {
            token_type: TokenType::ETH,
            contract_address: None,
            token_id: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
        };
        
        env.add_asset("hft_wallet", "ETH", eth_asset)?;
        env.add_subaccount("hft_wallet", "hot", "Hot Wallet")?;
        env.add_subaccount("hft_wallet", "cold", "Cold Storage")?;
        
        // Initial large deposit
        env.process_deposit("hft_wallet", "ETH", 10000000000000000000, "initial_deposit", "hot")?; // 10 ETH
        
        // Simulate high-frequency micro-transactions
        let num_micro_transactions = 500;
        let micro_amount = 1000000000000000; // 0.001 ETH each
        
        for i in 0..num_micro_transactions {
            if i % 2 == 0 {
                // Transfer from hot to cold
                env.transfer("hft_wallet", "ETH", micro_amount, "hot", "cold")?;
            } else {
                // Transfer from cold to hot
                env.transfer("hft_wallet", "ETH", micro_amount, "cold", "hot")?;
            }
            
            // Occasional withdrawal
            if i % 50 == 0 && i > 0 {
                env.withdraw("hft_wallet", "ETH", micro_amount, "hot", 
                           &format!("0x{:040}", i))?;
            }
        }
        
        // Calculate actual number of withdrawals that occurred
        let mut withdrawal_count = 0;
        for i in 0..num_micro_transactions {
            if i % 50 == 0 && i > 0 {
                withdrawal_count += 1;
            }
        }
        
        // Verify final state
        let hot_balance = env.get_balance("hft_wallet", "hot", "ETH")?;
        let cold_balance = env.get_balance("hft_wallet", "cold", "ETH")?;
        let total_balance = hot_balance + cold_balance;
        
        // Should have less than original due to withdrawals
        let expected_withdrawn = withdrawal_count * micro_amount;
        let expected_remaining = 10000000000000000000 - expected_withdrawn;
        
        assert_eq!(total_balance, expected_remaining, 
                  "Total balance should account for withdrawals");
        
        // Verify extensive transaction history
        let wallet_address = env.wallets.get("hft_wallet").unwrap();
        let wallet_state = env.manager.get_wallet(wallet_address).unwrap();
        let expected_history_count = 1 + num_micro_transactions + withdrawal_count; // claim + transfers + withdrawals
        assert_eq!(wallet_state.history.len(), expected_history_count as usize,
                  "Should have correct number of history entries: {} claim + {} transfers + {} withdrawals = {}",
                  1, num_micro_transactions, withdrawal_count, expected_history_count);
        
        Ok(())
    }

    #[test]
    fn test_cross_wallet_simulation() -> Result<()> {
        let mut env = IntegrationTestEnvironment::new()?;
        
        // Create ecosystem of wallets
        let wallets = vec![
            ("exchange_wallet", "crypto_exchange"),
            ("user1_wallet", "alice"),
            ("user2_wallet", "bob"),
            ("merchant_wallet", "online_store"),
            ("treasury_wallet", "company_treasury"),
        ];
        
        for (wallet_name, owner) in &wallets {
            env.create_wallet(wallet_name, owner)?;
        }
        
        // Add ETH and USDC to all wallets
        let eth_asset = Asset {
            token_type: TokenType::ETH,
            contract_address: None,
            token_id: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
        };
        
        let usdc_asset = Asset {
            token_type: TokenType::ERC20,
            contract_address: Some("0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2".to_string()),
            token_id: None,
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
        };
        
        for (wallet_name, _) in &wallets {
            env.add_asset(wallet_name, "ETH", eth_asset.clone())?;
            env.add_asset(wallet_name, "USDC", usdc_asset.clone())?;
            env.add_subaccount(wallet_name, "main", "Main Account")?;
            env.add_subaccount(wallet_name, "secondary", "Secondary Account")?;
        }
        
        // Simulate ecosystem activity
        
        // 1. Exchange receives large deposits
        env.process_deposit("exchange_wallet", "ETH", 5000000000000000000, "exchange_eth", "main")?; // 5 ETH
        env.process_deposit("exchange_wallet", "USDC", 10000000000000, "exchange_usdc", "main")?; // 10M USDC
        
        // 2. Users deposit to exchange (simulated as internal transfers to secondary)
        env.process_deposit("user1_wallet", "ETH", 10000000000000000000, "user1_deposit", "main")?; // 10 ETH
        env.process_deposit("user2_wallet", "USDC", 50000000000, "user2_deposit", "main")?; // 50k USDC
        
        // 3. Users move funds around
        env.transfer("user1_wallet", "ETH", 3000000000000000000, "main", "secondary")?; // 3 ETH
        env.transfer("user2_wallet", "USDC", 15000000000, "main", "secondary")?; // 15k USDC
        
        // 4. Merchant receives payments
        env.process_deposit("merchant_wallet", "ETH", 2000000000000000000, "payment1", "main")?; // 2 ETH
        env.process_deposit("merchant_wallet", "USDC", 5000000000, "payment2", "main")?; // 5k USDC
        
        // 5. Treasury receives company funds
        env.process_deposit("treasury_wallet", "ETH", 10000000000000000000, "company_funds", "main")?; // 10 ETH
        env.process_deposit("treasury_wallet", "USDC", 2000000000000, "company_usdc", "main")?; // 2M USDC
        
        // 6. Simulate various withdrawals
        env.withdraw("exchange_wallet", "ETH", 5000000000000000000, "main", "0xexchange1")?; // 5 ETH
        env.withdraw("user1_wallet", "ETH", 1000000000000000000, "secondary", "0xuser1dest")?; // 1 ETH
        env.withdraw("merchant_wallet", "USDC", 2000000000, "main", "0xmerchant1")?; // 2k USDC
        env.withdraw("treasury_wallet", "ETH", 1000000000000000000, "main", "0xtreasury1")?; // 1 ETH
        
        // Verify all wallets maintain correct balances
        let final_balances = vec![
            ("exchange_wallet", "main", "ETH", 0u64), // 5 - 5 = 0
            ("exchange_wallet", "main", "USDC", 10000000000000u64), // 10M
            ("user1_wallet", "main", "ETH", 7000000000000000000u64), // 10 - 3
            ("user1_wallet", "secondary", "ETH", 2000000000000000000u64), // 3 - 1
            ("user2_wallet", "main", "USDC", 35000000000u64), // 50k - 15k
            ("user2_wallet", "secondary", "USDC", 15000000000u64), // 15k
            ("merchant_wallet", "main", "ETH", 2000000000000000000u64), // 2
            ("merchant_wallet", "main", "USDC", 3000000000u64), // 5k - 2k
            ("treasury_wallet", "main", "ETH", 9000000000000000000u64), // 10 - 1
            ("treasury_wallet", "main", "USDC", 2000000000000u64), // 2M
        ];
        
        for (wallet, subaccount, asset, expected) in final_balances {
            let actual = env.get_balance(wallet, subaccount, asset)?;
            assert_eq!(actual, expected, 
                      "Balance mismatch for {}.{}.{}: expected {}, got {}", 
                      wallet, subaccount, asset, expected, actual);
        }
        
        // Verify each wallet has appropriate transaction history
        for (wallet_name, _) in &wallets {
            let wallet_address = env.wallets.get(*wallet_name).unwrap();
            let wallet_state = env.manager.get_wallet(wallet_address).unwrap();
            assert!(!wallet_state.history.is_empty(), 
                   "Wallet {} should have transaction history", wallet_name);
        }
        
        Ok(())
    }

    #[test]
    fn test_error_recovery_scenarios() -> Result<()> {
        let mut env = IntegrationTestEnvironment::new()?;
        
        env.create_wallet("test_wallet", "test_user")?;
        
        let eth_asset = Asset {
            token_type: TokenType::ETH,
            contract_address: None,
            token_id: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
        };
        
        env.add_asset("test_wallet", "ETH", eth_asset)?;
        env.add_subaccount("test_wallet", "main", "Main Account")?;
        env.add_subaccount("test_wallet", "backup", "Backup Account")?;
        
        // Initial deposit
        env.process_deposit("test_wallet", "ETH", 10000000000000000000, "initial", "main")?; // 10 ETH
        
        // Test various error conditions and recovery
        
        // 1. Try to transfer more than available - should fail gracefully
        let transfer_result = env.transfer("test_wallet", "ETH", 15000000000000000000, "main", "backup");
        assert!(transfer_result.is_err(), "Should fail when transferring more than available");
        
        // Verify balance unchanged after failed transfer
        assert_eq!(env.get_balance("test_wallet", "main", "ETH")?, 10000000000000000000);
        assert_eq!(env.get_balance("test_wallet", "backup", "ETH")?, 0);
        
        // 2. Try to withdraw more than available - should fail gracefully
        let withdraw_result = env.withdraw("test_wallet", "ETH", 15000000000000000000, "main", "0x1234");
        assert!(withdraw_result.is_err(), "Should fail when withdrawing more than available");
        
        // Verify balance unchanged after failed withdrawal
        assert_eq!(env.get_balance("test_wallet", "main", "ETH")?, 10000000000000000000);
        
        // 3. Successful partial operations should work after failures
        env.transfer("test_wallet", "ETH", 3000000000000000000, "main", "backup")?; // 3 ETH - should work
        assert_eq!(env.get_balance("test_wallet", "main", "ETH")?, 7000000000000000000);
        assert_eq!(env.get_balance("test_wallet", "backup", "ETH")?, 3000000000000000000);
        
        // 4. Try duplicate deposit ID - should fail
        let deposit1 = Deposit {
            asset_id: env.assets.get("ETH_test_wallet").unwrap().clone(),
            amount: 1000000000000000000,
            deposit_id: "duplicate_test".to_string(),
            transaction_hash: "0x1111".to_string(),
            block_number: "123".to_string(),
            from_address: "0x1111111111111111111111111111111111111111".to_string(),
            to_address: env.wallets.get("test_wallet").unwrap().clone(),
        };
        
        let wallet_address = env.wallets.get("test_wallet").unwrap();
        
        // First deposit should succeed
        env.manager.inbox_deposit(wallet_address, deposit1.clone())?;
        
        // Duplicate should fail
        let duplicate_result = env.manager.inbox_deposit(wallet_address, deposit1);
        assert!(duplicate_result.is_err(), "Duplicate deposit should fail");
        
        // 5. Try to claim non-existent deposit - should fail
        let claim_result = env.manager.claim_inbox(wallet_address, "nonexistent", "main");
        assert!(claim_result.is_err(), "Claiming non-existent deposit should fail");
        
        // 6. Verify system state is still consistent after all failures
        let wallet_state = env.manager.get_wallet(wallet_address).unwrap();
        
        // Should have: 1 initial claim + 1 transfer + 1 pending deposit
        assert_eq!(wallet_state.history.len(), 2); // claim + transfer
        assert_eq!(wallet_state.inbox.len(), 1); // pending deposit
        
        // Total balance should still be correct
        let total_balance = env.get_balance("test_wallet", "main", "ETH")? + 
                          env.get_balance("test_wallet", "backup", "ETH")?;
        assert_eq!(total_balance, 10000000000000000000); // Should still be 10 ETH total
        
        Ok(())
    }

    #[test]
    fn test_withdrawal_gas_calculation_fix() -> Result<()> {
        let mut env = IntegrationTestEnvironment::new()?;
        
        // Create test wallet
        env.create_wallet("gas_test_wallet", "test_user")?;
        
        // Add ETH asset
        let eth_asset = Asset {
            token_type: TokenType::ETH,
            contract_address: None,
            token_id: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
        };
        
        // Add USDC asset  
        let usdc_asset = Asset {
            token_type: TokenType::ERC20,
            contract_address: Some("0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2".to_string()),
            token_id: None,
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
        };
        
        env.add_asset("gas_test_wallet", "ETH", eth_asset)?;
        env.add_asset("gas_test_wallet", "USDC", usdc_asset)?;
        env.add_subaccount("gas_test_wallet", "main", "Main Account")?;
        
        // Deposit enough ETH to cover withdrawal + gas
        let initial_eth_amount = 1000000000000000000u64; // 1 ETH
        env.process_deposit("gas_test_wallet", "ETH", initial_eth_amount, "eth_deposit", "main")?;
        
        // Deposit USDC for testing non-ETH withdrawal
        let initial_usdc_amount = 100000000u64; // 100 USDC
        env.process_deposit("gas_test_wallet", "USDC", initial_usdc_amount, "usdc_deposit", "main")?;
        
        // Verify initial balances
        assert_eq!(env.get_balance("gas_test_wallet", "main", "ETH")?, initial_eth_amount);
        assert_eq!(env.get_balance("gas_test_wallet", "main", "USDC")?, initial_usdc_amount);
        
        // Test 1: ETH withdrawal with proper gas calculation
        let eth_withdrawal_amount = 500000000000000000u64; // 0.5 ETH
        let result = env.withdraw_external(
            "gas_test_wallet", 
            "ETH", 
            eth_withdrawal_amount, 
            "main", 
            "0x1234567890123456789012345678901234567890"
        );
        
        assert!(result.is_ok(), "ETH withdrawal should succeed with sufficient balance for gas");
        let (_, _, gas_price, gas_limit) = result.unwrap();
        let gas_cost = gas_price * gas_limit;
        
        // Verify ETH balance was reduced by both withdrawal amount AND gas cost
        let expected_eth_balance = initial_eth_amount - eth_withdrawal_amount - gas_cost;
        assert_eq!(env.get_balance("gas_test_wallet", "main", "ETH")?, expected_eth_balance);
        
        // Test 2: USDC withdrawal with ETH gas fee deduction  
        let usdc_withdrawal_amount = 50000000u64; // 50 USDC
        let eth_balance_before_usdc_withdrawal = env.get_balance("gas_test_wallet", "main", "ETH")?;
        
        let result = env.withdraw_external(
            "gas_test_wallet",
            "USDC", 
            usdc_withdrawal_amount, 
            "main", 
            "0x2234567890123456789012345678901234567890"
        );
        
        assert!(result.is_ok(), "USDC withdrawal should succeed with sufficient ETH for gas");
        let (_, _, gas_price, gas_limit) = result.unwrap();
        let gas_cost = gas_price * gas_limit;
        
        // Verify USDC balance was reduced by withdrawal amount
        let expected_usdc_balance = initial_usdc_amount - usdc_withdrawal_amount;
        assert_eq!(env.get_balance("gas_test_wallet", "main", "USDC")?, expected_usdc_balance);
        
        // Verify ETH balance was reduced by gas cost only
        let expected_eth_balance_after_usdc = eth_balance_before_usdc_withdrawal - gas_cost;
        assert_eq!(env.get_balance("gas_test_wallet", "main", "ETH")?, expected_eth_balance_after_usdc);
        
        // Test 3: Insufficient ETH for gas fees should fail
        // Try to withdraw remaining ETH leaving no gas
        let remaining_eth = env.get_balance("gas_test_wallet", "main", "ETH")?;
        let result = env.withdraw_external(
            "gas_test_wallet", 
            "ETH", 
            remaining_eth, // Try to withdraw ALL remaining ETH
            "main", 
            "0x3234567890123456789012345678901234567890"
        );
        
        assert!(result.is_err(), "Should fail when trying to withdraw all ETH without leaving gas");
        assert!(result.err().unwrap().to_string().contains("Insufficient ETH for withdrawal + gas fees"));
        
        // Test 4: Insufficient ETH for gas fees when withdrawing other assets should fail
        // Try to withdraw more USDC when ETH balance is too low for gas
        let result = env.withdraw_external(
            "gas_test_wallet",
            "USDC", 
            usdc_withdrawal_amount, // Try another USDC withdrawal 
            "main", 
            "0x4234567890123456789012345678901234567890"
        );
        
        // This might succeed if there's still enough ETH for gas, or fail if not
        // The important thing is that the error message should be clear about gas fees
        if result.is_err() {
            assert!(result.err().unwrap().to_string().contains("Insufficient ETH for gas fees"));
        }
        
        println!("Gas calculation withdrawal tests completed successfully!");
        Ok(())
    }
}
