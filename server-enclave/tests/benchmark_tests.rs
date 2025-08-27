use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use server_enclave::key_manager::EnclaveKMS;
use server_enclave::pass_logic::{Asset, Deposit, PassWalletManager, Subaccount, TokenType};

/// Benchmark configuration
#[derive(Clone)]
struct BenchmarkConfig {
    pub num_operations: usize,
    pub num_concurrent_operations: usize,
    pub warmup_operations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            num_operations: 100,
            num_concurrent_operations: 5,
            warmup_operations: 10,
        }
    }
}

/// Benchmark results structure
#[derive(Debug)]
struct BenchmarkResult {
    pub operation_name: String,
    pub total_operations: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub operations_per_second: f64,
}

impl BenchmarkResult {
    fn new(operation_name: &str, durations: &[Duration]) -> Self {
        let total_operations = durations.len();
        let total_duration: Duration = durations.iter().sum();
        let average_duration = total_duration / total_operations as u32;
        let min_duration = *durations.iter().min().unwrap_or(&Duration::ZERO);
        let max_duration = *durations.iter().max().unwrap_or(&Duration::ZERO);
        let operations_per_second = if total_duration.as_secs_f64() > 0.0 {
            total_operations as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        BenchmarkResult {
            operation_name: operation_name.to_string(),
            total_operations,
            total_duration,
            average_duration,
            min_duration,
            max_duration,
            operations_per_second,
        }
    }

    fn print_summary(&self) {
        println!("\n=== {} Benchmark Results ===", self.operation_name);
        println!("Total Operations: {}", self.total_operations);
        println!("Total Duration: {:?}", self.total_duration);
        println!("Average Duration: {:?}", self.average_duration);
        println!("Min Duration: {:?}", self.min_duration);
        println!("Max Duration: {:?}", self.max_duration);
        println!("Operations/Second: {:.2}", self.operations_per_second);
        println!("=======================================\n");
    }
}

/// Test environment for benchmarks
struct BenchmarkEnvironment {
    manager: PassWalletManager,
    wallet_address: String,
    subaccount_id: String,
    eth_asset_id: String,
    usdc_asset_id: String,
}

impl BenchmarkEnvironment {
    fn new() -> Result<Self> {
        let kms = Arc::new(Mutex::new(EnclaveKMS::new("benchmark_secret")?));
        let manager = PassWalletManager::new(kms);

        let wallet_address =
            manager.create_wallet("Benchmark Wallet".to_string(), "benchmark_user".to_string())?;

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

        // Add subaccount
        let subaccount_id = "bench_main".to_string();
        let subaccount = Subaccount {
            id: subaccount_id.clone(),
            label: "Benchmark Main Account".to_string(),
            address: wallet_address.clone(),
        };
        manager.add_subaccount(&wallet_address, subaccount)?;

        Ok(BenchmarkEnvironment {
            manager,
            wallet_address,
            subaccount_id,
            eth_asset_id,
            usdc_asset_id,
        })
    }

