use std::sync::{Arc, Mutex};
use anyhow::Result;

use nitro_enclave::key_manager::EnclaveKMS;
use nitro_enclave::pass_logic::{
    PassWalletManager, Asset, Subaccount, Deposit, TokenType, TransactionOperation
};

/// Test helper struct to manage test environment
struct TestEnvironment {
    manager: PassWalletManager,
    wallet_address: String,
    main_subaccount: String,
    trading_subaccount: String,
    eth_asset_id: String,
    usdc_asset_id: String,
}

impl TestEnvironment {
    /// Create a new test environment with sample wallet and assets
    fn new() -> Result<Self> {
        let kms = Arc::new(Mutex::new(EnclaveKMS::new("test_secret_123")?));
        let manager = PassWalletManager::new(kms);
        
        // Create wallet
        let wallet_address = manager.create_wallet(
            "Test Wallet".to_string(),
            "test_user".to_string()
        )?;
        
        // Add ETH asset
        let eth_asset = Asset {
            token_type: TokenType::ETH,
            contract_address: None,
            token_id: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
        };
        let eth_asset_id = "eth".to_string();
        manager.add_asset(&wallet_address, eth_asset_id.clone(), eth_asset)?;
        
        // Add USDC asset
        let usdc_asset = Asset {
            token_type: TokenType::ERC20,
            contract_address: Some("0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2".to_string()),
            token_id: None,
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
        };
        let usdc_asset_id = "usdc".to_string();
        manager.add_asset(&wallet_address, usdc_asset_id.clone(), usdc_asset)?;
        
        // Add subaccounts
        let main_subaccount = "main".to_string();
        let main_subaccount_obj = Subaccount {
            id: main_subaccount.clone(),
            label: "Main Account".to_string(),
            address: wallet_address.clone(),
        };
        manager.add_subaccount(&wallet_address, main_subaccount_obj)?;
        
        let trading_subaccount = "trading".to_string();
        let trading_subaccount_obj = Subaccount {
            id: trading_subaccount.clone(),
            label: "Trading Account".to_string(),
            address: wallet_address.clone(),
        };
        manager.add_subaccount(&wallet_address, trading_subaccount_obj)?;
        
        Ok(TestEnvironment {
            manager,
            wallet_address,
            main_subaccount,
            trading_subaccount,
            eth_asset_id,
            usdc_asset_id,
        })
    }
    
    /// Helper to create a deposit
    fn create_deposit(&self, asset_id: &str, amount: u64, deposit_id: &str) -> Deposit {
        Deposit {
            asset_id: asset_id.to_string(),
            amount,
            deposit_id: deposit_id.to_string(),
            transaction_hash: format!("0x{}", hex::encode(format!("txhash_{}", deposit_id))),
            block_number: "12345".to_string(),
            from_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            to_address: self.wallet_address.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        
        // Verify wallet exists
        let wallet_state = env.manager.get_wallet(&env.wallet_address);
        assert!(wallet_state.is_some());
        
        let state = wallet_state.unwrap();
        assert_eq!(state.name, "Test Wallet");
        assert_eq!(state.owner, "test_user");
        assert_eq!(state.assets.len(), 2); // ETH and USDC
        assert_eq!(state.subaccounts.len(), 2); // main and trading
    }

    #[test]
    fn test_claim_deposit_flow() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Create and add deposit to inbox
        let deposit = env.create_deposit(&env.eth_asset_id, 1000000000000000000, "deposit_1"); // 1 ETH
        env.manager.inbox_deposit(&env.wallet_address, deposit)?;
        
        // Verify deposit in inbox
        let wallet_state = env.manager.get_wallet(&env.wallet_address).unwrap();
        assert_eq!(wallet_state.inbox.len(), 1);
        assert_eq!(wallet_state.inbox[0].amount, 1000000000000000000);
        
        // Initial balance should be 0
        let initial_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.main_subaccount,
            &env.eth_asset_id
        )?;
        assert_eq!(initial_balance, 0);
        
        // Claim the deposit
        env.manager.claim_inbox(&env.wallet_address, "deposit_1", &env.main_subaccount)?;
        
