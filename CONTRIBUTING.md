# Contributing to Stream Scholar

Thank you for your interest in contributing to Stream Scholar! This document provides guidelines and instructions for contributing educational content and reporting bugs in the streaming logic.

## Table of Contents

- [Contributing Educational Content](#contributing-educational-content)
- [Reporting Streaming Logic Bugs](#reporting-streaming-logic-bugs)
- [Code of Conduct](#code-of-conduct)
- [Getting Help](#getting-help)

---

## Contributing Educational Content

We welcome contributions of educational content to the Stream Scholar platform. Follow these steps to contribute:

### 1. Course Creation Process

#### Prerequisites
- You must be registered as a teacher on the platform
- Your course must comply with our content standards
- All course materials must be original or properly licensed

#### Steps to Add a Course

```rust
// Example: Adding a course to the registry
client.add_course_to_registry(&course_id, &teacher_address);
```

**Required Information:**
- **Course ID**: Unique identifier for your course
- **Course Metadata**: Follow the standard defined in `docs/course-metadata-standard.json`
- **Duration**: Total watch time required for completion (in seconds)
- **Description**: Clear learning objectives and prerequisites

### 2. Course Metadata Standards

All courses must include metadata following this structure:

```json
{
  "course_id": "unique_identifier",
  "title": "Course Title",
  "description": "Brief description of the course",
  "duration_seconds": 3600,
  "difficulty_level": "beginner|intermediate|advanced",
  "prerequisites": [],
  "learning_objectives": [],
  "modules": [
    {
      "module_id": 1,
      "title": "Module Title",
      "duration_seconds": 600,
      "quiz_required": true
    }
  ]
}
```

See `docs/course-metadata-implementation-guide.md` for detailed implementation instructions.

### 3. Content Guidelines

**DO:**
- Create original, high-quality educational content
- Structure content in logical, sequential modules
- Include knowledge checks (quizzes) at module ends
- Provide clear learning outcomes
- Use professional language and formatting

**DON'T:**
- Upload copyrighted material without permission
- Include misleading or inaccurate information
- Create excessively short or low-effort content
- Spam or self-promote without educational value

### 4. Course Review Process

1. Submit your course via the `add_course_to_registry` function
2. Admin review within 5-7 business days
3. Address any feedback provided
4. Course goes live upon approval

### 5. Teacher Responsibilities

- Maintain course content accuracy
- Respond to student questions within 48 hours
- Update content when errors are reported
- Participate in platform governance

---

## Reporting Streaming Logic Bugs

We take streaming logic issues seriously. Follow this process to report bugs effectively.

### Before Reporting

**Check existing issues:** Search the issue tracker to avoid duplicates

**Verify the bug:** Ensure you can reproduce it consistently

**Gather information:**
- Contract version/hash
- Network (testnet/mainnet)
- Transaction hashes
- Error messages
- Timestamp of occurrence

### Bug Report Template

Use this template when reporting streaming logic bugs:

```markdown
### Bug Description
[Clear description of what's happening]

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happened]

### Steps to Reproduce
1. [First step]
2. [Second step]
3. [And so on...]

### Environment
- Network: [testnet/mainnet]
- Contract ID: [CB7OZPTIUENDWJWNHRGDPZLIEIS6TXMFRYT4WCGHIZVYLCTXEONC6VHY]
- Transaction Hash: [if applicable]
- Timestamp: [UTC time]

### Additional Context
[Any other relevant information]
```

### Critical vs Non-Critical Bugs

**Critical (Report Immediately):**
- Session sharing vulnerabilities
- Payment/heartbeat exploits
- Double-spending issues
- Authorization bypasses
- Fund loss or theft risks

**Non-Critical (Standard Process):**
- UI/UX issues
- Documentation errors
- Feature requests
- Performance optimizations

### Testing Streaming Features

When testing streaming functionality:

1. **Heartbeat Mechanism:**
   ```rust
   // Test normal heartbeat flow
   client.heartbeat(&student, &course_id, &session_hash);
   ```

2. **Session Validation:**
   - Verify single-session enforcement
   - Test session timeout behavior
   - Check session takeover scenarios

3. **Watch Time Tracking:**
   - Confirm accurate time calculation
   - Verify SBT minting triggers
   - Test edge cases (disconnects, etc.)

### Security Considerations

**DO NOT:**
- Attempt to exploit vulnerabilities for personal gain
- Share sensitive security details publicly before fixes
- Use testnet exploits to manipulate mainnet

**DO:**
- Practice responsible disclosure
- Provide minimal proof-of-concept code
- Work with maintainers on fixes

---

## Code Contributions

### Development Setup

1. Clone the repository
2. Install Rust toolchain: `rustup update`
3. Install Soroban CLI
4. Build contracts: `cargo build --release --target wasm32-unknown-unknown`

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_scholarship_flow

# Run with logging
RUST_LOG=debug cargo test
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add/update tests
5. Ensure all tests pass
6. Update documentation if needed
7. Submit PR with clear description

### Code Style

- Follow Rust idioms and best practices
- Use meaningful variable/function names
- Comment complex logic
- Keep functions focused and small
- Write tests for new features

---

## Common Issues and Solutions

### Issue: Heartbeat Rejected

**Symptoms:** Student receives error during heartbeat

**Possible Causes:**
- Session already active with different hash
- Heartbeat sent too frequently
- Access expired

**Solution:**
1. Check session hash consistency
2. Verify heartbeat timing (>60s interval)
3. Confirm access hasn't expired

### Issue: Course Not Accessible

**Symptoms:** `has_access` returns false despite payment

**Possible Causes:**
- Course globally vetoed
- Student-specific veto
- Transaction not confirmed

**Solution:**
1. Check veto status: `vetoed_course_global` or `vetoed_course_access`
2. Verify transaction on block explorer
3. Contact admin if incorrectly vetoed

### Issue: Scholarship Transfer Failed

**Symptoms:** Cannot transfer scholarship funds to teacher

**Possible Causes:**
- Teacher not approved
- Insufficient scholarship balance
- Missing authorization

**Solution:**
1. Verify teacher is approved: `set_teacher` by admin
2. Check scholarship balance
3. Ensure student authorization

---

## Getting Help

- **Documentation:** Check `/docs` directory
- **Issues:** Search existing GitHub issues
- **Discord:** Join Stellar Developer Discord
- **Email:** Contact maintainers for sensitive issues

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Annual contributor highlights

Thank you for contributing to Stream Scholar! 🎓
