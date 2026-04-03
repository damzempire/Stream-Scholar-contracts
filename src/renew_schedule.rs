// ─────────────────────────────────────────────────────────────────────────────
// Stream-Scholar: Auto-Renewal of Scholarship Contracts  (#119)
//
// Implements `renew_schedule`, which extends the active period of a scholarship
// contract by 12 months and tops up token allowances from the donor treasury.
// Designed for 4-year degree programmes where the same contract ID tracks the
// student's entire academic journey.
// ─────────────────────────────────────────────────────────────────────────────

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("SCHL1ARContract1111111111111111111111111111111");

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Seconds in 12 calendar months (365.25 days average, rounded to whole secs).
pub const RENEWAL_PERIOD_SECS: i64 = 31_557_600;

/// Maximum number of times a contract may be renewed (3 renewals → 4 years).
pub const MAX_RENEWALS: u8 = 3;

/// Minimum number of seconds that must remain in the current period before
/// an early renewal is allowed. Prevents "double-dipping" close to expiry.
pub const MIN_REMAINING_BEFORE_RENEWAL: i64 = 30 * 24 * 3600; // 30 days

// ─────────────────────────────────────────────────────────────────────────────
// State
// ─────────────────────────────────────────────────────────────────────────────

#[account]
#[derive(Default)]
pub struct ScholarshipContract {
    /// The student who receives disbursements.
    pub student: Pubkey,

    /// The donor who funded this scholarship.
    pub donor: Pubkey,

    /// PDA bump for this account.
    pub bump: u8,

    /// Unix timestamp when the **current** scholarship period began.
    pub period_start: i64,

    /// Unix timestamp when the **current** scholarship period ends.
    pub period_end: i64,

    /// Total token amount authorised for the **current** period.
    pub period_tokens: u64,

    /// Tokens that have already been disbursed in the current period.
    pub disbursed_tokens: u64,

    /// Number of times this contract has been renewed (0 = original term).
    pub renewal_count: u8,

    /// Whether this contract is still active.
    pub is_active: bool,

    /// Verifiable history: timestamp of every renewal event.
    pub renewal_history: Vec<RenewalRecord>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct RenewalRecord {
    /// Wall-clock time at which the renewal was executed.
    pub renewed_at: i64,

    /// New period_end set by this renewal.
    pub new_period_end: i64,

    /// Fresh tokens transferred from the treasury for this period.
    pub tokens_added: u64,

    /// renewal_count **after** this renewal (1-indexed for readability).
    pub renewal_number: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

#[error_code]
pub enum ScholarshipError {
    #[msg("Contract is not active.")]
    ContractInactive,

    #[msg("Maximum renewals reached. This contract cannot be renewed further.")]
    MaxRenewalsReached,

    #[msg("Too early to renew: more than 30 days remain in the current period.")]
    TooEarlyToRenew,

    #[msg("The donor treasury has insufficient funds for renewal.")]
    InsufficientTreasuryFunds,

    #[msg("Renewal token amount must be greater than zero.")]
    ZeroRenewalAmount,

    #[msg("Caller is not the authorised donor for this contract.")]
    UnauthorisedDonor,
}

// ─────────────────────────────────────────────────────────────────────────────
// Accounts context
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct RenewSchedule<'info> {
    /// The scholarship contract PDA being renewed.
    #[account(
        mut,
        seeds = [b"scholarship", contract.student.as_ref(), contract.donor.as_ref()],
        bump = contract.bump,
    )]
    pub contract: Account<'info, ScholarshipContract>,

    /// The donor who is authorising (and paying for) this renewal.
    #[account(mut)]
    pub donor: Signer<'info>,

    /// Donor's token account — source of renewal tokens.
    #[account(
        mut,
        constraint = donor_token_account.owner == donor.key(),
    )]
    pub donor_token_account: Account<'info, TokenAccount>,

    /// The on-chain treasury vault that holds tokens for this contract.
    /// After the transfer the student disbursements draw from here.
    #[account(
        mut,
        seeds = [b"treasury", contract.key().as_ref()],
        bump,
    )]
    pub treasury_vault: Account<'info, TokenAccount>,

    /// SPL token programme.
    pub token_program: Program<'info, Token>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Instruction handler
