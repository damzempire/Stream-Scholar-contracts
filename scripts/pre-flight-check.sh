#!/bin/bash

# Pre-Flight Checklist for Stream-Scholar Mainnet Deployment
# This script performs a "Dry-Run" deployment simulation on a local Mainnet fork.

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

LOG_FILE="pre-flight-check.log"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a $LOG_FILE
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a $LOG_FILE
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a $LOG_FILE
}

# --- Setup ---
log_info "Starting Pre-Flight Checklist..."

# Clean up previous logs
rm -f $LOG_FILE

# Start local Soroban network with Mainnet fork
log_info "Starting local Soroban network with Mainnet fork..."
soroban network start --mainnet-fork

# Wait for network to be ready
sleep 10

log_success "Local Mainnet fork started."

# Generate temporary admin keypair
log_info "Generating temporary admin keypair..."
ADMIN_SECRET=$(soroban keys generate --no-fund --quiet)
ADMIN_ADDRESS=$(soroban keys address $ADMIN_SECRET)
log_success "Admin keypair generated: $ADMIN_ADDRESS"

# Fund admin account
log_info "Funding admin account..."
if [ -z "$MAINNET_FUNDING_SECRET" ]; then
    log_error "MAINNET_FUNDING_SECRET environment variable is not set."
    log_error "Please set it to a funded Mainnet secret key to proceed."
    exit 1
fi

soroban keys fund $ADMIN_ADDRESS --secret $MAINNET_FUNDING_SECRET --network mainnet-fork
log_success "Admin account funded."

# Build contracts
log_info "Building contracts..."
cargo build --release --target wasm32v1-none
log_success "Contracts built."

# Deploy scholar contract
log_info "Deploying scholar contract..."
SCHOLAR_WASM_HASH=$(soroban contract install --wasm target/wasm32v1-none/release/scholar_contracts.wasm)
SCHOLAR_ID=$(soroban contract deploy --wasm-hash $SCHOLAR_WASM_HASH --source $ADMIN_SECRET --network mainnet-fork)
log_success "Scholar contract deployed: $SCHOLAR_ID"

# Initialize scholar contract
log_info "Initializing scholar contract..."
soroban contract invoke --id $SCHOLAR_ID --source $ADMIN_SECRET --network mainnet-fork -- init --base_rate 100 --discount_threshold 3600 --discount_percentage 10 --min_deposit 50 --heartbeat_interval 300
log_success "Scholar contract initialized."

# Deploy mock USDC token
log_info "Deploying mock USDC token..."
TOKEN_WASM_HASH=$(soroban contract install --wasm target/wasm32v1-none/release/soroban_token_contract.wasm)
TOKEN_ID=$(soroban contract deploy --wasm-hash $TOKEN_WASM_HASH --source $ADMIN_SECRET --network mainnet-fork)
soroban contract invoke --id $TOKEN_ID --source $ADMIN_SECRET --network mainnet-fork -- initialize --admin $ADMIN_ADDRESS --decimal 7 --name "USD Coin" --symbol USDC
log_success "Mock USDC token deployed: $TOKEN_ID"

# Mint mock USDC to admin
log_info "Minting mock USDC to admin..."
soroban contract invoke --id $TOKEN_ID --source $ADMIN_SECRET --network mainnet-fork -- mint --to $ADMIN_ADDRESS --amount 1000000000000 # 100,000 USDC
log_success "100,000 mock USDC minted to admin."

# --- Verification ---
log_info "Starting verification..."

# Get initial balances
log_info "Getting initial balances..."
ADMIN_INITIAL_XLM_BALANCE=$(soroban keys balance $ADMIN_ADDRESS --network mainnet-fork)
ADMIN_INITIAL_USDC_BALANCE=$(soroban contract invoke --id $TOKEN_ID --source $ADMIN_ADDRESS --network mainnet-fork -- balance --id $ADMIN_ADDRESS)
log_success "Initial balances recorded."

# --- Simulation ---
log_info "Starting simulation..."

# Student Enrollment
log_info "Simulating 50 student enrollments..."
for i in {1..50}; do
    log_info "Enrolling student $i..."
    STUDENT_SECRET_$(echo $i)=$(soroban keys generate --no-fund --quiet)
    STUDENT_ADDRESS_$(echo $i)=$(soroban keys address $(eval echo \$STUDENT_SECRET_$i))
    soroban keys fund $(eval echo \$STUDENT_ADDRESS_$i) --source $ADMIN_SECRET --network mainnet-fork
    soroban contract invoke --id $SCHOLAR_ID --source $ADMIN_SECRET --network mainnet-fork -- fund_scholarship --funder $ADMIN_ADDRESS --student $(eval echo \$STUDENT_ADDRESS_$i) --amount 10000000000 --token $TOKEN_ID --is_native false
    log_success "Student $i enrolled."
done

