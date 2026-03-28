#!/bin/bash

# Teacher Role Management Script for Stream Scholar Contracts
# This script provides utilities for managing teacher roles on deployed contracts

set -e

# Configuration
NETWORK="testnet"

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
        exit 1
    fi
}

# Function to check network configuration
check_network() {
    if ! soroban config network | grep -q "$NETWORK"; then
        log_info "Setting up $NETWORK network configuration"
        soroban config network add "$NETWORK" \
            --rpc-url "https://soroban-testnet.stellar.org:443" \
            --network-passphrase "Test SDF Network ; September 2015"
    fi
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

# Function to check if address is a teacher
is_teacher() {
    local contract_id="$1"
    local teacher_address="$2"
    
    log_info "Checking teacher status for: $teacher_address"
    
    local result=$(soroban contract invoke \
        --id "$contract_id" \
        --network "$NETWORK" \
        -- \
        is_teacher \
        --teacher "$teacher_address" 2>/dev/null || echo "false")
    
    if echo "$result" | grep -q "true"; then
        log_success "$teacher_address is a teacher"
        return 0
    else
        log_info "$teacher_address is not a teacher"
        return 1
    fi
}

# Function to add multiple teachers from file
add_teachers_from_file() {
    local contract_id="$1"
    local admin_address="$2"
    local teachers_file="$3"
    
    if [ ! -f "$teachers_file" ]; then
        log_error "Teachers file not found: $teachers_file"
        exit 1
    fi
    
    log_info "Adding teachers from file: $teachers_file"
    
    while IFS= read -r teacher_address; do
        # Skip empty lines and comments
        if [[ -n "$teacher_address" && ! "$teacher_address" =~ ^#.* ]]; then
            set_teacher "$contract_id" "$admin_address" "$teacher_address" true
        fi
    done < "$teachers_file"
    
    log_success "All teachers added successfully"
}

# Function to remove teacher role
remove_teacher() {
    local contract_id="$1"
    local admin_address="$2"
    local teacher_address="$3"
    
    log_info "Removing teacher role for: $teacher_address"
    
    soroban contract invoke \
        --id "$contract_id" \
        --source "$admin_address" \
        --network "$NETWORK" \
        -- \
        set_teacher \
        --admin "$admin_address" \
        --teacher "$teacher_address" \
        --status false
    
    if [ $? -ne 0 ]; then
        log_error "Failed to remove teacher role"
        exit 1
    fi
    
    log_success "Teacher role removed successfully"
}

# Function to list all teachers (if contract supports it)
list_teachers() {
    local contract_id="$1"
    
    log_info "Attempting to list teachers..."
    log_warning "Note: This requires the contract to have a list_teachers function"
    
    # Try to call list_teachers if it exists
    soroban contract invoke \
        --id "$contract_id" \
        --network "$NETWORK" \
        -- \
        list_teachers 2>/dev/null || {
        log_warning "list_teachers function not available in contract"
        log_info "You can check individual teachers using: $0 check CONTRACT_ID TEACHER_ADDRESS"
    }
}

# Function to create sample teachers file
create_sample_teachers_file() {
    local filename="teachers.txt"
    
    cat > "$filename" << EOF
# Stream Scholar Teachers File
# Add one Stellar address per line
# Lines starting with # are comments

# Example teacher addresses (replace with actual addresses)
# GD5DQ6KZQZJZHQ6Y5X2H5FQD2Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5
# GD7JQ6KZQZJZHQ6Y5X2H5FQD2Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5
# GD8RQ6KZQZJZHQ6Y5X2H5FQD2Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5
EOF

    log_success "Sample teachers file created: $filename"
    log_info "Edit this file with actual teacher addresses and use:"
    log_info "$0 add-batch CONTRACT_ID ADMIN_ADDRESS $filename"
}

# Function to display usage
usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  set CONTRACT_ID ADMIN_ADDRESS TEACHER_ADDRESS [true|false]"
    echo "                            Set teacher role (default: true)"
    echo "  remove CONTRACT_ID ADMIN_ADDRESS TEACHER_ADDRESS"
    echo "                            Remove teacher role"
    echo "  check CONTRACT_ID TEACHER_ADDRESS"
    echo "                            Check if address is a teacher"
    echo "  add-batch CONTRACT_ID ADMIN_ADDRESS TEACHERS_FILE"
    echo "                            Add multiple teachers from file"
    echo "  list CONTRACT_ID         List all teachers (if supported)"
    echo "  create-sample            Create sample teachers file"
    echo "  help                     Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  SOROBAN_ADMIN_ADDRESS    Default admin address"
    echo ""
    echo "Examples:"
    echo "  $0 set CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY GD5... GD6..."
    echo "  $0 remove CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY GD5... GD6..."
    echo "  $0 check CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY GD6..."
    echo "  $0 add-batch CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY GD5... teachers.txt"
    echo "  $0 create-sample"
}

# Main script logic
main() {
    # Check if soroban-cli is installed
    check_soroban_cli
    
    # Check network configuration
    check_network
    
    case "${1:-help}" in
        "set")
            if [ $# -lt 4 ]; then
                log_error "Insufficient arguments"
                echo "Usage: $0 set CONTRACT_ID ADMIN_ADDRESS TEACHER_ADDRESS [true|false]"
                exit 1
            fi
            set_teacher "$2" "$3" "$4" "${5:-true}"
            ;;
        "remove")
            if [ $# -lt 4 ]; then
                log_error "Insufficient arguments"
                echo "Usage: $0 remove CONTRACT_ID ADMIN_ADDRESS TEACHER_ADDRESS"
                exit 1
            fi
            remove_teacher "$2" "$3" "$4"
            ;;
        "check")
            if [ $# -lt 3 ]; then
                log_error "Insufficient arguments"
                echo "Usage: $0 check CONTRACT_ID TEACHER_ADDRESS"
                exit 1
            fi
            is_teacher "$2" "$3"
            ;;
        "add-batch")
            if [ $# -lt 4 ]; then
                log_error "Insufficient arguments"
                echo "Usage: $0 add-batch CONTRACT_ID ADMIN_ADDRESS TEACHERS_FILE"
                exit 1
            fi
            add_teachers_from_file "$2" "$3" "$4"
            ;;
        "list")
            if [ $# -lt 2 ]; then
                log_error "Insufficient arguments"
                echo "Usage: $0 list CONTRACT_ID"
                exit 1
            fi
            list_teachers "$2"
            ;;
        "create-sample")
            create_sample_teachers_file
            ;;
        "help"|*)
            usage
            ;;
    esac
}

# Run main function with all arguments
main "$@"
