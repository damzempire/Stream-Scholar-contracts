#!/bin/bash

# Setup Local Test Network with Mock Content
# Pre-loads 5 dummy courses and 100 test USDC

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Start local Soroban network
log_info "Starting local Soroban network..."
soroban network start --standalone

# Wait for network to be ready
sleep 5

# Generate keys
log_info "Generating keys..."
ADMIN_SECRET=$(soroban keys generate --no-fund --quiet)
ADMIN_ADDRESS=$(soroban keys address $ADMIN_SECRET)

TEACHER_SECRET=$(soroban keys generate --no-fund --quiet)
TEACHER_ADDRESS=$(soroban keys address $TEACHER_SECRET)

STUDENT_SECRET=$(soroban keys generate --no-fund --quiet)
STUDENT_ADDRESS=$(soroban keys address $STUDENT_SECRET)

log_success "Keys generated"

# Fund accounts
log_info "Funding accounts..."
soroban keys fund $ADMIN_ADDRESS
soroban keys fund $TEACHER_ADDRESS
soroban keys fund $STUDENT_ADDRESS

log_success "Accounts funded"

# Build contracts
log_info "Building contracts..."
cargo build --release --target wasm32v1-none

log_success "Contracts built"

# Deploy token contract for USDC
log_info "Deploying USDC token contract..."
TOKEN_WASM_HASH=$(soroban contract install --wasm target/wasm32v1-none/release/soroban_token_contract.wasm)
TOKEN_ID=$(soroban contract deploy --wasm-hash $TOKEN_WASM_HASH --source $ADMIN_SECRET --network standalone)

# Initialize token
soroban contract invoke --id $TOKEN_ID --source $ADMIN_SECRET --network standalone -- initialize --admin $ADMIN_ADDRESS --decimal 7 --name "USD Coin" --symbol USDC

# Mint 100 USDC to student
soroban contract invoke --id $TOKEN_ID --source $ADMIN_SECRET --network standalone -- mint --to $STUDENT_ADDRESS --amount 1000000000  # 100 * 10^7

log_success "USDC token deployed and 100 USDC minted to student"

# Deploy scholar contract
log_info "Deploying scholar contract..."
SCHOLAR_WASM_HASH=$(soroban contract install --wasm target/wasm32v1-none/release/scholar_contracts.wasm)
SCHOLAR_ID=$(soroban contract deploy --wasm-hash $SCHOLAR_WASM_HASH --source $ADMIN_SECRET --network standalone)

log_success "Scholar contract deployed: $SCHOLAR_ID"

# Initialize scholar contract
log_info "Initializing scholar contract..."
soroban contract invoke --id $SCHOLAR_ID --source $ADMIN_SECRET --network standalone -- init --base_rate 100 --discount_threshold 3600 --discount_percentage 10 --min_deposit 50 --heartbeat_interval 300

# Set admin
soroban contract invoke --id $SCHOLAR_ID --source $ADMIN_SECRET --network standalone -- set_admin --admin $ADMIN_ADDRESS

# Set teacher
soroban contract invoke --id $SCHOLAR_ID --source $ADMIN_SECRET --network standalone -- set_teacher --admin $ADMIN_ADDRESS --teacher $TEACHER_ADDRESS --status true

log_success "Scholar contract initialized"

# Add 5 dummy courses
log_info "Adding 5 dummy courses..."
for i in {1..5}; do
    soroban contract invoke --id $SCHOLAR_ID --source $TEACHER_ADDRESS --network standalone -- add_course_to_registry --course_id $i --creator $TEACHER_ADDRESS
done

log_success "5 dummy courses added"

# Set course durations (optional, but for completeness)
for i in {1..5}; do
    soroban contract invoke --id $SCHOLAR_ID --source $ADMIN_SECRET --network standalone -- set_course_duration --course_id $i --duration 3600  # 1 hour
done

log_success "Setup complete!"

# Output important info
echo ""
echo "========================================"
echo "Local Test Network Setup Complete"
echo "========================================"
echo "Network: Standalone"
echo "Scholar Contract ID: $SCHOLAR_ID"
echo "USDC Token ID: $TOKEN_ID"
echo "Admin Address: $ADMIN_ADDRESS"
echo "Admin Secret: $ADMIN_SECRET"
echo "Teacher Address: $TEACHER_ADDRESS"
echo "Teacher Secret: $TEACHER_SECRET"
echo "Student Address: $STUDENT_ADDRESS"
echo "Student Secret: $STUDENT_SECRET"
echo "========================================"

# Keep the network running
log_info "Network is running. Press Ctrl+C to stop."
trap "soroban network stop" EXIT
wait