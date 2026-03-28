# Stream Scholar Contracts Deployment Script (PowerShell)
# This script handles contract deployment, initialization, and teacher role setup for Stellar Testnet

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("deploy", "init", "set-admin", "set-teacher", "full-deploy", "help")]
    [string]$Command = "help",
    
    [Parameter(Mandatory=$false)]
    [string]$ContractId = "",
    
    [Parameter(Mandatory=$false)]
    [string]$TeacherAddress = "",
    
    [Parameter(Mandatory=$false)]
    [bool]$TeacherStatus = $true
)

# Configuration
$NETWORK = "testnet"
$CONTRACT_NAME = "scholar_contracts"
$CONTRACT_DIR = ".\contracts\scholar_contracts"
$WASM_FILE = "target\wasm32v1-none\release\scholar_contracts.wasm"

# Default values for initialization
$DEFAULT_BASE_RATE = 100
$DEFAULT_DISCOUNT_THRESHOLD = 3600
$DEFAULT_DISCOUNT_PERCENTAGE = 10
$DEFAULT_MIN_DEPOSIT = 50
$DEFAULT_HEARTBEAT_INTERVAL = 300

# Colors for output
$Colors = @{
    Red = "Red"
    Green = "Green"
    Yellow = "Yellow"
    Blue = "Blue"
    White = "White"
}

# Logging functions
function Log-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $Colors.Blue
}

function Log-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Colors.Green
}

function Log-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Colors.Yellow
}

function Log-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Colors.Red
}

# Function to check if soroban-cli is installed
function Test-SorobanCli {
    try {
        $null = Get-Command soroban -ErrorAction Stop
        Log-Success "soroban-cli found"
        return $true
    }
    catch {
        Log-Error "soroban-cli is not installed. Please install it first."
        Write-Host "Visit: https://github.com/stellar/soroban-cli"
        exit 1
    }
}

# Function to check if we're on the right network
function Test-Network {
    Log-Info "Checking network configuration for $NETWORK"
    
    # Check if network is configured
    $networks = soroban config network 2>$null
    if ($networks -notmatch $NETWORK) {
        Log-Info "Setting up $NETWORK network configuration"
        soroban config network add $NETWORK `
            --rpc-url "https://soroban-testnet.stellar.org:443" `
            --network-passphrase "Test SDF Network ; September 2015"
    }
    
    Log-Success "Network configuration verified"
}

# Function to build the contract
function Build-Contract {
    Log-Info "Building contract..."
    
    if (-not (Test-Path $CONTRACT_DIR)) {
        Log-Error "Contract directory $CONTRACT_DIR not found"
        exit 1
    }
    
    Push-Location $CONTRACT_DIR
    
    try {
        # Build the contract
        $buildResult = stellar contract build
        if ($LASTEXITCODE -ne 0) {
            throw "Contract build failed"
        }
        
        # Check if WASM file was created
        if (-not (Test-Path $WASM_FILE)) {
            Log-Error "WASM file not found at $WASM_FILE"
            exit 1
        }
        
        Log-Success "Contract built successfully"
    }
    catch {
        Log-Error $_.Exception.Message
        exit 1
    }
    finally {
        Pop-Location
    }
}

