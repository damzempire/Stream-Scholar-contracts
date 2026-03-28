# Stream Scholar Contracts Deployment Scripts

This directory contains comprehensive deployment and management scripts for the Stream Scholar smart contracts on Stellar Testnet using soroban-cli.

## Overview

The scripts provide:
- Contract deployment and initialization
- Admin role setup
- Teacher role management
- Batch operations for multiple teachers
- Cross-platform support (Linux/macOS/Windows)

## Prerequisites

1. **Install soroban-cli**
   ```bash
   # Install using cargo
   cargo install soroban-cli
   
   # Or download from GitHub releases
   # https://github.com/stellar/soroban-cli/releases
   ```

2. **Set up Stellar Account**
   - Create a Stellar account on testnet
   - Fund it with test lumens from the [Stellar Testnet Faucet](https://friendbot.stellar.org/)

3. **Set Environment Variable**
   ```bash
   # Linux/macOS
   export SOROBAN_ADMIN_ADDRESS="YOUR_STELLAR_ADDRESS"
   
   # Windows PowerShell
   $env:SOROBAN_ADMIN_ADDRESS="YOUR_STELLAR_ADDRESS"
   ```

## Scripts

### 1. Deployment Scripts

#### `deploy.sh` (Linux/macOS) / `deploy.ps1` (Windows)

Main deployment script that handles:
- Contract building
- Contract deployment
- Contract initialization
- Admin role setup

#### Usage

**Linux/macOS:**
```bash
# Make executable
chmod +x scripts/deploy.sh

# Full deployment (build, deploy, initialize, set admin)
./scripts/deploy.sh full-deploy

# Individual operations
./scripts/deploy.sh deploy
./scripts/deploy.sh init CONTRACT_ID
./scripts/deploy.sh set-admin CONTRACT_ID
./scripts/deploy.sh set-teacher CONTRACT_ID TEACHER_ADDRESS
```

**Windows PowerShell:**
```powershell
# Full deployment
.\scripts\deploy.ps1 -Command full-deploy

# Individual operations
.\scripts\deploy.ps1 -Command deploy
.\scripts\deploy.ps1 -Command init -ContractId CONTRACT_ID
.\scripts\deploy.ps1 -Command set-admin -ContractId CONTRACT_ID
.\scripts\deploy.ps1 -Command set-teacher -ContractId CONTRACT_ID -TeacherAddress TEACHER_ADDRESS
```

### 2. Teacher Management Scripts

#### `manage-teachers.sh` (Linux/macOS) / `manage-teachers.ps1` (Windows)

Specialized script for teacher role management:
- Add/remove individual teachers
- Batch operations from file
- Check teacher status
- List all teachers (if supported)

#### Usage

**Linux/macOS:**
```bash
# Make executable
chmod +x scripts/manage-teachers.sh

# Set individual teacher
./scripts/manage-teachers.sh set CONTRACT_ID ADMIN_ADDRESS TEACHER_ADDRESS

# Remove teacher
./scripts/manage-teachers.sh remove CONTRACT_ID ADMIN_ADDRESS TEACHER_ADDRESS

# Check if address is a teacher
./scripts/manage-teachers.sh check CONTRACT_ID TEACHER_ADDRESS

# Add multiple teachers from file
./scripts/manage-teachers.sh add-batch CONTRACT_ID ADMIN_ADDRESS teachers.txt

# Create sample teachers file
./scripts/manage-teachers.sh create-sample
```

**Windows PowerShell:**
```powershell
# Set individual teacher
.\scripts\manage-teachers.ps1 -Command set -ContractId CONTRACT_ID -AdminAddress ADMIN_ADDRESS -TeacherAddress TEACHER_ADDRESS

# Remove teacher
.\scripts\manage-teachers.ps1 -Command remove -ContractId CONTRACT_ID -AdminAddress ADMIN_ADDRESS -TeacherAddress TEACHER_ADDRESS

# Check if address is a teacher
.\scripts\manage-teachers.ps1 -Command check -ContractId CONTRACT_ID -TeacherAddress TEACHER_ADDRESS

# Add multiple teachers from file
.\scripts\manage-teachers.ps1 -Command add-batch -ContractId CONTRACT_ID -AdminAddress ADMIN_ADDRESS -TeachersFile teachers.txt

# Create sample teachers file
.\scripts\manage-teachers.ps1 -Command create-sample
```

## Configuration

### Default Initialization Parameters

The deployment scripts use these default values:
- **Base Rate**: 100 (tokens per second)
- **Discount Threshold**: 3600 seconds (1 hour)
- **Discount Percentage**: 10%
- **Min Deposit**: 50 tokens
- **Heartbeat Interval**: 300 seconds (5 minutes)

### Network Configuration

Scripts automatically configure the Stellar Testnet network:
- **RPC URL**: `https://soroban-testnet.stellar.org:443`
- **Network Passphrase**: `Test SDF Network ; September 2015`

## Example Workflow

### 1. First-time Deployment

```bash
# Set your admin address
export SOROBAN_ADMIN_ADDRESS="GD5DQ6KZQZJZHQ6Y5X2H5FQD2Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5"

# Deploy and initialize contract
./scripts/deploy.sh full-deploy

# Get the deployed contract ID
CONTRACT_ID=$(cat .contract_id)
echo "Contract deployed: $CONTRACT_ID"
```

### 2. Add Teachers

```bash
# Create teachers file
./scripts/manage-teachers.sh create-sample

# Edit teachers.txt with actual addresses
# nano teachers.txt

# Add teachers in batch
./scripts/manage-teachers.sh add-batch $CONTRACT_ID $SOROBAN_ADMIN_ADDRESS teachers.txt

# Or add individual teacher
./scripts/manage-teachers.sh set $CONTRACT_ID $SOROBAN_ADMIN_ADDRESS GD6JQ6KZQZJZHQ6Y5X2H5FQD2Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5Z5
```

### 3. Working with Existing Contract

The Stream Scholar contract is already deployed on testnet:
- **Contract ID**: `CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY`

```bash
# Initialize existing contract
./scripts/deploy.sh init CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY

# Set admin for existing contract
./scripts/deploy.sh set-admin CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY

# Add teacher to existing contract
./scripts/manage-teachers.sh set CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY $SOROBAN_ADMIN_ADDRESS TEACHER_ADDRESS
```

## File Structure

```
scripts/
├── deploy.sh              # Main deployment script (Linux/macOS)
├── deploy.ps1             # Main deployment script (Windows)
├── manage-teachers.sh     # Teacher management (Linux/macOS)
├── manage-teachers.ps1    # Teacher management (Windows)
└── README.md              # This file
```

## Troubleshooting

### Common Issues

1. **soroban-cli not found**
   ```bash
   # Check installation
   which soroban
   
   # Install if missing
   cargo install soroban-cli
   ```

2. **Environment variable not set**
   ```bash
   # Check if set
   echo $SOROBAN_ADMIN_ADDRESS
   
   # Set it
   export SOROBAN_ADMIN_ADDRESS="YOUR_ADDRESS"
   ```

3. **Insufficient balance**
   - Fund your account from the testnet faucet
   - Check balance: `soroban balance`

4. **Network configuration issues**
   - Scripts auto-configure network
   - Manual setup: `soroban config network add testnet --rpc-url "https://soroban-testnet.stellar.org:443" --network-passphrase "Test SDF Network ; September 2015"`

### Debug Mode

Enable verbose output by setting:
```bash
export RUST_LOG=debug
```

## Contract Functions Reference

### Initialization
- `init(base_rate, discount_threshold, discount_percentage, min_deposit, heartbeat_interval)`

### Admin Functions
- `set_admin(admin_address)`
- `set_teacher(admin, teacher_address, status)`
- `veto_course_globally(admin, course_id, status)`
- `veto_course_access(admin, student, course_id)`

### Teacher Functions
- Teachers can receive scholarship transfers from students
- Teacher status required for `transfer_scholarship_to_teacher`

## Security Considerations

1. **Admin Security**
   - Keep admin address secure
   - Use hardware wallet for production
   - Limit admin permissions

2. **Teacher Verification**
   - Verify teacher addresses before adding
   - Use the check command to confirm status
   - Regularly audit teacher roles

3. **Network Security**
   - Always use testnet for testing
   - Verify network configuration before transactions
   - Double-check contract addresses

## Support

For issues:
1. Check the [Soroban Documentation](https://soroban.stellar.org/)
2. Review the [Stellar Discord](https://discord.gg/stellar)
3. Create an issue in the repository

## Contributing

1. Fork the repository
2. Create a feature branch
3. Test scripts thoroughly
4. Submit a pull request

## License

This project is licensed under the same terms as the Stream Scholar contracts.
