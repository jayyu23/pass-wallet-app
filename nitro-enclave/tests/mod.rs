// Test module declarations for the PASS Wallet test suite

pub mod functionality_tests;
pub mod benchmark_tests;
pub mod integration_tests;

// Common test utilities and helpers
pub mod test_utils {
    use std::sync::{Arc, Mutex};
    use anyhow::Result;
    
    use nitro_enclave::key_manager::EnclaveKMS;
    use nitro_enclave::pass_logic::{PassWalletManager, Asset, Subaccount, Deposit, TokenType};
    
    /// Create a test KMS instance with a unique secret
    pub fn create_test_kms(secret_suffix: &str) -> Result<Arc<Mutex<EnclaveKMS>>> {
        let secret = format!("test_secret_{}", secret_suffix);
        let kms = EnclaveKMS::new(&secret)?;
        Ok(Arc::new(Mutex::new(kms)))
    }
    
    /// Create a test wallet manager with unique KMS
    pub fn create_test_manager(secret_suffix: &str) -> Result<PassWalletManager> {
        let kms = create_test_kms(secret_suffix)?;
        Ok(PassWalletManager::new(kms))
    }
    
    /// Create a standard ETH asset for testing
    pub fn create_eth_asset() -> Asset {
        Asset {
            token_type: TokenType::ETH,
            contract_address: None,
            token_id: None,
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
        }
    }
    
    /// Create a standard USDC asset for testing
    pub fn create_usdc_asset() -> Asset {
        Asset {
            token_type: TokenType::ERC20,
            contract_address: Some("0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2".to_string()),
            token_id: None,
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
        }
    }
    
    /// Create a standard WBTC asset for testing
    pub fn create_wbtc_asset() -> Asset {
        Asset {
            token_type: TokenType::ERC20,
            contract_address: Some("0x2260fac5e5542a773aa44fbcfedf7c193bc2c599".to_string()),
            token_id: None,
            symbol: "WBTC".to_string(),
            name: "Wrapped Bitcoin".to_string(),
            decimals: 8,
        }
    }
    
    /// Create a standard subaccount for testing
    pub fn create_test_subaccount(id: &str, label: &str, address: &str) -> Subaccount {
        Subaccount {
            id: id.to_string(),
            label: label.to_string(),
            address: address.to_string(),
        }
    }
    
    /// Create a test deposit
    pub fn create_test_deposit(
        asset_id: &str, 
        amount: u64, 
        deposit_id: &str, 
        to_address: &str
    ) -> Deposit {
        Deposit {
            asset_id: asset_id.to_string(),
            amount,
            deposit_id: deposit_id.to_string(),
            transaction_hash: format!("0x{}", hex::encode(format!("txhash_{}", deposit_id))),
            block_number: "12345".to_string(),
            from_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            to_address: to_address.to_string(),
        }
    }
    
    /// Setup a basic test environment with wallet, assets, and subaccounts
    pub fn setup_basic_test_env(test_name: &str) -> Result<(PassWalletManager, String, Vec<String>)> {
        let manager = create_test_manager(test_name)?;
        
        // Create wallet
        let wallet_address = manager.create_wallet(
            format!("{}_wallet", test_name),
            format!("{}_user", test_name)
        )?;
        
        // Add ETH and USDC assets
        let eth_asset_id = "eth".to_string();
        let usdc_asset_id = "usdc".to_string();
        
        manager.add_asset(&wallet_address, eth_asset_id.clone(), create_eth_asset())?;
        manager.add_asset(&wallet_address, usdc_asset_id.clone(), create_usdc_asset())?;
        
        // Add standard subaccounts
        let main_subaccount = create_test_subaccount("main", "Main Account", &wallet_address);
        let trading_subaccount = create_test_subaccount("trading", "Trading Account", &wallet_address);
        
        manager.add_subaccount(&wallet_address, main_subaccount)?;
        manager.add_subaccount(&wallet_address, trading_subaccount)?;
        
        Ok((manager, wallet_address, vec![eth_asset_id, usdc_asset_id]))
    }
    
    /// Common amounts for testing (in wei/base units)
    pub mod amounts {
        pub const ONE_ETH: u64 = 1_000_000_000_000_000_000;
        pub const HALF_ETH: u64 = 500_000_000_000_000_000;
        pub const TENTH_ETH: u64 = 100_000_000_000_000_000;
        pub const THOUSANDTH_ETH: u64 = 1_000_000_000_000_000;
        
        pub const ONE_THOUSAND_USDC: u64 = 1_000_000_000; // 6 decimals
        pub const ONE_HUNDRED_USDC: u64 = 100_000_000;
        pub const TEN_USDC: u64 = 10_000_000;
        pub const ONE_USDC: u64 = 1_000_000;
        
        pub const ONE_BTC_IN_WBTC: u64 = 100_000_000; // 8 decimals
        pub const HALF_BTC_IN_WBTC: u64 = 50_000_000;
    }
    
    /// Common test addresses
    pub mod addresses {
        pub const TEST_EXTERNAL_1: &str = "0x1111111111111111111111111111111111111111";
        pub const TEST_EXTERNAL_2: &str = "0x2222222222222222222222222222222222222222";
        pub const TEST_EXTERNAL_3: &str = "0x3333333333333333333333333333333333333333";
        pub const TEST_CONTRACT_1: &str = "0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2";
        pub const TEST_CONTRACT_2: &str = "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599";
    }
    
    /// Performance testing helpers
    pub mod perf {
        use std::time::{Duration, Instant};
        
        /// Measure execution time of a function
        pub fn measure_execution_time<F, R>(f: F) -> (R, Duration) 
        where
            F: FnOnce() -> R,
        {
            let start = Instant::now();
            let result = f();
            let duration = start.elapsed();
            (result, duration)
        }
        
        /// Calculate operations per second
        pub fn calculate_ops_per_second(operations: usize, duration: Duration) -> f64 {
            if duration.as_secs_f64() > 0.0 {
                operations as f64 / duration.as_secs_f64()
            } else {
                0.0
            }
        }
        
        /// Assert minimum performance threshold
        pub fn assert_performance_threshold(
            ops_per_second: f64, 
            min_threshold: f64, 
            operation_name: &str
        ) {
            assert!(
                ops_per_second >= min_threshold,
                "{} performance below threshold: {:.2} ops/sec (min: {:.2})",
                operation_name, ops_per_second, min_threshold
            );
        }
    }
}

// Test configuration constants
pub mod config {
    /// Default number of operations for benchmarks
    pub const DEFAULT_BENCHMARK_OPS: usize = 1000;
    
    /// Default number of concurrent operations
    pub const DEFAULT_CONCURRENT_OPS: usize = 10;
    
    /// Default warmup operations
    pub const DEFAULT_WARMUP_OPS: usize = 100;
    
    /// Performance thresholds (operations per second)
    pub mod thresholds {
        pub const WALLET_CREATION: f64 = 10.0;
        pub const CLAIM_OPERATIONS: f64 = 100.0;
        pub const TRANSFER_OPERATIONS: f64 = 500.0;
        pub const WITHDRAW_OPERATIONS: f64 = 500.0;
        pub const BALANCE_QUERIES: f64 = 10_000.0;
        pub const PROVENANCE_QUERIES: f64 = 1_000.0;
        pub const CONCURRENT_OPERATIONS: f64 = 50.0;
        pub const END_TO_END_WORKFLOW: f64 = 10.0;
    }
}