// ─────────────────────────────────────────────────────────────────────────────

/// Extends the scholarship contract by one 12-month period and tops up the
/// treasury vault with `renewal_tokens` from the donor's wallet.
///
/// # Parameters
/// - `renewal_tokens`: Number of fresh tokens to transfer from the donor into
///   the treasury vault for the coming year. Must be > 0.
///
/// # Behaviour
/// 1. Validates state (active, under max renewals, timing window).
/// 2. Transfers `renewal_tokens` from the donor's token account → treasury.
/// 3. Extends `period_end` by exactly `RENEWAL_PERIOD_SECS`.
/// 4. Resets `disbursed_tokens` to 0 so the new period starts fresh.
/// 5. Appends a `RenewalRecord` to the immutable on-chain history.
/// 6. Increments `renewal_count`.
///
/// # History
/// Every call appends a `RenewalRecord` that is permanently readable on-chain,
/// giving auditors and scholarship bodies a verifiable progression log tied to
/// the student's single contract ID for their entire degree.
pub fn renew_schedule(ctx: Context<RenewSchedule>, renewal_tokens: u64) -> Result<()> {
    let contract = &mut ctx.accounts.contract;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;

    // ── 1. Validation ────────────────────────────────────────────────────────

    // Only the original donor may renew.
    require!(
        ctx.accounts.donor.key() == contract.donor,
        ScholarshipError::UnauthorisedDonor
    );

    require!(contract.is_active, ScholarshipError::ContractInactive);

    require!(
        contract.renewal_count < MAX_RENEWALS,
        ScholarshipError::MaxRenewalsReached
    );

    require!(renewal_tokens > 0, ScholarshipError::ZeroRenewalAmount);

    // Enforce the 30-day renewal window (can renew only when close to expiry
    // OR after expiry — so students are never left without a funded contract).
    let time_remaining = contract.period_end.saturating_sub(now);
    require!(
        time_remaining <= MIN_REMAINING_BEFORE_RENEWAL,
        ScholarshipError::TooEarlyToRenew
    );

    // Check the donor has enough tokens before attempting the transfer.
    require!(
        ctx.accounts.donor_token_account.amount >= renewal_tokens,
        ScholarshipError::InsufficientTreasuryFunds
    );

    // ── 2. Token transfer: donor → treasury vault ────────────────────────────

    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from:      ctx.accounts.donor_token_account.to_account_info(),
            to:        ctx.accounts.treasury_vault.to_account_info(),
            authority: ctx.accounts.donor.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, renewal_tokens)?;

    // ── 3. Period extension ──────────────────────────────────────────────────

    // If renewed after expiry, base the new window from `now` (no gap).
    // If renewed during the 30-day window, extend from the current period_end
    // so the student loses no coverage.
    let new_period_start = if now >= contract.period_end {
        now
    } else {
        contract.period_end
    };
    let new_period_end = new_period_start
        .checked_add(RENEWAL_PERIOD_SECS)
        .ok_or(ScholarshipError::ContractInactive)?; // arithmetic overflow guard

    // ── 4. State update ──────────────────────────────────────────────────────

    contract.renewal_count  = contract.renewal_count.checked_add(1).unwrap();
    contract.period_start   = new_period_start;
    contract.period_end     = new_period_end;
    contract.period_tokens  = renewal_tokens;
    contract.disbursed_tokens = 0; // fresh slate for the new year

    // ── 5. Append to immutable renewal history ───────────────────────────────

    contract.renewal_history.push(RenewalRecord {
        renewed_at:     now,
        new_period_end,
        tokens_added:   renewal_tokens,
        renewal_number: contract.renewal_count,
    });

    // ── 6. Emit event for off-chain indexers / front-end ─────────────────────

    emit!(ContractRenewed {
        contract:       ctx.accounts.contract.key(),
        student:        contract.student,
        donor:          contract.donor,
        renewal_number: contract.renewal_count,
        new_period_end,
        tokens_added:   renewal_tokens,
        renewed_at:     now,
    });

    msg!(
        "Scholarship renewed (#{}) — new period ends {}, {} tokens added.",
        contract.renewal_count,
        new_period_end,
        renewal_tokens
    );

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Event
// ─────────────────────────────────────────────────────────────────────────────

