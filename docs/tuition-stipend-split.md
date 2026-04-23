# Tuition-Direct Drip vs Stipend Split Feature

## Overview

This feature implements a split-stream mechanism for educational funding that automatically divides payments between university tuition (70%) and student stipend (30%). This ensures that educational institutions are paid first, reducing the risk of students spending tuition money on living expenses and being forced to drop out due to unpaid fees.

## Key Features

- **Automatic Split**: Configurable 70/30 split (default) between university and student
- **Priority Payment**: University receives payment first, ensuring tuition is covered
- **Flexible Configuration**: Admin can set custom split percentages per student
- **Event Tracking**: All split distributions emit events for transparency
- **Backward Compatibility**: Works with existing scholarship and course payment systems

## Core Components

### 1. TuitionStipendSplit Structure
```rust
pub struct TuitionStipendSplit {
    pub university_address: Address,
    pub student_address: Address,
    pub university_percentage: u32, // Default 70
    pub student_percentage: u32,     // Default 30
}
```

### 2. Key Functions

#### `set_tuition_stipend_split`
Configures the split for a specific student.
- **Parameters**: admin, student, university_address, university_percentage, student_percentage
- **Validation**: Ensures percentages sum to 100%
- **Authorization**: Only admin can configure splits

#### `get_tuition_stipend_split`
Retrieves the split configuration for a student.
- **Returns**: Option<TuitionStipendSplit>

#### `distribute_tuition_stipend_split`
Core split logic that distributes funds according to configuration.
- **Priority**: University paid first
- **Returns**: Tuple of (university_amount, student_amount)

## Integration Points

### 1. Scholarship Funding
When `fund_scholarship` is called:
1. Full amount is transferred to contract
2. Split is applied automatically
3. University portion sent immediately
4. Student portion added to scholarship balance

### 2. Course Purchases
When `buy_access` is called:
1. Payment processed through contract
2. Split applied to course fees
3. University receives tuition portion
4. Student's access granted based on full payment
5. Course creator royalties handled separately

## Usage Example

```rust
// Admin configures split for a student
contract.set_tuition_stipend_split(
    &admin,
    &student_address,
    &university_address,
    &70, // 70% to university
    &30  // 30% to student
);

// Someone funds scholarship - split happens automatically
contract.fund_scholarship(
    &funder,
    &student_address,
    &1000, // $1000
    &token_address
);
// Result: University gets $700, Student scholarship gets $300
```

## Events

### TuitionStipendSplit_Configured
Emitted when a split is configured:
- Topics: admin, student
- Data: university_address, university_percentage, student_percentage

### TuitionStipendSplit_Distributed
Emitted when funds are split:
- Topics: student, university_address
- Data: university_amount, student_amount

### Scholarship_Granted (Updated)
Now includes split information:
- Topics: funder, student
- Data: total_amount, university_amount, student_amount

### Access_Purchased (Updated)
Now includes split information:
- Topics: student, course_id
- Data: total_cost, university_share, student_share, seconds_bought

## Security Considerations

1. **Admin Only**: Only authorized admins can configure splits
2. **Percentage Validation**: Ensures splits always sum to 100%
3. **Priority Payment**: University paid first to guarantee tuition coverage
4. **Atomic Operations**: Split and distribution happen in single transaction

## Testing

The feature includes comprehensive tests covering:
- Split configuration and validation
- Fund distribution with correct percentages
- Error handling for invalid configurations
- Backward compatibility when no split is configured

Run tests with:
```bash
make test
```

## Future Enhancements

1. **Time-based Splits**: Different percentages for different academic periods
2. **Performance-based Splits**: Adjust splits based on academic performance
3. **Multiple Beneficiaries**: Support for more complex split scenarios
4. **Split Templates**: Pre-configured split templates for different programs
