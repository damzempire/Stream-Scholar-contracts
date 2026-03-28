# Instructor Onboarding Guide

Get your course live on Stream Scholar: register your Stellar address, add courses to the platform, and receive payments through the scholarship system.

## Quick Start

```bash
# 1. Get teacher status (admin does this)
./scripts/manage-teachers.sh set CONTRACT_ID ADMIN_ADDRESS YOUR_ADDRESS

# 2. Add your course
soroban contract invoke --id CONTRACT_ID --source YOUR_ADDRESS -- \
  add_course_to_registry --course_id 1 --creator YOUR_ADDRESS

# 3. Set course duration (anyone can call this)
soroban contract invoke --id CONTRACT_ID -- \
  set_course_duration --course_id 1 --duration 7200
```

## Prerequisites

| Requirement | Description |
|------------|-------------|
| Stellar Account | Testnet wallet (e.g., `GD5JJ3STX5U5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5`) |
| Test XLM | Get from [Stellar Testnet Faucet](https://friendbot.stellar.org/) |
| soroban-cli | Install: `cargo install soroban-cli` |

Verify installation:

```bash
which soroban
```

## Step 1: Build the Contract

If you're deploying locally (or the admin is):

```bash
cd contracts/scholar_contracts
stellar contract build
```

The WASM file will be at: `contracts/scholar_contracts/target/wasm32v1-none/release/scholar_contracts.wasm`

## Step 2: Get Teacher Status

Teachers cannot self-register. Contact the platform admin with your Stellar address.

The admin uses the script:

```bash
./scripts/manage-teachers.sh set CONTRACT_ID ADMIN_ADDRESS TEACHER_ADDRESS
```

Or directly:

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source ADMIN_ADDRESS \
  -- \
  set_teacher \
  --admin ADMIN_ADDRESS \
  --teacher TEACHER_ADDRESS \
  --status true
```

Parameters:
- `CONTRACT_ID` - Deployed contract ID
- `ADMIN_ADDRESS` - Admin's Stellar address
- `TEACHER_ADDRESS` - Your Stellar address

## Step 3: Environment Setup

**Linux/macOS:**
```bash
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org:443"
export SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export SOROBAN_ADMIN_ADDRESS="ADMIN_ADDRESS"
export TEACHER_ADDRESS="YOUR_STELLAR_ADDRESS"
export CONTRACT_ID="CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY"
```

**Windows PowerShell:**
```powershell
$env:SOROBAN_RPC_URL="https://soroban-testnet.stellar.org:443"
$env:SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
$env:SOROBAN_ADMIN_ADDRESS="ADMIN_ADDRESS"
$env:TEACHER_ADDRESS="YOUR_STELLAR_ADDRESS"
$env:CONTRACT_ID="CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY"
```

## Step 4: Add Course to Registry

Make your course discoverable:

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source $TEACHER_ADDRESS \
  -- \
  add_course_to_registry \
  --course_id COURSE_ID \
  --creator $TEACHER_ADDRESS
```

Where `COURSE_ID` is a unique number (e.g., `1`, `2`, `3`).

Requirements:
- You must have teacher status (set by admin)
- Course ID must not already exist

## Step 5: Set Course Duration

Set total length for completion tracking and SBT minting:

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  -- \
  set_course_duration \
  --course_id COURSE_ID \
  --duration DURATION_SECONDS
```

Example: `7200` seconds = 2 hours.

**Note:** This function does not require authorization. Anyone can set a course duration.

## Step 6: Create Course Metadata

Store metadata on IPFS for frontend display:

```json
{
  "courseId": "intro-blockchain-101",
  "title": "Introduction to Blockchain",
  "description": "Learn blockchain fundamentals.",
  "instructor": {
    "name": "Your Name",
    "address": "YOUR_STELLAR_ADDRESS"
  },
  "duration": {
    "totalMinutes": 120,
    "estimatedHours": 2
  },
  "thumbnail": {
    "ipfsCid": "QmYourCID",
    "mimeType": "image/jpeg"
  },
  "createdAt": "2026-03-26T10:00:00Z"
}
```

## Pricing

**Important:** Teachers do not set per-minute prices. Pricing is global, set by the platform admin at contract initialization.

Default values (from deploy.sh):
- **Base Rate**: `100` tokens per second
- **Discount Threshold**: `3600` seconds (1 hour)
- **Discount Percentage**: `10%`
- **Min Deposit**: `50` tokens
- **Heartbeat Interval**: `300` seconds (5 minutes)

The dynamic rate calculation (from `calculate_dynamic_rate`):
- Before threshold: full `base_rate`
- After threshold: `base_rate - (base_rate * discount_percentage / 100)`

Example: With base rate 100 and 10% discount after 1 hour, students pay:
- First hour: 100 tokens/second
- After that: 90 tokens/second

Students pay per second watched. Teachers receive payments via the scholarship system.

## Receiving Payments

Students pay teachers through a scholarship balance system:

1. **Funder** adds to student's scholarship balance via `fund_scholarship`
2. **Student** withdraws or transfers to teachers via `transfer_scholarship_to_teacher`

**Student transfers to you:**

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source STUDENT_ADDRESS \
  -- \
  transfer_scholarship_to_teacher \
  --student STUDENT_ADDRESS \
  --teacher $TEACHER_ADDRESS \
  --amount AMOUNT
```

Requirements:
- Your teacher status must be `true` (checked via `IsTeacher` storage)
- Student must have sufficient scholarship balance

## Monitoring Engagement

**Check if student completed your course (SBT minted):**

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  -- \
  is_sbt_minted \
  --student STUDENT_ADDRESS \
  --course_id COURSE_ID
```

Returns `true` if student watched for `course_duration` seconds.

**Get course info:**

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  -- \
  get_course_info \
  --course_id COURSE_ID
```

Returns: `{course_id, created_at, is_active, creator}`

**List all courses:**

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  -- \
  list_courses
```

**Paginated listing (max 100):**

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  -- \
  list_courses_paginated \
  --offset 0 \
  --limit 10
```

**Track student watch time:**

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  -- \
  get_watch_time \
  --student STUDENT_ADDRESS \
  --course_id COURSE_ID
```

## Contract Reference

### Initialization (admin)

```rust
init(base_rate, discount_threshold, discount_percentage, min_deposit, heartbeat_interval)
```

### Admin Functions

| Function | Auth | Description |
|----------|------|-------------|
| `set_admin` | requires auth | Set admin address (once only) |
| `set_teacher` | admin | Add/remove teacher status |
| `veto_course_globally` | admin | Globally block a course |
| `veto_course_access` | admin | Block student's course access |
| `deactivate_course` | admin | Mark course inactive |
| `cleanup_inactive_courses` | admin | Remove inactive courses |

### Teacher Functions

| Function | Auth | Description |
|----------|------|-------------|
| `add_course_to_registry` | creator | Register new course |
| `set_course_duration` | **none** | Set course length (public) |

### Student Functions

| Function | Auth | Description |
|----------|------|-------------|
| `buy_access` | student | Buy watch time |
| `heartbeat` | student | Send viewing heartbeat |
| `buy_subscription` | student | Buy subscription |
| `fund_scholarship` | funder | Add to student balance |
| `transfer_scholarship_to_teacher` | student | Pay teacher |
| `withdraw_scholarship` | student | Withdraw balance |
| `pro_rated_refund` | student | Refund within 5 min |

### Query Functions

| Function | Description |
|----------|-------------|
| `has_access` | Check if student has access |
| `get_watch_time` | Get student's watch seconds |
| `is_sbt_minted` | Check if completion SBT minted |
| `list_courses` | Get all course IDs |
| `list_courses_paginated` | Paginated course listing |
| `get_course_info` | Get course metadata |
| `calculate_remaining_airtime` | Student remaining time |
| `get_bonus_minutes` | Get referral bonus minutes |

### Storage Keys

- `Admin` - Admin address
- `IsTeacher(address)` - Teacher approval status
- `CourseInfo(course_id)` - Course metadata
- `CourseRegistry` - All courses list
- `CourseRegistrySize` - Total course count
- `Scholarship(address)` - Student balance + token
- `Access(student, course_id)` - Student access record
- `SbtMinted(student, course_id)` - Completion flag

## Troubleshooting

### Error: Contract Error #6

Invalid operation or authorization failure.

Causes:
- Course ID already exists
- Teacher status not set
- Wrong transaction source

### Teacher Status Not Active

1. Confirm admin added your address with `set_teacher`
2. Check XLM balance for fees
3. Verify correct contract ID

### Course Not Showing

1. Verify `add_course_to_registry` succeeded
2. Check course ID is correct
3. Confirm on testnet

### Student Cannot Transfer

1. Verify teacher status is `true`
2. Student needs scholarship balance
3. Check contract ID

### Error on set_course_duration

This function is public (no auth required). If it fails:
- Check contract ID is correct
- Verify network connectivity

## Deployment

For admins deploying the contract:

```bash
# Full deployment
./scripts/deploy.sh full-deploy

# Or step by step
./scripts/deploy.sh deploy
./scripts/deploy.sh init CONTRACT_ID
./scripts/deploy.sh set-admin CONTRACT_ID

# Set teacher
./scripts/manage-teachers.sh set CONTRACT_ID ADMIN_ADDRESS TEACHER_ADDRESS
```

## Support

- [Stellar Developers Documentation](https://developers.stellar.org/docs)
- [Stellar Discord](https://discord.gg/stellar)
- Repository Issues
- Contact platform admin