# Probation Cases
log_info "Simulating 5 probation cases..."
for i in {1..5}; do
    log_info "Putting student $i on probation..."
    STUDENT_ADDRESS=$(soroban keys address $(eval echo \$STUDENT_SECRET_$i))
    soroban contract invoke --id $SCHOLAR_ID --source $ADMIN_SECRET --network mainnet-fork -- pause_scholarship --admin $ADMIN_ADDRESS --student $STUDENT_ADDRESS --reason "academic_probation"
    log_success "Student $i is on probation."
    log_info "Unpausing student $i..."
    soroban contract invoke --id $SCHOLAR_ID --source $ADMIN_SECRET --network mainnet-fork -- unpause_scholarship --admin $ADMIN_ADDRESS --student $STUDENT_ADDRESS
    log_success "Student $i is no longer on probation."
done

# Graduations
log_info "Simulating 2 graduations..."
for i in {6..7}; do
    log_info "Graduating student $i..."
    STUDENT_ADDRESS=$(soroban keys address $(eval echo \$STUDENT_SECRET_$i))
    soroban contract invoke --id $SCHOLAR_ID --source $(eval echo \$STUDENT_SECRET_$i) --network mainnet-fork -- initiate_final_release_vote --student $STUDENT_ADDRESS
    for j in {1..5}; do
        VOTER_SECRET=$(soroban keys generate --no-fund --quiet)
        VOTER_ADDRESS=$(soroban keys address $VOTER_SECRET)
        soroban keys fund $VOTER_ADDRESS --source $ADMIN_SECRET --network mainnet-fork
        soroban contract invoke --id $SCHOLAR_ID --source $VOTER_SECRET --network mainnet-fork -- cast_community_vote --voter $VOTER_ADDRESS --student $STUDENT_ADDRESS
    done
    soroban contract invoke --id $SCHOLAR_ID --source $(eval echo \$STUDENT_SECRET_$i) --network mainnet-fork -- claim_final_release --student $STUDENT_ADDRESS
    log_success "Student $i graduated."
done

# --- Verification ---
log_info "Verifying balances..."

# Get final balances
log_info "Getting final balances..."
ADMIN_FINAL_XLM_BALANCE=$(soroban keys balance $ADMIN_ADDRESS --network mainnet-fork)
ADMIN_FINAL_USDC_BALANCE=$(soroban contract invoke --id $TOKEN_ID --source $ADMIN_ADDRESS --network mainnet-fork -- balance --id $ADMIN_ADDRESS)
log_success "Final balances recorded."

# Reconcile balances
log_info "Reconciling balances..."

# Calculate expected balances
log_info "Calculating expected balances..."
EXPECTED_ADMIN_USDC_BALANCE=$((ADMIN_INITIAL_USDC_BALANCE - 50 * 10000000000))
EXPECTED_CONTRACT_USDC_BALANCE=$((50 * 10000000000))
log_success "Expected balances calculated."

# Verify balances
log_info "Verifying balances..."
if [ "$ADMIN_FINAL_USDC_BALANCE" -ne "$EXPECTED_ADMIN_USDC_BALANCE" ]; then
    log_error "Admin USDC balance mismatch. Expected: $EXPECTED_ADMIN_USDC_BALANCE, Actual: $ADMIN_FINAL_USDC_BALANCE"
    exit 1
fi

CONTRACT_FINAL_USDC_BALANCE=$(soroban contract invoke --id $TOKEN_ID --source $ADMIN_ADDRESS --network mainnet-fork -- balance --id $SCHOLAR_ID)
if [ "$CONTRACT_FINAL_USDC_BALANCE" -ne "$EXPECTED_CONTRACT_USDC_BALANCE" ]; then
    log_error "Contract USDC balance mismatch. Expected: $EXPECTED_CONTRACT_USDC_BALANCE, Actual: $CONTRACT_FINAL_USDC_BALANCE"
    exit 1
fi

log_success "Balances verified."

# --- Reporting ---
log_info "Generating report..."

# Get gas consumption
GAS_CONSUMED=$(grep "soroban contract invoke" $LOG_FILE | grep -o "cpu_insns_consumed: [0-9]*" | awk '{sum += $2} END {print sum}')

# Get transaction success rate
TOTAL_TRANSACTIONS=$(grep "soroban contract invoke" $LOG_FILE | wc -l)
SUCCESSFUL_TRANSACTIONS=$(grep "soroban contract invoke" $LOG_FILE | grep -v "error" | wc -l)
SUCCESS_RATE=$((SUCCESSFUL_TRANSACTIONS * 100 / TOTAL_TRANSACTIONS))

# Generate report
REPORT_FILE="pre-flight-check-report.txt"
echo "--- Pre-Flight Checklist Report ---" > $REPORT_FILE
echo "" >> $REPORT_FILE
echo "Date: $(date)" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "--- Summary ---" >> $REPORT_FILE
echo "Status: PASS" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "--- Metrics ---" >> $REPORT_FILE
echo "Gas Consumed: $GAS_CONSUMED" >> $REPORT_FILE
echo "Transaction Success Rate: $SUCCESS_RATE%" >> $REPORT_FILE
echo "Balance Discrepancies: 0" >> $REPORT_FILE

log_success "Report generated: $REPORT_FILE"

log_success "Pre-Flight Checklist completed."