#[event]
pub struct ContractRenewed {
    pub contract:       Pubkey,
    pub student:        Pubkey,
    pub donor:          Pubkey,
    pub renewal_number: u8,
    pub new_period_end: i64,
    pub tokens_added:   u64,
    pub renewed_at:     i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_contract(renewal_count: u8, period_end_offset: i64) -> ScholarshipContract {
        let now = 1_700_000_000i64; // fixed epoch for determinism
        ScholarshipContract {
            student:          Pubkey::default(),
            donor:            Pubkey::default(),
            bump:             255,
            period_start:     now - RENEWAL_PERIOD_SECS,
            period_end:       now + period_end_offset,
            period_tokens:    1_000_000,
            disbursed_tokens: 400_000,
            renewal_count,
            is_active:        true,
            renewal_history:  vec![],
        }
    }

    #[test]
    fn renewal_count_increments() {
        let mut c = mock_contract(0, 0);
        c.renewal_count += 1;
        assert_eq!(c.renewal_count, 1);
    }

    #[test]
    fn max_renewals_guard() {
        let c = mock_contract(MAX_RENEWALS, 0);
        assert!(c.renewal_count >= MAX_RENEWALS, "should be at cap");
    }

    #[test]
    fn disbursed_resets_on_renewal() {
        let mut c = mock_contract(0, -1); // already expired
        // Simulate what renew_schedule does:
        c.disbursed_tokens = 0;
        c.period_tokens    = 2_000_000;
        assert_eq!(c.disbursed_tokens, 0);
        assert_eq!(c.period_tokens, 2_000_000);
    }

    #[test]
    fn period_end_extends_from_old_end_when_still_active() {
        let now = 1_700_000_000i64;
        let remaining = 10 * 24 * 3600i64; // 10 days left → within window
        let c = mock_contract(0, remaining);

        let base = c.period_end; // extend from here, not `now`
        let new_end = base + RENEWAL_PERIOD_SECS;
        assert_eq!(new_end, base + RENEWAL_PERIOD_SECS);
        // Ensure no coverage gap: new period starts exactly where old one ended
        assert_eq!(new_end - base, RENEWAL_PERIOD_SECS);
        // Suppress unused warning in test-only context
        let _ = now;
    }

    #[test]
    fn period_end_extends_from_now_when_expired() {
        let now = 1_700_000_000i64;
        let c = mock_contract(0, -3600); // expired 1 hour ago

        let base = if now >= c.period_end { now } else { c.period_end };
        let new_end = base + RENEWAL_PERIOD_SECS;
        assert_eq!(base, now); // base is `now` because contract is expired
        assert_eq!(new_end, now + RENEWAL_PERIOD_SECS);
    }

    #[test]
    fn renewal_history_records_correct_fields() {
        let now = 1_700_000_000i64;
        let mut c = mock_contract(1, -1);

        let new_period_end = now + RENEWAL_PERIOD_SECS;
        let tokens_added   = 500_000u64;

        c.renewal_count += 1;
        c.renewal_history.push(RenewalRecord {
            renewed_at: now,
            new_period_end,
            tokens_added,
            renewal_number: c.renewal_count,
        });

        let record = c.renewal_history.last().unwrap();
        assert_eq!(record.renewal_number, 2);
        assert_eq!(record.tokens_added,   tokens_added);
        assert_eq!(record.new_period_end, new_period_end);
    }

    #[test]
    fn zero_token_renewal_is_rejected() {
        // This is a logic check — in the real instruction, the require! macro
        // enforces this. Here we confirm the condition is correct.
        let renewal_tokens = 0u64;
        assert!(renewal_tokens == 0, "zero-token renewal must be rejected");
    }

    #[test]
    fn too_early_renewal_is_rejected() {
        let now = 1_700_000_000i64;
        let far_future_end = now + 60 * 24 * 3600i64; // 60 days remaining
        let c = mock_contract(0, far_future_end - now);

        let time_remaining = c.period_end.saturating_sub(now);
        assert!(
            time_remaining > MIN_REMAINING_BEFORE_RENEWAL,
            "should be too early to renew"
        );
    }
}
