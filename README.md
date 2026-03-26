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