    fn setup_initial_balance(&self, amount: u64) -> Result<()> {
        let deposit = Deposit {
            asset_id: self.eth_asset_id.clone(),
            amount,
            deposit_id: "initial_setup".to_string(),
            transaction_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
            block_number: "12345".to_string(),
            from_address: "0x1111111111111111111111111111111111111111".to_string(),
            to_address: self.wallet_address.clone(),
        };

        self.manager.inbox_deposit(&self.wallet_address, deposit)?;
        self.manager
            .claim_inbox(&self.wallet_address, "initial_setup", &self.subaccount_id)?;

        Ok(())
    }
}

/// Measure execution time of a closure
fn measure_time<F, R>(f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;

    #[test]
    fn benchmark_wallet_creation() -> Result<()> {
        let config = BenchmarkConfig::default();
        let mut durations = Vec::with_capacity(config.num_operations);

        println!("Benchmarking wallet creation...");

        // Warmup
        for _ in 0..config.warmup_operations {
            let kms = Arc::new(Mutex::new(EnclaveKMS::new("warmup_secret")?));
            let manager = PassWalletManager::new(kms);
            let _ = manager.create_wallet("Warmup".to_string(), "warmup".to_string());
        }

        // Actual benchmark
        for i in 0..config.num_operations {
            let (_, duration) = measure_time(|| -> Result<()> {
                let kms = Arc::new(Mutex::new(EnclaveKMS::new(&format!("secret_{}", i))?));
                let manager = PassWalletManager::new(kms);
                let _ = manager.create_wallet(format!("Wallet_{}", i), format!("user_{}", i))?;
                Ok(())
            });
            durations.push(duration);
        }

        let result = BenchmarkResult::new("Wallet Creation", &durations);
        result.print_summary();

        // Assert reasonable performance (should create at least 10 wallets per second)
        assert!(
            result.operations_per_second > 10.0,
            "Wallet creation too slow: {} ops/sec",
            result.operations_per_second
        );

        Ok(())
    }

    #[test]
    fn benchmark_claim_operations() -> Result<()> {
        let env = BenchmarkEnvironment::new()?;
        let config = BenchmarkConfig::default();
        let mut durations = Vec::with_capacity(config.num_operations);

        println!("Benchmarking claim operations...");

        // Setup deposits for benchmarking
        for i in 0..config.num_operations + config.warmup_operations {
            let deposit = Deposit {
                asset_id: env.eth_asset_id.clone(),
                amount: 1000000000000000, // 0.001 ETH
                deposit_id: format!("bench_deposit_{}", i),
                transaction_hash: format!("0x{:064}", i),
                block_number: "12345".to_string(),
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: env.wallet_address.clone(),
            };
            env.manager.inbox_deposit(&env.wallet_address, deposit)?;
        }

        // Warmup
        for i in 0..config.warmup_operations {
            let _ = env.manager.claim_inbox(
                &env.wallet_address,
                &format!("bench_deposit_{}", i),
                &env.subaccount_id,
            );
        }

        // Actual benchmark
        for i in config.warmup_operations..(config.warmup_operations + config.num_operations) {
            let (_, duration) = measure_time(|| {
                env.manager
                    .claim_inbox(
                        &env.wallet_address,
                        &format!("bench_deposit_{}", i),
                        &env.subaccount_id,
                    )
                    .unwrap();
            });
            durations.push(duration);
        }

        let result = BenchmarkResult::new("Claim Operations", &durations);
        result.print_summary();

        // Assert reasonable performance
        assert!(
            result.operations_per_second > 100.0,
            "Claim operations too slow: {} ops/sec",
            result.operations_per_second
        );

        Ok(())
    }

    #[test]
    fn benchmark_transfer_operations() -> Result<()> {
        let env = BenchmarkEnvironment::new()?;
        let config = BenchmarkConfig::default();

        // Setup initial large balance
        env.setup_initial_balance(10000000000000000000)?; // 10 ETH - enough for all transfers

        // Add second subaccount for transfers
        let trading_subaccount = Subaccount {
            id: "bench_trading".to_string(),
            label: "Benchmark Trading Account".to_string(),
            address: env.wallet_address.clone(),
        };
        env.manager
            .add_subaccount(&env.wallet_address, trading_subaccount)?;

        let mut durations = Vec::with_capacity(config.num_operations);

        println!("Benchmarking transfer operations...");

        // Warmup
        for _ in 0..config.warmup_operations {
            let _ = env.manager.internal_transfer(
                &env.wallet_address,
                &env.eth_asset_id,
                1000000000000000, // 0.001 ETH
                &env.subaccount_id,
                "bench_trading",
            );
        }

        // Actual benchmark
        for _ in 0..config.num_operations {
            let (_, duration) = measure_time(|| {
                env.manager
                    .internal_transfer(
                        &env.wallet_address,
                        &env.eth_asset_id,
                        1000000000000000, // 0.001 ETH
                        &env.subaccount_id,
                        "bench_trading",
                    )
                    .unwrap();
            });
            durations.push(duration);
        }

        let result = BenchmarkResult::new("Transfer Operations", &durations);
        result.print_summary();

        // Assert reasonable performance
        assert!(
            result.operations_per_second > 500.0,
            "Transfer operations too slow: {} ops/sec",
            result.operations_per_second
        );

        Ok(())
    }

    #[test]
    fn benchmark_withdraw_operations() -> Result<()> {
        let env = BenchmarkEnvironment::new()?;
        let config = BenchmarkConfig::default();

        // Setup initial large balance
        env.setup_initial_balance(10000000000000000000)?; // 10 ETH - enough for all withdrawals

        let mut durations = Vec::with_capacity(config.num_operations);

        println!("Benchmarking withdraw operations...");

        // Warmup
        for _ in 0..config.warmup_operations {
            let _ = env.manager.withdraw(
                &env.wallet_address,
                &env.eth_asset_id,
                1000000000000000, // 0.001 ETH
                &env.subaccount_id,
                "0x1111111111111111111111111111111111111111",
            );
        }

        // Actual benchmark
        for i in 0..config.num_operations {
            let destination = format!("0x{:040}", i); // Generate unique destination
            let (_, duration) = measure_time(|| {
                env.manager
                    .withdraw(
                        &env.wallet_address,
                        &env.eth_asset_id,
                        1000000000000000, // 0.001 ETH
                        &env.subaccount_id,
                        &destination,
                    )
                    .unwrap();
            });
            durations.push(duration);
        }

        let result = BenchmarkResult::new("Withdraw Operations", &durations);
        result.print_summary();

        // Assert reasonable performance
        assert!(
            result.operations_per_second > 500.0,
            "Withdraw operations too slow: {} ops/sec",
            result.operations_per_second
        );

        Ok(())
    }

    #[test]
    fn benchmark_balance_queries() -> Result<()> {
        let env = BenchmarkEnvironment::new()?;
        let config = BenchmarkConfig::default();

        env.setup_initial_balance(1000000000000000000)?; // 1 ETH

        let mut durations = Vec::with_capacity(config.num_operations);

        println!("Benchmarking balance query operations...");

        // Warmup
        for _ in 0..config.warmup_operations {
            let _ =
                env.manager
                    .get_balance(&env.wallet_address, &env.subaccount_id, &env.eth_asset_id);
        }

        // Actual benchmark
        for _ in 0..config.num_operations {
            let (_, duration) = measure_time(|| {
                env.manager
                    .get_balance(&env.wallet_address, &env.subaccount_id, &env.eth_asset_id)
                    .unwrap();
            });
            durations.push(duration);
        }

        let result = BenchmarkResult::new("Balance Query Operations", &durations);
        result.print_summary();

        // Assert high performance for read operations
        assert!(
            result.operations_per_second > 10000.0,
            "Balance queries too slow: {} ops/sec",
            result.operations_per_second
        );

        Ok(())
    }

    #[test]
    fn benchmark_concurrent_operations() -> Result<()> {
        let config = BenchmarkConfig::default();
        let num_threads = config.num_concurrent_operations;
        let operations_per_thread = config.num_operations / num_threads;

        println!(
            "Benchmarking concurrent operations with {} threads...",
            num_threads
        );

        let start_time = Instant::now();

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                thread::spawn(move || -> Result<()> {
                    let env = BenchmarkEnvironment::new()?;
                    env.setup_initial_balance(10000000000000000000)?; // 10 ETH per thread

                    // Add trading subaccount
                    let trading_subaccount = Subaccount {
                        id: format!("trading_{}", thread_id),
                        label: format!("Trading Account {}", thread_id),
                        address: env.wallet_address.clone(),
                    };
                    env.manager
                        .add_subaccount(&env.wallet_address, trading_subaccount)?;

                    // Perform operations
                    for i in 0..operations_per_thread {
                        // Alternating transfers and withdrawals
                        if i % 2 == 0 {
                            env.manager.internal_transfer(
                                &env.wallet_address,
                                &env.eth_asset_id,
                                1000000000000000, // 0.001 ETH
                                &env.subaccount_id,
                                &format!("trading_{}", thread_id),
                            )?;
                        } else {
                            env.manager.withdraw(
                                &env.wallet_address,
                                &env.eth_asset_id,
                                1000000000000000, // 0.001 ETH
                                &env.subaccount_id,
                                &format!("0x{:040}", thread_id * 1000 + i),
                            )?;
                        }
                    }

                    Ok(())
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap()?;
        }

        let total_duration = start_time.elapsed();
        let total_operations = config.num_operations;
        let operations_per_second = total_operations as f64 / total_duration.as_secs_f64();

        println!("\n=== Concurrent Operations Benchmark ===");
        println!("Threads: {}", num_threads);
        println!("Operations per thread: {}", operations_per_thread);
        println!("Total operations: {}", total_operations);
        println!("Total duration: {:?}", total_duration);
        println!("Operations/second: {:.2}", operations_per_second);
        println!("=======================================\n");

        // Assert reasonable concurrent performance
        assert!(
            operations_per_second > 50.0,
            "Concurrent operations too slow: {} ops/sec",
            operations_per_second
        );

        Ok(())
    }

    #[test]
    fn benchmark_end_to_end_workflow() -> Result<()> {
        let config = BenchmarkConfig {
            num_operations: 100,
            ..Default::default()
        };
        let mut durations = Vec::with_capacity(config.num_operations);

        println!("Benchmarking end-to-end workflow operations...");

        // Warmup
        for i in 0..config.warmup_operations {
            let env = BenchmarkEnvironment::new()?;
            let _ = perform_full_workflow(&env, i);
        }

        // Actual benchmark
        for i in 0..config.num_operations {
            let env = BenchmarkEnvironment::new()?;

            let (_, duration) = measure_time(|| {
                perform_full_workflow(&env, i).unwrap();
            });
            durations.push(duration);
        }

        let result = BenchmarkResult::new("End-to-End Workflow", &durations);
        result.print_summary();

        // Assert reasonable end-to-end performance
        assert!(
            result.operations_per_second > 10.0,
            "End-to-end workflow too slow: {} ops/sec",
            result.operations_per_second
        );

        Ok(())
    }

    #[test]
    fn benchmark_provenance_queries() -> Result<()> {
        let env = BenchmarkEnvironment::new()?;
        let config = BenchmarkConfig::default();

        // Setup complex history
        env.setup_initial_balance(1000000000000000000)?;

        // Create transaction history
        for i in 0..100 {
            let deposit = Deposit {
                asset_id: env.eth_asset_id.clone(),
                amount: 1000000000000000,
                deposit_id: format!("prov_deposit_{}", i),
                transaction_hash: format!("0x{:064}", i),
                block_number: "12345".to_string(),
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: env.wallet_address.clone(),
            };
            env.manager.inbox_deposit(&env.wallet_address, deposit)?;
            env.manager.claim_inbox(
                &env.wallet_address,
                &format!("prov_deposit_{}", i),
                &env.subaccount_id,
            )?;
        }

        let mut durations = Vec::with_capacity(config.num_operations);

        println!("Benchmarking provenance query operations...");

        // Warmup
        for _ in 0..config.warmup_operations {
            let _ = env.manager.get_provenance_log(&env.wallet_address);
        }

        // Actual benchmark
        for _ in 0..config.num_operations {
            let (_, duration) = measure_time(|| {
                env.manager.get_provenance_log(&env.wallet_address).unwrap();
            });
            durations.push(duration);
        }

        let result = BenchmarkResult::new("Provenance Query Operations", &durations);
        result.print_summary();

        // Assert reasonable performance for provenance queries
        assert!(
            result.operations_per_second > 1000.0,
            "Provenance queries too slow: {} ops/sec",
            result.operations_per_second
        );

        Ok(())
    }

    #[test]
    fn benchmark_memory_usage() -> Result<()> {
        println!("Benchmarking memory usage with large datasets...");

        let env = BenchmarkEnvironment::new()?;
        let large_dataset_size = 10000;

        let start_time = Instant::now();

        // Create large number of deposits and claims
        for i in 0..large_dataset_size {
            let deposit = Deposit {
                asset_id: env.eth_asset_id.clone(),
                amount: 1000000000000000, // 0.001 ETH
                deposit_id: format!("memory_test_{}", i),
                transaction_hash: format!("0x{:064}", i),
                block_number: format!("{}", 12345 + i),
                from_address: format!("0x{:040}", i),
                to_address: env.wallet_address.clone(),
            };

            env.manager.inbox_deposit(&env.wallet_address, deposit)?;
            env.manager.claim_inbox(
                &env.wallet_address,
                &format!("memory_test_{}", i),
                &env.subaccount_id,
            )?;

            // Print progress every 1000 operations
            if (i + 1) % 1000 == 0 {
                println!("Processed {} operations", i + 1);
            }
        }

        let total_duration = start_time.elapsed();
        let operations_per_second = large_dataset_size as f64 / total_duration.as_secs_f64();

        println!("\n=== Memory Usage Benchmark ===");
        println!("Total operations: {}", large_dataset_size);
        println!("Total duration: {:?}", total_duration);
        println!("Operations/second: {:.2}", operations_per_second);
        println!("==============================\n");

        // Verify the wallet state is still functional
        let wallet_state = env.manager.get_wallet(&env.wallet_address).unwrap();
        assert_eq!(wallet_state.history.len(), large_dataset_size);

        let final_balance =
            env.manager
                .get_balance(&env.wallet_address, &env.subaccount_id, &env.eth_asset_id)?;
        let expected_balance = large_dataset_size as u64 * 1000000000000000;
        assert_eq!(final_balance, expected_balance);

        Ok(())
    }
}

/// Helper function to perform a complete workflow
fn perform_full_workflow(env: &BenchmarkEnvironment, iteration: usize) -> Result<()> {
    // Add trading subaccount
    let trading_subaccount = Subaccount {
        id: format!("trading_{}", iteration),
        label: format!("Trading Account {}", iteration),
        address: env.wallet_address.clone(),
    };
    env.manager
        .add_subaccount(&env.wallet_address, trading_subaccount)?;

    // Step 1: Deposit
    let deposit = Deposit {
        asset_id: env.eth_asset_id.clone(),
        amount: 5000000000000000000, // 5 ETH
        deposit_id: format!("workflow_deposit_{}", iteration),
        transaction_hash: format!("0x{:064}", iteration),
        block_number: "12345".to_string(),
        from_address: "0x1111111111111111111111111111111111111111".to_string(),
        to_address: env.wallet_address.clone(),
    };
    env.manager.inbox_deposit(&env.wallet_address, deposit)?;

    // Step 2: Claim
    env.manager.claim_inbox(
        &env.wallet_address,
        &format!("workflow_deposit_{}", iteration),
        &env.subaccount_id,
    )?;

    // Step 3: Transfer
    env.manager.internal_transfer(
        &env.wallet_address,
        &env.eth_asset_id,
        2000000000000000000, // 2 ETH
        &env.subaccount_id,
        &format!("trading_{}", iteration),
    )?;

    // Step 4: Withdraw
    env.manager.withdraw(
        &env.wallet_address,
        &env.eth_asset_id,
        1000000000000000000, // 1 ETH
        &env.subaccount_id,
        &format!("0x{:040}", iteration),
    )?;

    // Step 5: Process outbox
    env.manager.process_outbox(&env.wallet_address)?;

    Ok(())
}
