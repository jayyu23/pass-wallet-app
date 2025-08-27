# Server Enclave PASS Wallet Test Suite

This directory contains comprehensive tests for the PASS Wallet functionality within the Server Enclave. The test suite covers functionality testing, performance benchmarking, and integration testing for the end-to-end flow of claim, transfer, and withdraw operations.

**📁 All testing-related files are organized in this `tests/` directory:**
- Test files (`.rs`)
- Test utilities (`mod.rs`)
- Test configuration (`test_config.toml`)
- Throughput analysis tools (`throughput_report.sh`)
- Documentation (this `README.md`)

## Test Structure

### 1. Functionality Tests (`functionality_tests.rs`)

Comprehensive unit and integration tests covering all core PASS wallet operations:

- **Wallet Creation**: Tests wallet creation with proper initialization
- **Claim Operations**: Tests claiming deposits from inbox to subaccounts
- **Transfer Operations**: Tests internal transfers between subaccounts
- **Withdraw Operations**: Tests withdrawals to external addresses
- **End-to-End Workflow**: Complete workflow testing multiple operations
- **Error Scenarios**: Tests for insufficient balance, duplicate deposits, etc.
- **Multi-Asset Support**: Tests with different asset types (ETH, ERC20)
- **Provenance Tracking**: Tests transaction history and filtering

#### Key Test Cases:
- `test_wallet_creation()` - Verifies wallet setup
- `test_claim_deposit_flow()` - Tests deposit claiming process
- `test_internal_transfer_flow()` - Tests transfers between subaccounts
- `test_withdraw_flow()` - Tests withdrawal to external addresses
- `test_end_to_end_workflow()` - Complete multi-operation workflow
- `test_insufficient_balance_scenarios()` - Error handling tests
- `test_multi_asset_balances()` - Multi-asset functionality
- `test_provenance_filtering()` - Transaction history filtering

### 2. Benchmark Tests (`benchmark_tests.rs`)

Performance testing suite to measure and validate system performance:

- **Wallet Creation Benchmarks**: Measures wallet creation performance
- **Operation Benchmarks**: Measures claim, transfer, withdraw performance
- **Concurrent Operations**: Tests performance under concurrent load
- **Memory Usage**: Tests system behavior with large datasets
- **Query Performance**: Measures balance and provenance query performance

#### Key Benchmarks:
- `benchmark_wallet_creation()` - Wallet creation speed
- `benchmark_claim_operations()` - Claim operation throughput
- `benchmark_transfer_operations()` - Transfer operation speed
- `benchmark_withdraw_operations()` - Withdrawal operation performance
- `benchmark_balance_queries()` - Query performance testing
- `benchmark_concurrent_operations()` - Concurrent load testing
- `benchmark_end_to_end_workflow()` - Complete workflow performance
- `benchmark_memory_usage()` - Large dataset handling

#### Performance Thresholds:
- Balance queries: > 100,000 ops/sec
- Transfer operations: > 10,000 ops/sec
- Withdrawal operations: > 5,000 ops/sec
- Claim operations: > 5,000 ops/sec
- End-to-end workflows: > 5,000 ops/sec
- Wallet creation: > 100 ops/sec
- Concurrent operations: > 1,000 ops/sec
- Provenance queries: > 500 ops/sec

### 3. Integration Tests (`integration_tests.rs`)

Real-world scenario testing with complex workflows:

- **Multi-User Trading**: Simulates multiple users with complex trading patterns
- **Enterprise Workflows**: Complex multi-department asset management
- **High-Frequency Transactions**: Tests system under high transaction volume
- **Cross-Wallet Simulations**: Ecosystem-level testing with multiple wallets
- **Error Recovery**: Tests system resilience and error handling

#### Key Integration Tests:
- `test_multi_user_trading_scenario()` - Multi-user trading simulation
- `test_complex_multi_asset_workflow()` - Enterprise asset management
- `test_high_frequency_micro_transactions()` - HFT simulation
- `test_cross_wallet_simulation()` - Ecosystem-wide testing
- `test_error_recovery_scenarios()` - Error handling and recovery

## Running Tests

### Prerequisites

1. Ensure Rust is installed with the correct version:
   ```bash
   rustc --version  # Should be compatible with the project
   ```

2. Navigate to the server-enclave directory:
   ```bash
   cd /path/to/pass-wallet-app/server-enclave
   ```

### Running All Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode for better performance
cargo test --release
```

### Running Specific Test Suites

```bash
# Run only functionality tests
cargo test functionality_tests

# Run only benchmark tests
cargo test benchmark_tests

# Run only integration tests
cargo test integration_tests
```

### Running Individual Tests

```bash
# Run a specific functionality test
cargo test test_end_to_end_workflow

# Run a specific benchmark
cargo test benchmark_wallet_creation

