# DISASTER RECOVERY RUNBOOK
## Stream-Scholar Protocol — Security Council Emergency Procedures

> **BREAK GLASS IN CASE OF EMERGENCY**
>
> This document is the authoritative emergency response manual for the Stream-Scholar
> Security Council. It is written for high-stress situations. Read each step carefully
> before executing. Every command listed here is **irreversible or high-impact** unless
> explicitly noted otherwise.

---

## Table of Contents

1. [Severity Levels](#1-severity-levels)
2. [Contact Tree](#2-contact-tree)
3. [Scenario A — Active Exploit / Fund Drain](#3-scenario-a--active-exploit--fund-drain)
4. [Scenario B — Oracle Compromise](#4-scenario-b--oracle-compromise)
5. [Scenario C — Admin Key Compromise](#5-scenario-c--admin-key-compromise)
6. [Scenario D — Contract Bug (No Active Drain)](#6-scenario-d--contract-bug-no-active-drain)
7. [Wasm Upgrade Procedure (Multi-Sig)](#7-wasm-upgrade-procedure-multi-sig)
8. [State Migration V1 → V2](#8-state-migration-v1--v2)
9. [Post-Incident Checklist](#9-post-incident-checklist)
10. [Key Rotation Procedure](#10-key-rotation-procedure)

---

## 1. Severity Levels

| Level | Description | Response Time | Action |
|-------|-------------|---------------|--------|
| **P0** | Active fund drain / exploit in progress | Immediate | Emergency pause + war room |
| **P1** | Vulnerability confirmed, not yet exploited | < 1 hour | Coordinated pause + patch |
| **P2** | Oracle or admin key suspected compromised | < 2 hours | Key rotation + audit |
| **P3** | Bug found, no financial risk | < 24 hours | Scheduled patch |

---

## 2. Contact Tree

Activate in order. Do not skip levels.

```
1. On-call Security Lead  →  Telegram: @security_lead
2. Core Dev Lead          →  Telegram: @core_dev
3. Dean's Council Member 1 → Telegram: @council_m1
4. Dean's Council Member 2 → Telegram: @council_m2
5. Dean's Council Member 3 → Telegram: @council_m3
```

**War Room**: Create a private Telegram group named `INCIDENT-YYYY-MM-DD` immediately.
Add all council members. All decisions must be logged in this group.

---

## 3. Scenario A — Active Exploit / Fund Drain

### Step 1: Confirm the exploit

```bash
# Check contract balance — if draining, act immediately
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <READ_ONLY_KEY> \
  -- get_student_gpa --student <SUSPECTED_ATTACKER>
```

### Step 2: Trigger emergency pause (requires 2-of-3 council signatures)

**Council Member 1 initiates the board pause request:**

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <COUNCIL_MEMBER_1_KEY> \
  -- board_pause_request \
  --council_member <COUNCIL_MEMBER_1_ADDRESS> \
  --student <ATTACKER_OR_AFFECTED_STUDENT> \
  --reason "EMERGENCY_EXPLOIT"
```

**Council Member 2 signs to execute the pause:**

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <COUNCIL_MEMBER_2_KEY> \
  -- board_pause_sign \
  --council_member <COUNCIL_MEMBER_2_ADDRESS> \
  --student <ATTACKER_OR_AFFECTED_STUDENT>
```

> ✅ After 2 signatures, the scholarship is automatically paused and marked disputed.

### Step 3: Pause all scholarships (admin action)

If the exploit affects multiple students, the admin must pause each affected scholarship:

```bash
# Repeat for each affected student address
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <ADMIN_KEY> \
  -- pause_scholarship \
  --admin <ADMIN_ADDRESS> \
  --student <STUDENT_ADDRESS> \
  --status true
```

### Step 4: Veto affected courses globally if course is the attack vector

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <ADMIN_KEY> \
  -- veto_course_globally \
  --admin <ADMIN_ADDRESS> \
  --course_id <COURSE_ID> \
  --status true
```

### Step 5: Notify users

Post to all official channels (Discord, Twitter/X, Telegram):

```
⚠️ MAINTENANCE NOTICE: Stream-Scholar is temporarily paused for emergency maintenance.
Funds are safe. We will provide an update within [X] hours.
```

---

## 4. Scenario B — Oracle Compromise

If the academic oracle key is suspected compromised:

### Step 1: Immediately rotate the oracle address

```bash
# Set oracle to a safe temporary address (a new key you control)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <ADMIN_KEY> \
  -- set_academic_oracle \
  --admin <ADMIN_ADDRESS> \
  --oracle <NEW_SAFE_ORACLE_ADDRESS>
```

### Step 2: Audit all GPA records updated in the last 24 hours

```bash
# Query the Stellar Horizon API for recent contract events
curl "https://horizon.stellar.org/accounts/<CONTRACT_ID>/effects?order=desc&limit=200" \
  | jq '.._embedded.records[] | select(.type == "contract_event")'
```

Look for `GPA_Updated` events with suspicious GPA values (e.g., all students suddenly at 4.4).

### Step 3: Revert suspicious GPA records

For each student with a suspicious GPA, the new oracle must re-report the correct value:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <NEW_ORACLE_KEY> \
  -- report_student_gpa \
  --oracle <NEW_ORACLE_ADDRESS> \
  --student <STUDENT_ADDRESS> \
  --gpa <CORRECT_GPA_SCALED>
```

---

## 5. Scenario C — Admin Key Compromise

> **CRITICAL**: If the admin key is compromised, an attacker can set teachers, pause
> scholarships, and veto courses. Act within minutes.

### Step 1: Assess damage

The admin key controls:
- `set_teacher` — could grant attacker teacher status
- `pause_scholarship` — could freeze all student funds
- `veto_course_globally` — could block all course access
- `set_academic_oracle` — could replace the oracle

### Step 2: Deploy a new contract version with a new admin

Since `set_admin` can only be called once, a compromised admin requires a full contract
upgrade. Follow the [Wasm Upgrade Procedure](#7-wasm-upgrade-procedure-multi-sig).

### Step 3: Communicate to users

Provide a new contract address and migration instructions.

---

## 6. Scenario D — Contract Bug (No Active Drain)

### Step 1: Deactivate affected courses

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <ADMIN_KEY> \
  -- deactivate_course \
  --admin <ADMIN_ADDRESS> \
  --course_id <AFFECTED_COURSE_ID>
```

### Step 2: Prepare and test the patch on testnet

```bash
# Build the patched contract
cd contracts/scholar_contracts
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet for validation
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/scholar_contracts.wasm \
  --source <TESTNET_DEPLOYER_KEY> \
  --network testnet
```

### Step 3: Run full test suite

```bash
cargo test --all
```

All tests must pass before proceeding to mainnet upgrade.

---

## 7. Wasm Upgrade Procedure (Multi-Sig)

Soroban contract upgrades require the admin to call `upgrade` with the new Wasm hash.
This is a **multi-step, irreversible operation**.

### Prerequisites

- [ ] New Wasm binary built and audited
- [ ] Full test suite passing on testnet
- [ ] At least 2 council members available
- [ ] Upgrade announced 24 hours in advance (except P0)

### Step 1: Build and upload the new Wasm

```bash
# Build
cd contracts/scholar_contracts
cargo build --target wasm32-unknown-unknown --release

# Upload to Stellar network (returns a Wasm hash)
stellar contract upload \
  --wasm target/wasm32-unknown-unknown/release/scholar_contracts.wasm \
  --source <ADMIN_KEY> \
  --network mainnet
# Save the returned WASM_HASH
```

### Step 2: Verify the Wasm hash matches the expected binary

```bash
# Compute local hash
sha256sum target/wasm32-unknown-unknown/release/scholar_contracts.wasm

# Compare with the hash returned by the upload command
# They must match exactly before proceeding
```

### Step 3: Execute the upgrade

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <ADMIN_KEY> \
  -- upgrade \
  --new_wasm_hash <WASM_HASH>
```

### Step 4: Verify the upgrade

```bash
# Check the contract's current Wasm hash
stellar contract info \
  --id <CONTRACT_ID> \
  --network mainnet
```

### Step 5: Run smoke tests against mainnet

```bash
# Verify a read-only function still works
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <READ_ONLY_KEY> \
  -- list_courses
```

---

## 8. State Migration V1 → V2

If a new contract version requires migrating trapped user state:

### Step 1: Snapshot all V1 state

```bash
# Export all student scholarship balances
stellar contract invoke \
  --id <V1_CONTRACT_ID> \
  --network mainnet \
  --source <READ_ONLY_KEY> \
  -- list_courses > v1_courses_snapshot.json

# For each student, export their scholarship balance
# (Use a script to iterate known student addresses from event history)
```

### Step 2: Deploy V2 contract

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/scholar_contracts_v2.wasm \
  --source <ADMIN_KEY> \
  --network mainnet
# Save V2_CONTRACT_ID
```

### Step 3: Initialize V2 with same parameters as V1

```bash
stellar contract invoke \
  --id <V2_CONTRACT_ID> \
  --network mainnet \
  --source <ADMIN_KEY> \
  -- init \
  --base_rate <SAME_AS_V1> \
  --discount_threshold <SAME_AS_V1> \
  --discount_percentage <SAME_AS_V1> \
  --min_deposit <SAME_AS_V1> \
  --heartbeat_interval <SAME_AS_V1>
```

### Step 4: Migrate student balances

For each student with a non-zero scholarship balance in V1:

```bash
# Fund the student's scholarship in V2 from the migration treasury
stellar contract invoke \
  --id <V2_CONTRACT_ID> \
  --network mainnet \
  --source <MIGRATION_TREASURY_KEY> \
  -- fund_scholarship \
  --funder <MIGRATION_TREASURY_ADDRESS> \
  --student <STUDENT_ADDRESS> \
  --amount <STUDENT_BALANCE_FROM_V1> \
  --token <TOKEN_ADDRESS>
```

### Step 5: Announce migration complete

Update all frontend configurations to point to V2_CONTRACT_ID.

---

## 9. Post-Incident Checklist

Complete within 48 hours of incident resolution:

- [ ] All affected scholarships unpaused (or confirmed drained and compensated)
- [ ] Oracle address verified and rotated if needed
- [ ] All vetoed courses re-activated (or confirmed permanently removed)
- [ ] Incident timeline documented in `docs/incidents/YYYY-MM-DD.md`
- [ ] Root cause analysis completed
- [ ] Fix deployed and verified on mainnet
- [ ] Public post-mortem published
- [ ] Security Council debrief held
- [ ] Monitoring alerts updated to catch similar issues

---

## 10. Key Rotation Procedure

### Admin Key Rotation

Since `set_admin` can only be called once per contract, admin key rotation requires
a full contract upgrade. Plan accordingly.

**Prevention**: Store the admin key in a hardware wallet (Ledger/Trezor) with a
multi-sig setup. Never store it in plaintext on any server.

### Oracle Key Rotation

The oracle key can be rotated at any time by the admin:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <ADMIN_KEY> \
  -- set_academic_oracle \
  --admin <ADMIN_ADDRESS> \
  --oracle <NEW_ORACLE_ADDRESS>
```

Rotate the oracle key:
- Every 90 days as routine maintenance
- Immediately upon any suspected compromise
- When an oracle team member leaves the organisation

### Council Member Key Rotation

Council member keys are stored in the `DeansCouncil` struct. To rotate:

1. Deploy a new contract version with updated council members, OR
2. Call `init_deans_council` again with the new member set (admin-only, overwrites existing)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network mainnet \
  --source <ADMIN_KEY> \
  -- init_deans_council \
  --admin <ADMIN_ADDRESS> \
  --members '["<NEW_MEMBER_1>","<NEW_MEMBER_2>","<NEW_MEMBER_3>"]' \
  --required_signatures 2
```

---

## Appendix: Useful Horizon Queries

```bash
# Get recent contract events
curl "https://horizon.stellar.org/accounts/<CONTRACT_ID>/effects?order=desc&limit=100"

# Get account balance
curl "https://horizon.stellar.org/accounts/<ADDRESS>"

# Get transaction details
curl "https://horizon.stellar.org/transactions/<TX_HASH>"
```

---

> **REMINDER**: Do not include actual private keys, IP addresses, or sensitive
> credentials in this document. Store secrets in a hardware wallet or a secrets
> manager (e.g., HashiCorp Vault, AWS Secrets Manager).
>
> This document should be reviewed and updated after every incident and every
> quarterly security review.
>
> Last reviewed: 2026-04-27
