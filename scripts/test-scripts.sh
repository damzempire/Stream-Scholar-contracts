#!/bin/bash

# Test script to validate deployment scripts
# This script performs basic validation without actually deploying

set -e

echo "=== Stream Scholar Deployment Scripts Test ==="

# Test if scripts exist
echo "Checking script files..."
SCRIPTS_DIR="scripts"
REQUIRED_SCRIPTS=(
    "deploy.sh"
    "deploy.ps1" 
    "manage-teachers.sh"
    "manage-teachers.ps1"
    "README.md"
    "config.example.env"
)

for script in "${REQUIRED_SCRIPTS[@]}"; do
    if [ -f "$SCRIPTS_DIR/$script" ]; then
        echo "✓ $script exists"
    else
        echo "✗ $script missing"
        exit 1
    fi
done

# Test script syntax (basic)
echo ""
echo "Checking script syntax..."

if command -v bash &> /dev/null; then
    if bash -n "$SCRIPTS_DIR/deploy.sh"; then
        echo "✓ deploy.sh syntax OK"
    else
        echo "✗ deploy.sh syntax error"
    fi
    
    if bash -n "$SCRIPTS_DIR/manage-teachers.sh"; then
        echo "✓ manage-teachers.sh syntax OK"
    else
        echo "✗ manage-teachers.sh syntax error"
    fi
else
    echo "! bash not available for syntax checking"
fi

# Test PowerShell scripts (if available)
if command -v pwsh &> /dev/null; then
    if pwsh -Command "& { . '$SCRIPTS_DIR/deploy.ps1' -Command help }" > /dev/null 2>&1; then
        echo "✓ deploy.ps1 syntax OK"
    else
        echo "✗ deploy.ps1 syntax error"
    fi
    
    if pwsh -Command "& { . '$SCRIPTS_DIR/manage-teachers.ps1' -Command help }" > /dev/null 2>&1; then
        echo "✓ manage-teachers.ps1 syntax OK"
    else
        echo "✗ manage-teachers.ps1 syntax error"
    fi
else
    echo "! PowerShell not available for syntax checking"
fi

# Check if soroban-cli is available (optional)
echo ""
echo "Checking dependencies..."
if command -v soroban &> /dev/null; then
    echo "✓ soroban-cli found"
    SOROBAN_VERSION=$(soroban --version 2>/dev/null || echo "unknown")
    echo "  Version: $SOROBAN_VERSION"
else
    echo "! soroban-cli not found (install required for deployment)"
fi

# Check if stellar is available (optional)
if command -v stellar &> /dev/null; then
    echo "✓ stellar CLI found"
else
    echo "! stellar CLI not found (required for building)"
fi

# Check contract structure
echo ""
echo "Checking contract structure..."
if [ -d "contracts/scholar_contracts" ]; then
    echo "✓ Contract directory exists"
    
    if [ -f "contracts/scholar_contracts/Cargo.toml" ]; then
        echo "✓ Cargo.toml exists"
    else
        echo "✗ Cargo.toml missing"
    fi
    
    if [ -f "contracts/scholar_contracts/src/lib.rs" ]; then
        echo "✓ lib.rs exists"
    else
        echo "✗ lib.rs missing"
    fi
else
    echo "✗ Contract directory missing"
fi

echo ""
echo "=== Test Summary ==="
echo "✓ Scripts created successfully"
echo "✓ Documentation provided"
echo "✓ Configuration examples included"
echo ""
echo "Next steps:"
echo "1. Install soroban-cli: cargo install soroban-cli"
echo "2. Set SOROBAN_ADMIN_ADDRESS environment variable"
echo "3. Run: ./scripts/deploy.sh full-deploy (Linux/macOS) or .\\scripts\\deploy.ps1 -Command full-deploy (Windows)"
echo ""
echo "For existing contract:"
echo "Contract ID: CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY"
echo "Use: ./scripts/deploy.sh init CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY"
