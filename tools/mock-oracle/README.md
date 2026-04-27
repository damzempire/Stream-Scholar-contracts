# Mock University Oracle CLI

A standalone Rust CLI tool that simulates a University Oracle for local Stream-Scholar development.

## Purpose

Testing the academic compliance layer (`report_student_gpa`, GPA bonus logic) normally requires
a live Oracle network. This tool lets you generate cryptographically valid ed25519-signed payloads
instantly, eliminating that bottleneck during local testnet development.

> **Security:** Keys generated here are for **testnet only**. Never use them on mainnet.

## Installation

```bash
# From the repo root
cargo build -p mock-oracle --release
# Binary at: target/release/mock-oracle
```

## Usage

### 1. Generate a keypair

```bash
mock-oracle keygen
```

Output:
```json
{
  "note": "TESTNET ONLY – never use on mainnet",
  "secret_key": "a1b2c3...",
  "public_key": "d4e5f6..."
}
```

Save the `public_key` — you'll register it as the oracle address in your contract tests.

### 2. Sign a student payload

```bash
mock-oracle sign \
  --secret-key a1b2c3... \
  --student GABC...XYZ \
  --gpa 3.8 \
  --start-date 2024-09-01 \
  --status active
```

Output:
```json
{
  "public_key": "d4e5f6...",
  "payload": "{\"student\":\"GABC...XYZ\",\"gpa_scaled\":38,...}",
  "signature": "7f8a9b...",
  "gpa_for_contract": 38
}
```

Pass `gpa_for_contract` directly to `report_student_gpa` in your Soroban test.

### 3. Verify a payload

```bash
mock-oracle verify \
  --public-key d4e5f6... \
  --payload '{"student":"GABC...XYZ","gpa_scaled":38,...}' \
  --signature 7f8a9b...
```

## GPA Scaling

The contract stores GPA as an integer × 10 to avoid floating-point:

| GPA  | `gpa_scaled` |
|------|-------------|
| 3.5  | 35 (bonus threshold) |
| 3.8  | 38 |
| 4.0  | 40 |

## Running Tests

```bash
cargo test -p mock-oracle
```
