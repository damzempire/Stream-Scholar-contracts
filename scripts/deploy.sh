#!/bin/bash

# Stream Scholar Contracts Deployment Script
# This script handles contract deployment, initialization, and teacher role setup for Stellar Testnet

set -e

# Configuration
NETWORK="testnet"
CONTRACT_NAME="scholar_contracts"
CONTRACT_DIR="contracts/scholar_contracts"
WASM_FILE="target/wasm32v1-none/release/scholar_contracts.wasm"

# Default values for initialization
DEFAULT_BASE_RATE=100
DEFAULT_DISCOUNT_THRESHOLD=3600
DEFAULT_DISCOUNT_PERCENTAGE=10
DEFAULT_MIN_DEPOSIT=50
DEFAULT_HEARTBEAT_INTERVAL=300

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if soroban-cli is installed
check_soroban_cli() {
    if ! command -v soroban &> /dev/null; then
        log_error "soroban-cli is not installed. Please install it first."
        echo "Visit: https://github.com/stellar/soroban-cli"
        exit 1
    fi
    log_success "soroban-cli found"
}

# Function to check if we're on the right network
check_network() {
    log_info "Checking network configuration for $NETWORK"
    
    # Set network if not already set
    if ! soroban config network | grep -q "$NETWORK"; then
        log_info "Setting up $NETWORK network configuration"
        soroban config network add "$NETWORK" \
            --rpc-url "https://soroban-testnet.stellar.org:443" \
            --network-passphrase "Test SDF Network ; September 2015"
    fi
    
    log_success "Network configuration verified"
}

# Function to build the contract
build_contract() {
    log_info "Building contract..."
    
    if [ ! -d "$CONTRACT_DIR" ]; then
        log_error "Contract directory $CONTRACT_DIR not found"
        exit 1
    fi
    
    cd "$CONTRACT_DIR"
    
    # Build the contract
    if ! stellar contract build; then
        log_error "Contract build failed"
        exit 1
    fi
    
    # Check if WASM file was created
    if [ ! -f "$WASM_FILE" ]; then
        log_error "WASM file not found at $WASM_FILE"
        exit 1
    fi
    
    cd - > /dev/null
    log_success "Contract built successfully"
}

# Function to deploy contract
deploy_contract() {
    log_info "Deploying contract to $NETWORK..."
    
    cd "$CONTRACT_DIR"
    
    # Deploy the contract
    CONTRACT_ID=$(soroban contract deploy \
        --wasm "$WASM_FILE" \
        --source "$SOROBAN_ADMIN_ADDRESS" \
        --network "$NETWORK")
    
    if [ $? -ne 0 ]; then
        log_error "Contract deployment failed"
        exit 1
    fi
    
    cd - > /dev/null
    log_success "Contract deployed successfully"
    echo "Contract ID: $CONTRACT_ID"
    
    # Save contract ID to file
    echo "$CONTRACT_ID" > .contract_id
    log_info "Contract ID saved to .contract_id"
}

# Function to initialize contract
initialize_contract() {
    local contract_id="$1"
    local base_rate="${2:-$DEFAULT_BASE_RATE}"
    local discount_threshold="${3:-$DEFAULT_DISCOUNT_THRESHOLD}"
    local discount_percentage="${4:-$DEFAULT_DISCOUNT_PERCENTAGE}"
    local min_deposit="${5:-$DEFAULT_MIN_DEPOSIT}"
    local heartbeat_interval="${6:-$DEFAULT_HEARTBEAT_INTERVAL}"
    
    log_info "Initializing contract with parameters:"
    log_info "  Base Rate: $base_rate"
    log_info "  Discount Threshold: $discount_threshold seconds"
    log_info "  Discount Percentage: $discount_percentage%"
    log_info "  Min Deposit: $min_deposit"
    log_info "  Heartbeat Interval: $heartbeat_interval seconds"
    
    # Call the init function
    soroban contract invoke \
        --id "$contract_id" \
        --source "$SOROBAN_ADMIN_ADDRESS" \
        --network "$NETWORK" \
        -- \
        init \
        --base_rate "$base_rate" \
        --discount_threshold "$discount_threshold" \
        --discount_percentage "$discount_percentage" \
        --min_deposit "$min_deposit" \
        --heartbeat_interval "$heartbeat_interval"
    
    if [ $? -ne 0 ]; then
        log_error "Contract initialization failed"
        exit 1
    fi
    
    log_success "Contract initialized successfully"
}