# Function to deploy contract
function Deploy-Contract {
    Log-Info "Deploying contract to $NETWORK..."
    
    Push-Location $CONTRACT_DIR
    
    try {
        # Deploy the contract
        $deployResult = soroban contract deploy `
            --wasm $WASM_FILE `
            --source $env:SOROBAN_ADMIN_ADDRESS `
            --network $NETWORK
        
        if ($LASTEXITCODE -ne 0) {
            throw "Contract deployment failed"
        }
        
        $CONTRACT_ID = $deployResult.Trim()
        Log-Success "Contract deployed successfully"
        Write-Host "Contract ID: $CONTRACT_ID"
        
        # Save contract ID to file
        $CONTRACT_ID | Out-File -FilePath ".contract_id" -Encoding UTF8
        Log-Info "Contract ID saved to .contract_id"
        
        return $CONTRACT_ID
    }
    catch {
        Log-Error $_.Exception.Message
        exit 1
    }
    finally {
        Pop-Location
    }
}

# Function to initialize contract
function Initialize-Contract {
    param([string]$ContractId)
    
    $baseRate = $DEFAULT_BASE_RATE
    $discountThreshold = $DEFAULT_DISCOUNT_THRESHOLD
    $discountPercentage = $DEFAULT_DISCOUNT_PERCENTAGE
    $minDeposit = $DEFAULT_MIN_DEPOSIT
    $heartbeatInterval = $DEFAULT_HEARTBEAT_INTERVAL
    
    Log-Info "Initializing contract with parameters:"
    Log-Info "  Base Rate: $baseRate"
    Log-Info "  Discount Threshold: $discountThreshold seconds"
    Log-Info "  Discount Percentage: $discountPercentage%"
    Log-Info "  Min Deposit: $minDeposit"
    Log-Info "  Heartbeat Interval: $heartbeatInterval seconds"
    
    try {
        # Call the init function
        soroban contract invoke `
            --id $ContractId `
            --source $env:SOROBAN_ADMIN_ADDRESS `
            --network $NETWORK `
            -- `
            init `
            --base_rate $baseRate `
            --discount_threshold $discountThreshold `
            --discount_percentage $discountPercentage `
            --min_deposit $minDeposit `
            --heartbeat_interval $heartbeatInterval
        
        if ($LASTEXITCODE -ne 0) {
            throw "Contract initialization failed"
        }
        
        Log-Success "Contract initialized successfully"
    }
    catch {
        Log-Error $_.Exception.Message
        exit 1
    }
}

# Function to set admin
function Set-AdminRole {
    param([string]$ContractId, [string]$AdminAddress)
    
    Log-Info "Setting admin to: $AdminAddress"
    
    try {
        soroban contract invoke `
            --id $ContractId `
            --source $env:SOROBAN_ADMIN_ADDRESS `
            --network $NETWORK `
            -- `
            set_admin `
            --admin $AdminAddress
        
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to set admin"
        }
        
        Log-Success "Admin set successfully"
    }
    catch {
        Log-Error $_.Exception.Message
        exit 1
    }
}

# Function to set teacher role
function Set-TeacherRole {
    param([string]$ContractId, [string]$AdminAddress, [string]$TeacherAddress, [bool]$Status)
    
    $statusStr = if ($Status) { "true" } else { "false" }
    Log-Info "Setting teacher role for: $TeacherAddress (status: $statusStr)"
    
    try {
        soroban contract invoke `
            --id $ContractId `
            --source $AdminAddress `
            --network $NETWORK `
            -- `
            set_teacher `
            --admin $AdminAddress `
            --teacher $TeacherAddress `
            --status $statusStr
        
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to set teacher role"
        }
        
        Log-Success "Teacher role set successfully"
    }
    catch {
        Log-Error $_.Exception.Message
        exit 1
    }
}

# Function to display usage
function Show-Usage {
    Write-Host "Usage: .\deploy.ps1 [COMMAND] [OPTIONS]"
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "  deploy                    Build and deploy contract"
    Write-Host "  init CONTRACT_ID         Initialize deployed contract"
    Write-Host "  set-admin CONTRACT_ID    Set admin for contract"
    Write-Host "  set-teacher CONTRACT_ID  Set teacher role for contract"
    Write-Host "  full-deploy              Complete deployment with init and admin setup"
    Write-Host "  help                     Show this help message"
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -ContractId             Contract ID for operations"
    Write-Host "  -TeacherAddress         Teacher address for set-teacher command"
    Write-Host "  -TeacherStatus          Teacher status (true/false, default: true)"
    Write-Host ""
    Write-Host "Environment Variables:"
    Write-Host "  SOROBAN_ADMIN_ADDRESS    Admin address for deployment (required)"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\deploy.ps1 -Command full-deploy"
    Write-Host "  .\deploy.ps1 -Command deploy"
    Write-Host "  .\deploy.ps1 -Command init -ContractId CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY"
    Write-Host "  .\deploy.ps1 -Command set-teacher -ContractId CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY -TeacherAddress GD5..."
    Write-Host ""
    Write-Host "For existing contract (already deployed):"
    Write-Host "  `$env:CONTRACT_ID='CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY'; .\deploy.ps1 -Command init"
}

# Main script logic
function Main {
    # Check if soroban-cli is installed
    Test-SorobanCli
    
    # Check network configuration
    Test-Network
    
    # Check if admin address is set
    if (-not $env:SOROBAN_ADMIN_ADDRESS) {
        Log-Error "SOROBAN_ADMIN_ADDRESS environment variable is not set"
        Log-Error "Please set it with: `$env:SOROBAN_ADMIN_ADDRESS='your_address'"
        exit 1
    }
    
    Log-Info "Using admin address: $env:SOROBAN_ADMIN_ADDRESS"
    
    switch ($Command) {
        "deploy" {
            Build-Contract
            Deploy-Contract
        }
        "init" {
            $contractIdToUse = $ContractId
            if ([string]::IsNullOrEmpty($contractIdToUse)) {
                if (Test-Path ".contract_id") {
                    $contractIdToUse = Get-Content ".contract_id" -Raw
                } else {
                    Log-Error "Contract ID is required. Use -ContractId parameter or ensure .contract_id file exists"
                    exit 1
                }
            }
            Initialize-Contract $contractIdToUse.Trim()
        }
        "set-admin" {
            if ([string]::IsNullOrEmpty($ContractId)) {
                Log-Error "Contract ID is required. Use -ContractId parameter"
                exit 1
            }
            Set-AdminRole $ContractId $env:SOROBAN_ADMIN_ADDRESS
        }
        "set-teacher" {
            if ([string]::IsNullOrEmpty($ContractId) -or [string]::IsNullOrEmpty($TeacherAddress)) {
                Log-Error "Contract ID and teacher address are required"
                Log-Error "Use -ContractId and -TeacherAddress parameters"
                exit 1
            }
            Set-TeacherRole $ContractId $env:SOROBAN_ADMIN_ADDRESS $TeacherAddress $TeacherStatus
        }
        "full-deploy" {
            Build-Contract
            $deployedContractId = Deploy-Contract
            Initialize-Contract $deployedContractId
            Set-AdminRole $deployedContractId $env:SOROBAN_ADMIN_ADDRESS
            Log-Success "Full deployment completed!"
            Log-Info "Contract ID: $deployedContractId"
            Log-Info "Admin: $env:SOROBAN_ADMIN_ADDRESS"
        }
        "help" {
            Show-Usage
        }
    }
}

# Run main function
Main