        // Verify balance updated
        let final_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.main_subaccount,
            &env.eth_asset_id
        )?;
        assert_eq!(final_balance, 1000000000000000000);
        
        // Verify deposit removed from inbox
        let wallet_state = env.manager.get_wallet(&env.wallet_address).unwrap();
        assert_eq!(wallet_state.inbox.len(), 0);
        
        // Verify provenance record
        assert_eq!(wallet_state.history.len(), 1);
        match &wallet_state.history[0].operation {
            TransactionOperation::Claim { asset_id, amount, deposit_id, subaccount_id } => {
                assert_eq!(asset_id, &env.eth_asset_id);
                assert_eq!(*amount, 1000000000000000000);
                assert_eq!(deposit_id, "deposit_1");
                assert_eq!(subaccount_id, &env.main_subaccount);
            }
            _ => panic!("Expected Claim operation"),
        }
        
        Ok(())
    }

    #[test]
    fn test_internal_transfer_flow() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Setup: Add deposit and claim it to main account
        let deposit = env.create_deposit(&env.eth_asset_id, 2000000000000000000, "deposit_2"); // 2 ETH
        env.manager.inbox_deposit(&env.wallet_address, deposit)?;
        env.manager.claim_inbox(&env.wallet_address, "deposit_2", &env.main_subaccount)?;
        
        // Verify initial balances
        let main_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.main_subaccount,
            &env.eth_asset_id
        )?;
        let trading_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.trading_subaccount,
            &env.eth_asset_id
        )?;
        assert_eq!(main_balance, 2000000000000000000);
        assert_eq!(trading_balance, 0);
        
        // Transfer 1 ETH from main to trading
        let transfer_amount = 1000000000000000000;
        env.manager.internal_transfer(
            &env.wallet_address,
            &env.eth_asset_id,
            transfer_amount,
            &env.main_subaccount,
            &env.trading_subaccount
        )?;
        
        // Verify final balances
        let main_balance_after = env.manager.get_balance(
            &env.wallet_address,
            &env.main_subaccount,
            &env.eth_asset_id
        )?;
        let trading_balance_after = env.manager.get_balance(
            &env.wallet_address,
            &env.trading_subaccount,
            &env.eth_asset_id
        )?;
        assert_eq!(main_balance_after, 1000000000000000000);
        assert_eq!(trading_balance_after, 1000000000000000000);
        
        // Verify provenance record
        let wallet_state = env.manager.get_wallet(&env.wallet_address).unwrap();
        assert_eq!(wallet_state.history.len(), 2); // Claim + Transfer
        
        match &wallet_state.history[1].operation {
            TransactionOperation::Transfer { asset_id, amount, from_subaccount, to_subaccount } => {
                assert_eq!(asset_id, &env.eth_asset_id);
                assert_eq!(*amount, transfer_amount);
                assert_eq!(from_subaccount, &env.main_subaccount);
                assert_eq!(to_subaccount, &env.trading_subaccount);
            }
            _ => panic!("Expected Transfer operation"),
        }
        
        Ok(())
    }

    #[test]
    fn test_withdraw_flow() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Setup: Add deposit and claim it
        let deposit = env.create_deposit(&env.eth_asset_id, 3000000000000000000, "deposit_3"); // 3 ETH
        env.manager.inbox_deposit(&env.wallet_address, deposit)?;
        env.manager.claim_inbox(&env.wallet_address, "deposit_3", &env.main_subaccount)?;
        
        // Verify initial balance
        let initial_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.main_subaccount,
            &env.eth_asset_id
        )?;
        assert_eq!(initial_balance, 3000000000000000000);
        
        // Perform withdrawal
        let withdraw_amount = 1000000000000000000; // 1 ETH
        let destination = "0x9876543210fedcba9876543210fedcba98765432";
        env.manager.withdraw(
            &env.wallet_address,
            &env.eth_asset_id,
            withdraw_amount,
            &env.main_subaccount,
            destination
        )?;
        
        // Verify balance reduced
        let final_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.main_subaccount,
            &env.eth_asset_id
        )?;
        assert_eq!(final_balance, 2000000000000000000);
        
        // Verify outbox entry created
        let wallet_state = env.manager.get_wallet(&env.wallet_address).unwrap();
        assert_eq!(wallet_state.outbox.len(), 1);
        assert_eq!(wallet_state.outbox[0].amount, withdraw_amount);
        assert_eq!(wallet_state.outbox[0].external_destination, destination);
        
        // Verify provenance record
        assert_eq!(wallet_state.history.len(), 2); // Claim + Withdraw
        match &wallet_state.history[1].operation {
            TransactionOperation::Withdraw { asset_id, amount, subaccount_id, destination: dest, .. } => {
                assert_eq!(asset_id, &env.eth_asset_id);
                assert_eq!(*amount, withdraw_amount);
                assert_eq!(subaccount_id, &env.main_subaccount);
                assert_eq!(dest, destination);
            }
            _ => panic!("Expected Withdraw operation"),
        }
        
        Ok(())
    }

    #[test]
    fn test_end_to_end_workflow() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Step 1: Deposit ETH and USDC
        let eth_deposit = env.create_deposit(&env.eth_asset_id, 5000000000000000000, "eth_deposit"); // 5 ETH
        let usdc_deposit = env.create_deposit(&env.usdc_asset_id, 10000000000, "usdc_deposit"); // 10,000 USDC (6 decimals)
        
        env.manager.inbox_deposit(&env.wallet_address, eth_deposit)?;
        env.manager.inbox_deposit(&env.wallet_address, usdc_deposit)?;
        
        // Step 2: Claim deposits to main account
        env.manager.claim_inbox(&env.wallet_address, "eth_deposit", &env.main_subaccount)?;
        env.manager.claim_inbox(&env.wallet_address, "usdc_deposit", &env.main_subaccount)?;
        
        // Step 3: Transfer some assets to trading account
        env.manager.internal_transfer(
            &env.wallet_address,
            &env.eth_asset_id,
            2000000000000000000, // 2 ETH
            &env.main_subaccount,
            &env.trading_subaccount
        )?;
        
        env.manager.internal_transfer(
            &env.wallet_address,
            &env.usdc_asset_id,
            5000000000, // 5,000 USDC
            &env.main_subaccount,
            &env.trading_subaccount
        )?;
        
        // Step 4: Withdraw from both accounts
        env.manager.withdraw(
            &env.wallet_address,
            &env.eth_asset_id,
            1000000000000000000, // 1 ETH from main
            &env.main_subaccount,
            "0x1111111111111111111111111111111111111111"
        )?;
        
        env.manager.withdraw(
            &env.wallet_address,
            &env.usdc_asset_id,
            2000000000, // 2,000 USDC from trading
            &env.trading_subaccount,
            "0x2222222222222222222222222222222222222222"
        )?;
        
        // Step 5: Verify final balances
        let main_eth_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.main_subaccount,
            &env.eth_asset_id
        )?;
        let main_usdc_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.main_subaccount,
            &env.usdc_asset_id
        )?;
        let trading_eth_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.trading_subaccount,
            &env.eth_asset_id
        )?;
        let trading_usdc_balance = env.manager.get_balance(
            &env.wallet_address,
            &env.trading_subaccount,
            &env.usdc_asset_id
        )?;
        
        // Expected balances:
        // Main ETH: 5 - 2 (transferred) - 1 (withdrawn) = 2 ETH
        // Main USDC: 10,000 - 5,000 (transferred) = 5,000 USDC
        // Trading ETH: 2 ETH (transferred)
        // Trading USDC: 5,000 - 2,000 (withdrawn) = 3,000 USDC
        
        assert_eq!(main_eth_balance, 2000000000000000000);
        assert_eq!(main_usdc_balance, 5000000000);
        assert_eq!(trading_eth_balance, 2000000000000000000);
        assert_eq!(trading_usdc_balance, 3000000000);
        
        // Step 6: Verify provenance history
        let wallet_state = env.manager.get_wallet(&env.wallet_address).unwrap();
        assert_eq!(wallet_state.history.len(), 6); // 2 claims + 2 transfers + 2 withdrawals
        assert_eq!(wallet_state.outbox.len(), 2); // 2 withdrawals
        
        // Step 7: Process outbox
        let processed = env.manager.process_outbox(&env.wallet_address)?;
        assert_eq!(processed.len(), 2);
        
        let wallet_state_after = env.manager.get_wallet(&env.wallet_address).unwrap();
        assert_eq!(wallet_state_after.outbox.len(), 0); // Outbox should be empty
        assert!(wallet_state_after.nonce > 0); // Nonce should be incremented
        
        Ok(())
    }

    #[test]
    fn test_insufficient_balance_scenarios() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Try to transfer without any balance
        let transfer_result = env.manager.internal_transfer(
            &env.wallet_address,
            &env.eth_asset_id,
            1000000000000000000,
            &env.main_subaccount,
            &env.trading_subaccount
        );
        assert!(transfer_result.is_err());
        
        // Add some balance
        let deposit = env.create_deposit(&env.eth_asset_id, 1000000000000000000, "deposit_4"); // 1 ETH
        env.manager.inbox_deposit(&env.wallet_address, deposit)?;
        env.manager.claim_inbox(&env.wallet_address, "deposit_4", &env.main_subaccount)?;
        
        // Try to transfer more than available
        let transfer_result = env.manager.internal_transfer(
            &env.wallet_address,
            &env.eth_asset_id,
            2000000000000000000, // 2 ETH, but only have 1
            &env.main_subaccount,
            &env.trading_subaccount
        );
        assert!(transfer_result.is_err());
        
        // Try to withdraw more than available
        let withdraw_result = env.manager.withdraw(
            &env.wallet_address,
            &env.eth_asset_id,
            2000000000000000000, // 2 ETH, but only have 1
            &env.main_subaccount,
            "0x1234567890abcdef1234567890abcdef12345678"
        );
        assert!(withdraw_result.is_err());
        
        Ok(())
    }

    #[test]
    fn test_duplicate_deposit_rejection() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        let deposit = env.create_deposit(&env.eth_asset_id, 1000000000000000000, "duplicate_deposit");
        
        // First deposit should succeed
        env.manager.inbox_deposit(&env.wallet_address, deposit.clone())?;
        
        // Second deposit with same ID should fail
        let duplicate_result = env.manager.inbox_deposit(&env.wallet_address, deposit);
        assert!(duplicate_result.is_err());
        
        Ok(())
    }

    #[test]
    fn test_claim_nonexistent_deposit() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Try to claim a deposit that doesn't exist
        let claim_result = env.manager.claim_inbox(
            &env.wallet_address,
            "nonexistent_deposit",
            &env.main_subaccount
        );
        assert!(claim_result.is_err());
        
        Ok(())
    }

    #[test]
    fn test_multi_asset_balances() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Add deposits for multiple assets
        let eth_deposit = env.create_deposit(&env.eth_asset_id, 2000000000000000000, "multi_eth");
        let usdc_deposit = env.create_deposit(&env.usdc_asset_id, 5000000000, "multi_usdc");
        
        env.manager.inbox_deposit(&env.wallet_address, eth_deposit)?;
        env.manager.inbox_deposit(&env.wallet_address, usdc_deposit)?;
        
        env.manager.claim_inbox(&env.wallet_address, "multi_eth", &env.main_subaccount)?;
        env.manager.claim_inbox(&env.wallet_address, "multi_usdc", &env.main_subaccount)?;
        
        // Get all balances for main subaccount
        let balances = env.manager.get_subaccount_balances(
            &env.wallet_address,
            &env.main_subaccount
        )?;
        
        assert_eq!(balances.len(), 2);
        assert_eq!(balances.get(&env.eth_asset_id), Some(&2000000000000000000));
        assert_eq!(balances.get(&env.usdc_asset_id), Some(&5000000000));
        
        Ok(())
    }

    #[test]
    fn test_provenance_filtering() -> Result<()> {
        let env = TestEnvironment::new()?;
        
        // Create complex transaction history
        let eth_deposit = env.create_deposit(&env.eth_asset_id, 3000000000000000000, "prov_eth");
        let usdc_deposit = env.create_deposit(&env.usdc_asset_id, 6000000000, "prov_usdc");
        
        env.manager.inbox_deposit(&env.wallet_address, eth_deposit)?;
        env.manager.inbox_deposit(&env.wallet_address, usdc_deposit)?;
        
        env.manager.claim_inbox(&env.wallet_address, "prov_eth", &env.main_subaccount)?;
        env.manager.claim_inbox(&env.wallet_address, "prov_usdc", &env.trading_subaccount)?;
        
        env.manager.internal_transfer(
            &env.wallet_address,
            &env.eth_asset_id,
            1000000000000000000,
            &env.main_subaccount,
            &env.trading_subaccount
        )?;
        
        env.manager.withdraw(
            &env.wallet_address,
            &env.usdc_asset_id,
            2000000000,
            &env.trading_subaccount,
            "0x3333333333333333333333333333333333333333"
        )?;
        
        // Test provenance by asset
        let eth_provenance = env.manager.get_provenance_by_asset(&env.wallet_address, &env.eth_asset_id)?;
        let eth_records: Vec<serde_json::Value> = serde_json::from_value(
            eth_provenance["provenance_records"].clone()
        )?;
        assert_eq!(eth_records.len(), 2); // ETH claim + ETH transfer
        
        let usdc_provenance = env.manager.get_provenance_by_asset(&env.wallet_address, &env.usdc_asset_id)?;
        let usdc_records: Vec<serde_json::Value> = serde_json::from_value(
            usdc_provenance["provenance_records"].clone()
        )?;
        assert_eq!(usdc_records.len(), 2); // USDC claim + USDC withdraw
        
        // Test provenance by subaccount
        let main_provenance = env.manager.get_provenance_by_subaccount(&env.wallet_address, &env.main_subaccount)?;
        let main_records: Vec<serde_json::Value> = serde_json::from_value(
            main_provenance["provenance_records"].clone()
        )?;
        assert_eq!(main_records.len(), 2); // ETH claim + ETH transfer (from main)
        
        let trading_provenance = env.manager.get_provenance_by_subaccount(&env.wallet_address, &env.trading_subaccount)?;
        let trading_records: Vec<serde_json::Value> = serde_json::from_value(
            trading_provenance["provenance_records"].clone()
        )?;
        assert_eq!(trading_records.len(), 3); // USDC claim + ETH transfer (to trading) + USDC withdraw
        
        Ok(())
    }
}