# Run a specific integration test
cargo test test_multi_user_trading_scenario
```

### Benchmark-Specific Commands

```bash
# Run benchmarks with release optimizations
cargo test benchmark --release

# Run benchmarks with detailed timing
cargo test benchmark --release -- --nocapture

# Run memory usage tests (may take longer)
cargo test benchmark_memory_usage --release -- --nocapture
```

## Test Configuration

### Environment Variables

The tests use the following environment variables (with defaults):

- `ENCLAVE_SECRET`: Secret for KMS initialization (default: "test_secret")
- `RUST_LOG`: Logging level (default: not set)

### Benchmark Configuration

Benchmarks can be configured by modifying the `BenchmarkConfig` struct in `benchmark_tests.rs`:

```rust
BenchmarkConfig {
    num_operations: 1000,        // Number of operations to benchmark
    num_concurrent_operations: 10, // Number of concurrent threads
    warmup_operations: 100,      // Warmup operations before measurement
}
```

## Test Data and Scenarios

### Asset Types Tested

1. **ETH (Native Ethereum)**
   - Symbol: ETH
   - Decimals: 18
   - Contract: None

2. **USDC (ERC20 Token)**
   - Symbol: USDC
   - Decimals: 6
   - Contract: 0xa0b86a33e6776e7bb8c4c9f8d9b2d5f1c4e3f1d2

3. **WBTC (ERC20 Token)**
   - Symbol: WBTC
   - Decimals: 8
   - Contract: 0x2260fac5e5542a773aa44fbcfedf7c193bc2c599

### Test Scenarios

1. **Basic Operations**: Simple claim, transfer, withdraw flows
2. **Multi-Asset**: Operations involving multiple asset types
3. **High Volume**: Large numbers of micro-transactions
4. **Enterprise**: Complex multi-department workflows
5. **Error Cases**: Invalid operations and edge cases
6. **Concurrent**: Multiple operations happening simultaneously

## Expected Results

### Functionality Tests
All functionality tests should pass, demonstrating:
- Correct balance tracking
- Proper transaction history recording
- Error handling for invalid operations
- Multi-asset support
- Provenance tracking accuracy

### Benchmark Results
Benchmark tests should meet performance thresholds:
- Operations should complete within expected timeframes
- Memory usage should remain reasonable under load
- Concurrent operations should scale appropriately

### Integration Tests
Integration tests should demonstrate:
- System stability under complex workflows
- Correct behavior across multiple wallets
- Proper error recovery and state consistency
- Realistic scenario handling

## Troubleshooting

### Common Issues

1. **Test Timeouts**
   - Increase timeout values for slower systems
   - Run with `--release` flag for better performance

2. **Memory Issues**
   - Reduce benchmark dataset sizes for memory-constrained systems
   - Run individual test suites instead of all tests at once

3. **Compilation Errors**
   - Ensure all dependencies are properly installed
   - Check Rust version compatibility

4. **Failed Assertions**
   - Review test output for specific failure details
   - Check that test data matches expected values

### Debug Mode

For detailed debugging, set the log level:

```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Performance Analysis

For detailed performance analysis:

```bash
# Run all benchmarks with detailed output
cargo test benchmark_tests --tests -- --nocapture

# Generate throughput report  
cd tests/ && ./throughput_report.sh

# Run specific operation benchmarks
cargo test benchmark_claim_operations --tests -- --nocapture
cargo test benchmark_transfer_operations --tests -- --nocapture
cargo test benchmark_withdraw_operations --tests -- --nocapture

# Run with release mode for production performance
cargo test benchmark --release -- --nocapture | tee benchmark_results.txt
```

#### Throughput Testing Tools

The `throughput_report.sh` script provides a quick summary of all operation throughput:

```bash
cd tests/
./throughput_report.sh
```

This generates a comprehensive report showing operations/second for all core PASS wallet operations.

## Key Differences from Nitro Enclave Tests

1. **Module References**: Uses `server_enclave::` instead of `nitro_enclave::`
2. **Same Test Logic**: All test scenarios and assertions remain identical
3. **Compatibility**: Fully compatible with the server-enclave architecture
4. **Performance**: Should show similar or better performance due to reduced overhead

## Contributing

When adding new tests:

1. Follow the existing naming conventions
2. Add appropriate documentation
3. Include both positive and negative test cases
4. Update this README with new test descriptions
5. Ensure tests are deterministic and repeatable

### Test Categories

- **Unit Tests**: Test individual functions and components
- **Integration Tests**: Test component interactions
- **End-to-End Tests**: Test complete workflows
- **Performance Tests**: Measure and validate performance
- **Error Tests**: Test error conditions and recovery

## Continuous Integration

These tests are designed to be run in CI/CD pipelines:

- All tests should be deterministic
- No external dependencies required
- Configurable for different environments
- Clear pass/fail criteria
- Performance regression detection