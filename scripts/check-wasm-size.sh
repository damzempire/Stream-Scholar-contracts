#!/bin/bash
# Post-build Wasm size check script (Issue #205)
# Asserts the compiled .wasm file remains under 100KB after aggressive size optimization.
# Release profile settings (Cargo.toml):
#   opt-level = "z"      — optimize for size (smallest binary)
#   lto = true           — Link-Time Optimization (cross-crate dead-code elimination)
#   codegen-units = 1    — single codegen unit for maximum LTO effectiveness
#   strip = "symbols"    — strip debug symbols from the final binary
#   panic = "abort"      — removes panic unwinding machinery
#   debug = 0            — no debug info

set -e

echo "🔨 Building Soroban contract with release optimizations..."
cargo build --release --target wasm32-unknown-unknown

echo "📏 Checking Wasm file size..."

WASM_FILE=$(find target/wasm32-unknown-unknown/release -name "*.wasm" | grep -v deps | head -1)
if [ -z "$WASM_FILE" ]; then
  WASM_FILE=$(find target/wasm32-unknown-unknown/release -name "*.wasm" | head -1)
fi

if [ -z "$WASM_FILE" ]; then
  echo "❌ ERROR: No .wasm file found!"
  exit 1
fi

FILE_SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE")
FILE_SIZE_KB=$(awk "BEGIN {printf \"%.2f\", $FILE_SIZE/1024}")

echo "📁 Wasm file: $WASM_FILE"
echo "📊 File size: $FILE_SIZE bytes ($FILE_SIZE_KB KB)"

# Hard limit: 100KB as required by Issue #205
LIMIT_BYTES=102400
LIMIT_KB=100
REMAINING_BYTES=$((LIMIT_BYTES - FILE_SIZE))
REMAINING_KB=$(awk "BEGIN {printf \"%.2f\", $REMAINING_BYTES/1024}")
UTILIZATION=$(awk "BEGIN {printf \"%.1f\", $FILE_SIZE*100/$LIMIT_BYTES}")

echo ""
echo "## Wasm Size Report"
echo "| Metric | Value |"
echo "|--------|-------|"
echo "| File | \`$WASM_FILE\` |"
echo "| Size | $FILE_SIZE bytes ($FILE_SIZE_KB KB) |"
echo "| Hard Limit (Issue #205) | $LIMIT_BYTES bytes ($LIMIT_KB KB) |"
echo "| Remaining Capacity | $REMAINING_BYTES bytes ($REMAINING_KB KB) |"
echo "| Utilization | $UTILIZATION% |"
echo ""
echo "## Active Optimizations (Cargo.toml [profile.release])"
echo "| Setting | Value | Purpose |"
echo "|---------|-------|---------|"
echo "| opt-level | \"z\" | Optimize for smallest binary size |"
echo "| lto | true | Link-Time Optimization removes dead code |"
echo "| codegen-units | 1 | Single unit maximizes LTO effectiveness |"
echo "| strip | \"symbols\" | Remove debug symbols from binary |"
echo "| panic | \"abort\" | Remove panic unwinding machinery |"
echo "| debug | 0 | No debug information |"

if [ "$FILE_SIZE" -gt "$LIMIT_BYTES" ]; then
  echo ""
  echo "❌ FAIL: Wasm size ($FILE_SIZE_KB KB) exceeds $LIMIT_KB KB limit!"
  echo "   Over by: $((FILE_SIZE - LIMIT_BYTES)) bytes"
  exit 1
else
  echo ""
  echo "✅ PASS: Wasm size ($FILE_SIZE_KB KB) is within the $LIMIT_KB KB limit."
fi
