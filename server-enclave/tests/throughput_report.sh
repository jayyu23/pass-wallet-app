#!/bin/bash

echo "SERVER-ENCLAVE THROUGHPUT REPORT"
echo "================================="
echo "Timestamp: $(date)"
echo ""

# Function to extract throughput from test output
extract_throughput() {
    local test_name=$1
    echo "Running $test_name..."
    
    # Capture test output and extract operations/second (run from parent directory)
    local output=$(cd .. && cargo test $test_name --tests -- --nocapture 2>/dev/null | grep "Operations/Second:")
    echo "$output"
    echo ""
}

echo "Individual Operation Throughput:"
echo "-----------------------------------"

extract_throughput "benchmark_balance_queries"
extract_throughput "benchmark_transfer_operations" 
extract_throughput "benchmark_withdraw_operations"
extract_throughput "benchmark_claim_operations"
extract_throughput "benchmark_wallet_creation"
extract_throughput "benchmark_concurrent_operations"
extract_throughput "benchmark_end_to_end_workflow"
extract_throughput "benchmark_provenance_queries"

echo "Memory Performance:"
echo "---------------------"
extract_throughput "benchmark_memory_usage"

echo ""
echo "Report Complete!"
echo "For detailed metrics, run: cd .. && cargo test benchmark_tests --tests -- --nocapture"