# Function to set admin
set_admin() {
    local contract_id="$1"
    local admin_address="$2"
    
    log_info "Setting admin to: $admin_address"
    
    soroban contract invoke \
        --id "$contract_id" \
        --source "$SOROBAN_ADMIN_ADDRESS" \
        --network "$NETWORK" \
        -- \
        set_admin \
        --admin "$admin_address"
    
    if [ $? -ne 0 ]; then
        log_error "Failed to set admin"
        exit 1
    fi
    
    log_success "Admin set successfully"
}

# Function to set teacher role
set_teacher() {
    local contract_id="$1"
    local admin_address="$2"
    local teacher_address="$3"
    local status="${4:-true}"
    
    log_info "Setting teacher role for: $teacher_address (status: $status)"
    
    soroban contract invoke \
        --id "$contract_id" \
        --source "$admin_address" \
        --network "$NETWORK" \
        -- \
        set_teacher \
        --admin "$admin_address" \
        --teacher "$teacher_address" \
        --status "$status"
    
    if [ $? -ne 0 ]; then
        log_error "Failed to set teacher role"
        exit 1
    fi
    
    log_success "Teacher role set successfully"
}

# Function to display usage
usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  deploy                    Build and deploy contract"
    echo "  init CONTRACT_ID         Initialize deployed contract"
    echo "  set-admin CONTRACT_ID    Set admin for contract"
    echo "  set-teacher CONTRACT_ID  Set teacher role for contract"
    echo "  full-deploy              Complete deployment with init and admin setup"
    echo "  help                     Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  SOROBAN_ADMIN_ADDRESS    Admin address for deployment (required)"
    echo ""
    echo "Examples:"
    echo "  $0 full-deploy"
    echo "  $0 deploy"
    echo "  $0 init CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY"
    echo "  $0 set-teacher CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY GD5..."
    echo ""
    echo "For existing contract (already deployed):"
    echo "  CONTRACT_ID=CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY $0 init"
}

# Main script logic
main() {
    # Check if soroban-cli is installed
    check_soroban_cli
    
    # Check network configuration
    check_network
    
    # Check if admin address is set
    if [ -z "$SOROBAN_ADMIN_ADDRESS" ]; then
        log_error "SOROBAN_ADMIN_ADDRESS environment variable is not set"
        log_error "Please set it with: export SOROBAN_ADMIN_ADDRESS=your_address"
        exit 1
    fi
    
    log_info "Using admin address: $SOROBAN_ADMIN_ADDRESS"
    
    case "${1:-help}" in
        "deploy")
            build_contract
            deploy_contract
            ;;
        "init")
            if [ -z "$2" ]; then
                if [ -f ".contract_id" ]; then
                    CONTRACT_ID=$(cat .contract_id)
                else
                    log_error "Contract ID is required. Usage: $0 init CONTRACT_ID"
                    exit 1
                fi
            else
                CONTRACT_ID="$2"
            fi
            initialize_contract "$CONTRACT_ID"
            ;;
        "set-admin")
            if [ -z "$2" ]; then
                log_error "Contract ID is required. Usage: $0 set-admin CONTRACT_ID"
                exit 1
            fi
            set_admin "$2" "$SOROBAN_ADMIN_ADDRESS"
            ;;
        "set-teacher")
            if [ -z "$2" ] || [ -z "$3" ]; then
                log_error "Contract ID and teacher address are required"
                log_error "Usage: $0 set-teacher CONTRACT_ID TEACHER_ADDRESS [true|false]"
                exit 1
            fi
            set_teacher "$2" "$SOROBAN_ADMIN_ADDRESS" "$3" "${4:-true}"
            ;;
        "full-deploy")
            build_contract
            deploy_contract
            CONTRACT_ID=$(cat .contract_id)
            initialize_contract "$CONTRACT_ID"
            set_admin "$CONTRACT_ID" "$SOROBAN_ADMIN_ADDRESS"
            log_success "Full deployment completed!"
            log_info "Contract ID: $CONTRACT_ID"
            log_info "Admin: $SOROBAN_ADMIN_ADDRESS"
            ;;
        "help"|*)
            usage
            ;;
    esac
}

# Run main function with all arguments
main "$@"
