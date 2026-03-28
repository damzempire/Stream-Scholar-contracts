# Research Grant Milestone Escrow Feature

## Overview

The Research Grant Milestone Escrow feature implements issue #83, providing financial flexibility for large research grants while maintaining the stability of students' daily income through living stipends. This feature allows students to purchase equipment and other research necessities through a milestone-based approval system.

## Key Features

### 1. **Research Grant Creation**
- Grantors can create research grants for students
- Funds are transferred to the contract treasury for secure escrow
- Each grant is uniquely identified and tracked

### 2. **Milestone Claim Submission**
- Students submit milestone claims with invoice hashes
- Supports equipment purchases and other research expenses
- Claims include description, amount, and invoice verification

### 3. **Grantor Approval System**
- Only the original grantor can approve milestone claims
- Ensures proper verification before fund release
- Prevents unauthorized access to research funds

### 4. **Lump Sum Distribution**
- Approved claims release lump sums directly to students
- Treasury management ensures secure fund transfers
- Living stipend drips continue uninterrupted

## Core Functions

### `create_research_grant(grantor, student, total_amount, token) -> u64`
Creates a new research grant and returns the grant ID.

**Parameters:**
- `grantor`: Address of the funding entity
- `student`: Address of the student researcher
- `total_amount`: Total grant amount in tokens
- `token`: Token address for the grant

**Events:**
- `Research_Grant_Created`: Emitted when grant is created

### `submit_milestone_claim(student, milestone_id, amount, description, invoice_hash)`
Students submit milestone claims for equipment purchases.

**Parameters:**
- `student`: Student address
- `milestone_id`: Unique milestone identifier
- `amount`: Claim amount
- `description`: Milestone description
- `invoice_hash`: Hash of the invoice document

**Events:**
- `Milestone_Claim_Submitted`: Emitted when claim is submitted

### `approve_milestone_claim(grantor, milestone_id)`
Grantor approves a submitted milestone claim.

**Parameters:**
- `grantor`: Original grantor address
- `milestone_id`: Milestone to approve

**Events:**
- `Milestone_Claim_Approved`: Emitted when claim is approved

### `claim_milestone_lump_sum(student, milestone_id)`
Students claim approved milestone funds.

**Parameters:**
- `student`: Student address
- `milestone_id`: Approved milestone to claim

**Events:**
- `Milestone_Lump_Sum_Claimed`: Emitted when funds are claimed

## Data Structures

### `ResearchGrant`
```rust
pub struct ResearchGrant {
    pub student: Address,
    pub total_amount: i128,
    pub token: Address,
    pub granted_at: u64,
    pub is_active: bool,
    pub grantor: Address,
}
```

### `MilestoneClaim`
```rust
pub struct MilestoneClaim {
    pub milestone_id: u64,
    pub student: Address,
    pub amount: i128,
    pub description: Symbol,
    pub invoice_hash: Option<Symbol>,
    pub is_approved: bool,
    pub is_claimed: bool,
    pub submitted_at: u64,
    pub approved_at: Option<u64>,
    pub claimed_at: Option<u64>,
}
```

## Usage Example

### 1. Grantor Creates Research Grant
```rust
let grant_id = contract.create_research_grant(
    &grantor_address,
    &student_address,
    &5000, // $5,000 for lab equipment
    &token_address
);
```

### 2. Student Submits Milestone Claim
```rust
contract.submit_milestone_claim(
    &student_address,
    &1, // milestone_id
    &5000,
    &Symbol::new(&env, "Lab Equipment Purchase"),
    &Symbol::new(&env, "invoice_hash_abc123")
);
```

### 3. Grantor Approves Claim
```rust
contract.approve_milestone_claim(
    &grantor_address,
    &1 // milestone_id
);
```

### 4. Student Claims Lump Sum
```rust
contract.claim_milestone_lump_sum(
    &student_address,
    &1 // milestone_id
);
```

## Security Features

### 1. **Authorization Controls**
- Only grantors can approve milestone claims
- Only students can submit and claim their own milestones
- Proper authentication checks at each step

### 2. **Fund Security**
- All funds held in contract treasury
- No direct access to grantor funds after creation
- Secure token transfers with validation

### 3. **State Management**
- Milestone claims track approval and claim status
- Prevents double-spending and unauthorized claims
- Timestamp tracking for audit trails

## Integration with Existing Features

### Living Stipend Compatibility
The Research Grant Milestone Escrow operates independently from the existing scholarship system:

- **Scholarship Funds**: Continue to provide daily living stipend drips
- **Research Grants**: Provide lump-sum payments for equipment and research expenses
- **No Interference**: Milestone claims don't affect scholarship balances or drip rates

### Coexistence Example
A student can have:
- A $2,000 scholarship providing daily living stipend
- A $5,000 research grant for equipment purchases
- Both systems operate independently without interference

## Testing

The feature includes comprehensive test coverage:

1. **Full Flow Test**: Complete grant creation → claim submission → approval → claim flow
2. **Authorization Tests**: Verify only authorized parties can perform actions
3. **Validation Tests**: Ensure proper error handling for invalid operations
4. **Coexistence Tests**: Verify compatibility with existing scholarship system

## Gas Optimization

The implementation includes several gas optimization features:

- **Efficient Storage**: Minimal storage keys and TTL management
- **Batch Operations**: Support for multiple milestone operations
- **Event-Driven**: Events for off-chain tracking and UI updates

## Future Enhancements

Potential future improvements:

1. **Multi-Milestone Support**: Batch processing of multiple milestones
2. **Partial Approvals**: Support for partial milestone funding
3. **Oracle Integration**: Automated invoice verification
4. **Delegated Approval**: Support for delegated grantor authorities

## Conclusion

The Research Grant Milestone Escrow feature successfully implements issue #83, providing the financial flexibility needed for complex scientific research projects while maintaining the stability of students' daily income through uninterrupted living stipend drips.
