# Teacher Role Management Script for Stream Scholar Contracts (PowerShell)
# This script provides utilities for managing teacher roles on deployed contracts

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("set", "remove", "check", "add-batch", "list", "create-sample", "help")]
    [string]$Command = "help",
    
    [Parameter(Mandatory=$false)]
    [string]$ContractId = "",
    
    [Parameter(Mandatory=$false)]
    [string]$AdminAddress = "",
    
    [Parameter(Mandatory=$false)]
    [string]$TeacherAddress = "",
    
    [Parameter(Mandatory=$false)]
    [bool]$TeacherStatus = $true,
    
    [Parameter(Mandatory=$false)]
    [string]$TeachersFile = ""
)

# Configuration
$NETWORK = "testnet"

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
        return $true
    }
    catch {
        Log-Error "soroban-cli is not installed. Please install it first."
        exit 1
    }
}

# Function to check network configuration
function Test-Network {
    $networks = soroban config network 2>$null
    if ($networks -notmatch $NETWORK) {
        Log-Info "Setting up $NETWORK network configuration"
        soroban config network add $NETWORK `
            --rpc-url "https://soroban-testnet.stellar.org:443" `
            --network-passphrase "Test SDF Network ; September 2015"
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

# Function to check if address is a teacher
function Test-IsTeacher {
    param([string]$ContractId, [string]$TeacherAddress)
    
    Log-Info "Checking teacher status for: $TeacherAddress"
    
    try {
        $result = soroban contract invoke `
            --id $ContractId `
            --network $NETWORK `
            -- `
            is_teacher `
            --teacher $TeacherAddress 2>$null
        
        if ($result -match "true") {
            Log-Success "$TeacherAddress is a teacher"
            return $true
        } else {
            Log-Info "$TeacherAddress is not a teacher"
            return $false
        }
    }
    catch {
        Log-Info "$TeacherAddress is not a teacher or function not available"
        return $false
    }
}

# Function to add multiple teachers from file
function Add-TeachersFromFile {
    param([string]$ContractId, [string]$AdminAddress, [string]$TeachersFile)
    
    if (-not (Test-Path $TeachersFile)) {
        Log-Error "Teachers file not found: $TeachersFile"
        exit 1
    }
    
    Log-Info "Adding teachers from file: $TeachersFile"
    
    try {
        $teachers = Get-Content $TeachersFile | Where-Object { 
            $_ -notmatch '^#' -and $_ -notmatch '^\s*$' 
        }
        
        foreach ($teacher in $teachers) {
            $teacher = $teacher.Trim()
            if ($teacher) {
                Set-TeacherRole $ContractId $AdminAddress $teacher $true
            }
        }
        
        Log-Success "All teachers added successfully"
    }
    catch {
        Log-Error $_.Exception.Message
        exit 1
    }
}

# Function to remove teacher role
function Remove-TeacherRole {
    param([string]$ContractId, [string]$AdminAddress, [string]$TeacherAddress)
    
    Log-Info "Removing teacher role for: $TeacherAddress"
    
    try {
        soroban contract invoke `
            --id $ContractId `
            --source $AdminAddress `
            --network $NETWORK `
            -- `
            set_teacher `
            --admin $AdminAddress `
            --teacher $TeacherAddress `
            --status false
        
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to remove teacher role"
        }
        
        Log-Success "Teacher role removed successfully"
    }
    catch {
        Log-Error $_.Exception.Message
        exit 1
    }
}

# Function to list all teachers (if contract supports it)
function Get-TeacherList {
    param([string]$ContractId)
    
    Log-Info "Attempting to list teachers..."
    Log-Warning "Note: This requires the contract to have a list_teachers function"
    
    try {
        soroban contract invoke `
            --id $ContractId `
            --network $NETWORK `
            -- `
            list_teachers
    }
    catch {
        Log-Warning "list_teachers function not available in contract"
        Log-Info "You can check individual teachers using: .\manage-teachers.ps1 -Command check -ContractId CONTRACT_ID -TeacherAddress TEACHER_ADDRESS"
    }
}

# Function to create sample teachers file
function New-SampleTeachersFile {
    $filename = "teachers.txt"
    
    $content = @"
# Stream Scholar Teachers File
# Add one Stellar address per line
# Lines starting with # are comments

# Example teacher addresses (replace with actual addresses)
# GD5DQ6KZQZJZHQ6Y5X2H5FQD2Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5
# GD7JQ6KZQZJZHQ6Y5X2H5FQD2Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5
# GD8RQ6KZQZJZHQ6Y5X2H5FQD2Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5
"@
    
    $content | Out-File -FilePath $filename -Encoding UTF8
    Log-Success "Sample teachers file created: $filename"
    Log-Info "Edit this file with actual teacher addresses and use:"
    Log-Info ".\manage-teachers.ps1 -Command add-batch -ContractId CONTRACT_ID -AdminAddress ADMIN_ADDRESS -TeachersFile $filename"
}

# Function to display usage
function Show-Usage {
    Write-Host "Usage: .\manage-teachers.ps1 [COMMAND] [OPTIONS]"
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "  set                       Set teacher role"
    Write-Host "  remove                    Remove teacher role"
    Write-Host "  check                     Check if address is a teacher"
    Write-Host "  add-batch                 Add multiple teachers from file"
    Write-Host "  list                      List all teachers (if supported)"
    Write-Host "  create-sample             Create sample teachers file"
    Write-Host "  help                      Show this help message"
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -Command                  Command to execute"
    Write-Host "  -ContractId              Contract ID for operations"
    Write-Host "  -AdminAddress            Admin address for operations"
    Write-Host "  -TeacherAddress          Teacher address for set/remove/check"
    Write-Host "  -TeacherStatus           Teacher status (true/false, default: true)"
    Write-Host "  -TeachersFile            File containing teacher addresses (for add-batch)"
    Write-Host ""
    Write-Host "Environment Variables:"
    Write-Host "  SOROBAN_ADMIN_ADDRESS    Default admin address"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\manage-teachers.ps1 -Command set -ContractId CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY -AdminAddress GD5... -TeacherAddress GD6..."
    Write-Host "  .\manage-teachers.ps1 -Command remove -ContractId CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY -AdminAddress GD5... -TeacherAddress GD6..."
    Write-Host "  .\manage-teachers.ps1 -Command check -ContractId CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY -TeacherAddress GD6..."
    Write-Host "  .\manage-teachers.ps1 -Command add-batch -ContractId CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY -AdminAddress GD5... -TeachersFile teachers.txt"
    Write-Host "  .\manage-teachers.ps1 -Command create-sample"
}

# Main script logic
function Main {
    # Check if soroban-cli is installed
    Test-SorobanCli
    
    # Check network configuration
    Test-Network
    
    switch ($Command) {
        "set" {
            if ([string]::IsNullOrEmpty($ContractId) -or [string]::IsNullOrEmpty($AdminAddress) -or [string]::IsNullOrEmpty($TeacherAddress)) {
                Log-Error "ContractId, AdminAddress, and TeacherAddress are required for set command"
                exit 1
            }
            Set-TeacherRole $ContractId $AdminAddress $TeacherAddress $TeacherStatus
        }
        "remove" {
            if ([string]::IsNullOrEmpty($ContractId) -or [string]::IsNullOrEmpty($AdminAddress) -or [string]::IsNullOrEmpty($TeacherAddress)) {
                Log-Error "ContractId, AdminAddress, and TeacherAddress are required for remove command"
                exit 1
            }
            Remove-TeacherRole $ContractId $AdminAddress $TeacherAddress
        }
        "check" {
            if ([string]::IsNullOrEmpty($ContractId) -or [string]::IsNullOrEmpty($TeacherAddress)) {
                Log-Error "ContractId and TeacherAddress are required for check command"
                exit 1
            }
            Test-IsTeacher $ContractId $TeacherAddress
        }
        "add-batch" {
            if ([string]::IsNullOrEmpty($ContractId) -or [string]::IsNullOrEmpty($AdminAddress) -or [string]::IsNullOrEmpty($TeachersFile)) {
                Log-Error "ContractId, AdminAddress, and TeachersFile are required for add-batch command"
                exit 1
            }
            Add-TeachersFromFile $ContractId $AdminAddress $TeachersFile
        }
        "list" {
            if ([string]::IsNullOrEmpty($ContractId)) {
                Log-Error "ContractId is required for list command"
                exit 1
            }
            Get-TeacherList $ContractId
        }
        "create-sample" {
            New-SampleTeachersFile
        }
        "help" {
            Show-Usage
        }
    }
}

# Run main function
Main
