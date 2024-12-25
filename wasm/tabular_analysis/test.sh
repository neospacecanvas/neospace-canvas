#!/bin/bash
# test.sh - Local test script

# Exit on any error
set -e

echo "Running Rust tests..."
cargo test

echo "Running WASM tests..."
wasm-pack test --node

# The script will exit with non-zero status if any command fails
# due to the set -e flag
