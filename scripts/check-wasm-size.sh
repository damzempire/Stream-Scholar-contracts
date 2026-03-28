#!/bin/bash

# Wasm Size Checker Script for Soroban Contracts
# This script builds the contract and checks if it's within the 64KB Soroban limit

set -e

echo "🔨 Building Soroban contract for wasm32-unknown-unknown target..."
cargo build --release --target wasm32-unknown-unknown

echo "📏 Checking Wasm file size..."

# Find the built .wasm file
WASM_FILE=$(find target/wasm32-unknown-unknown/release -name "*.wasm" | head -1)

if [ -z "$WASM_FILE" ]; then
  echo "❌ ERROR: No .wasm file found!"
  exit 1
fi

# Get file size in bytes
FILE_SIZE=$(stat -c%s "$WASM_FILE")
FILE_SIZE_KB=$(echo "scale=2; $FILE_SIZE / 1024" | bc)

echo "📁 Wasm file: $WASM_FILE"
echo "📊 File size: $FILE_SIZE bytes ($FILE_SIZE_KB KB)"

# Soroban limit is 64KB (65536 bytes)
LIMIT_BYTES=65536
LIMIT_KB=64
REMAINING_BYTES=$((LIMIT_BYTES - FILE_SIZE))
REMAINING_KB=$(echo "scale=2; $REMAINING_BYTES / 1024" | bc)

if [ $FILE_SIZE -gt $LIMIT_BYTES ]; then
  echo "❌ ERROR: Wasm file size ($FILE_SIZE_KB KB) exceeds Soroban limit of $LIMIT_KB KB!"
  echo "   Current size: $FILE_SIZE bytes"
  echo "   Limit: $LIMIT_BYTES bytes"
  echo "   Over by: $(($FILE_SIZE - $LIMIT_BYTES)) bytes"
  exit 1
else
  echo "✅ SUCCESS: Wasm file size ($FILE_SIZE_KB KB) is within Soroban limit of $LIMIT_KB KB"
  echo "   Remaining capacity: $REMAINING_BYTES bytes ($REMAINING_KB KB)"
fi

# Generate size report
echo ""
echo "## Wasm Size Report"
echo "| Metric | Value |"
echo "|--------|-------|"
echo "| File | \`$WASM_FILE\` |"
echo "| Size | $FILE_SIZE bytes ($FILE_SIZE_KB KB) |"
echo "| Soroban Limit | $LIMIT_BYTES bytes ($LIMIT_KB KB) |"
echo "| Status | ✅ Within limit |"
echo "| Remaining Capacity | $REMAINING_BYTES bytes ($REMAINING_KB KB) |"
