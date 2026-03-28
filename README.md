# Soroban Project

## Project Structure

This repository uses the recommended structure for a Soroban project:

```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.

## Deployed Contract
- **Network:** Stellar Testnet
- **Contract ID:** CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY


## Session Security

This contract prevents multi-device streaming by enforcing a strict single-session lock per user account. 

It natively extends the existing `heartbeat` function to validate a unique 32-byte `session_hash` (passed via the previously unused `_signature` parameter), ensuring complete backward compatibility with zero breaking changes to the API.

**How it works:**
* **Accepted Session:** When a heartbeat is received, it checks the stored session hash. If the hash matches the active session, or if the previous session has safely timed out (exceeding the `heartbeat_interval`), the stream is securely permitted.
* **Rejected Session:** If the incoming hash does not match the stored hash *and* the previous session is currently active, the contract explicitly rejects the heartbeat. This immediately halts unauthorized parallel streams or duplicate logins.

## Local Test Network

To set up a local test network with Docker that pre-loads 5 dummy courses and 100 test USDC:

1. Ensure Docker and Docker Compose are installed.

2. Run the following command in the project root:
   ```bash
   docker compose up
   ```

3. The setup script will:
   - Start a local Soroban network
   - Generate and fund test accounts (admin, teacher, student)
   - Deploy a USDC token contract and mint 100 USDC to the student account
   - Deploy the scholar contract and initialize it
   - Add 5 dummy courses to the registry

4. The network will remain running. The contract IDs and account details will be displayed in the output.

### Testing the Setup

To verify the setup is successful:

1. In a new terminal, exec into the running container:
   ```bash
   docker compose exec soroban-local bash
   ```

2. Check the list of courses:
   ```bash
   soroban contract invoke --id <SCHOLAR_CONTRACT_ID> --network standalone -- list_courses
   ```
   Expected output: `[1,2,3,4,5]`

3. Check the USDC balance of the student:
   ```bash
   soroban contract invoke --id <USDC_TOKEN_ID> --network standalone -- balance --id <STUDENT_ADDRESS>
   ```
   Expected output: `1000000000` (100 USDC with 7 decimals)

4. Verify course info for course 1:
   ```bash
   soroban contract invoke --id <SCHOLAR_CONTRACT_ID> --network standalone -- get_course_info --course_id 1
   ```
   Should return course info with is_active: true

5. Test buying access (this should succeed if setup is correct):
   ```bash
   soroban contract invoke --id <SCHOLAR_CONTRACT_ID> --source <STUDENT_SECRET> --network standalone -- buy_access --student <STUDENT_ADDRESS> --course_id 1 --amount 100 --token <USDC_TOKEN_ID>
   ```

If all tests pass, the local test network setup is successful.

## GPA-Weighted Flow Rate Bonus Logic

This feature implements a "Meritocratic Drip" system that incentivizes academic excellence by adjusting token flow rates based on student GPA.

### How It Works

**GPA Bonus Calculation:**
- **Threshold:** 3.5 GPA (stored as 35 to avoid floating-point arithmetic)
- **Bonus Rate:** 2% increase for every 0.1 GPA point above 3.5
- **Maximum GPA:** 4.4 (18% maximum bonus)
- **Oracle Verification:** Only oracle-verified GPAs are considered

**Example Calculations:**
- 3.5 GPA = 0% bonus (base rate)
- 3.6 GPA = 2% bonus
- 3.7 GPA = 4% bonus
- 4.0 GPA = 10% bonus
- 4.4 GPA = 18% bonus

### Key Features

**1. Dynamic Rate Adjustment**
The `calculate_dynamic_rate` function now includes GPA bonuses alongside existing watch time discounts:
```rust
// Apply GPA bonus (increase rate based on academic performance)
let gpa_bonus_percentage = Self::calculate_gpa_bonus(env.clone(), student.clone());
if gpa_bonus_percentage > 0 {
    let bonus = (rate * gpa_bonus_percentage as i128) / 100;
    rate += bonus; // Increase rate for high-performing students
}
```

**2. Oracle-Based GPA Reporting**
Only the designated academic oracle can report student GPAs:
```rust
pub fn report_student_gpa(env: Env, oracle: Address, student: Address, gpa: u64)
```

**3. On-the-Fly Drip Recalculation**
When GPA updates occur, the system automatically recalculates flow rates without resetting stream start dates:
```rust
pub fn recalculate_drip_rate_on_gpa_change(env: Env, student: Address)
```

### Implementation Details

**Data Structures:**
```rust
pub struct StudentGPA {
    pub student: Address,
    pub gpa: u64, // Stored as integer (e.g., 3.7 = 37)
    pub last_updated: u64,
    pub oracle_verified: bool,
}
```

**Constants:**
```rust
const GPA_BONUS_THRESHOLD: u64 = 35; // 3.5 GPA threshold
const GPA_BONUS_PERCENTAGE_PER_POINT: u64 = 20; // 2% per 0.1 GPA (20% per 1.0 GPA)
```

### Usage Example

1. **Set up Academic Oracle:**
```bash
soroban contract invoke --id <CONTRACT_ID> --source <ADMIN> --network standalone -- set_academic_oracle --admin <ADMIN> --oracle <ORACLE_ADDRESS>
```

2. **Report Student GPA:**
```bash
soroban contract invoke --id <CONTRACT_ID> --source <ORACLE> --network standalone -- report_student_gpa --oracle <ORACLE> --student <STUDENT_ADDRESS> --gpa 38
```

3. **Check GPA Bonus:**
```bash
soroban contract invoke --id <CONTRACT_ID> --network standalone -- get_student_gpa_bonus --student <STUDENT_ADDRESS>
```

4. **Get Student GPA Data:**
```bash
soroban contract invoke --id <CONTRACT_ID> --network standalone -- get_student_gpa --student <STUDENT_ADDRESS>
```

### Events

The system emits events for GPA updates and drip recalculations:
- `GPA_Updated`: Emitted when a student's GPA is updated
- `Drip_Rate_Recalculated`: Emitted when flow rates are recalculated due to GPA changes

### Security Considerations

1. **Oracle Authorization:** Only the designated academic oracle can report GPAs
2. **GPA Validation:** GPAs are validated to be within 0.0-4.4 range
3. **Verification Flag:** Only oracle-verified GPAs affect bonus calculations
4. **Backward Compatibility:** Existing functionality remains unchanged

### Testing

Comprehensive tests are included in `test.rs`:
- `test_gpa_bonus_calculation`: Verifies bonus percentage calculations
- `test_gpa_weighted_flow_rate`: Tests integration with access purchases
- `test_gpa_data_storage`: Validates GPA data storage and retrieval
- `test_drip_recalculation_on_gpa_change`: Tests automatic recalculation
- `test_gpa_validation`: Ensures GPA validation works correctly

### Benefits

1. **Academic Incentive:** Students are financially motivated to maintain high GPAs
2. **Meritocratic System:** Higher-performing students receive better rates
3. **Dynamic Adjustment:** Rates update automatically as academic performance changes
4. **Preserved Contracts:** Stream start dates and balances remain intact during recalculation
5. **Gamification:** Creates a competitive environment for academic excellence
