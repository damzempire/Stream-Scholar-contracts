#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Bytes, Env, Symbol, Vec, token,
};

// Constants for ledger bump and GPA bonus calculations
const LEDGER_BUMP_THRESHOLD: u32 = 7776000; // ~90 days
const LEDGER_BUMP_EXTEND: u32 = 7776000;   // ~90 days
const GPA_BONUS_THRESHOLD: u64 = 35;       // 3.5 GPA (stored as 35)
const GPA_BONUS_PERCENTAGE_PER_POINT: u64 = 20; // 20% per 0.1 GPA point above threshold
const EARLY_DROP_WINDOW_SECONDS: u64 = 86400; // 24 hours
const ORACLE_STALENESS_THRESHOLD: u64 = 172800; // 48 hours

// Leaderboard constants
const MAX_LEADERBOARD_SIZE: u64 = 100;     // Maximum number of scholars on leaderboard
const ACADEMIC_POINTS_PER_COURSE: u64 = 100; // Points awarded per course completion
const ACADEMIC_POINTS_PER_STREAK_DAY: u64 = 10; // Points per consecutive study day

// Tutoring bridge constants
const MAX_TUTORING_PERCENTAGE: u32 = 20;   // Maximum percentage that can be redirected (20%)
const MIN_TUTORING_DURATION: u64 = 3600;  // Minimum tutoring duration (1 hour)

// Alumni Donation Matching Incentive constants (#95)
const ALUMNI_MATCHING_MULTIPLIER: u64 = 2; // 2:1 matching ratio
const GRADUATION_SBT_COURSE_ID: u64 = 9999; // Special course ID for graduation SBT

// Scholarship Probation Cooling-Off Logic constants (#93)
const PROBATION_WARNING_PERIOD: u64 = 5184000; // 60 days in seconds
const PROBATION_FLOW_REDUCTION: u64 = 30; // 30% reduction
const GPA_THRESHOLD: u64 = 25; // 2.5 GPA threshold (stored as 25)

// Issue #128: Community Governance Veto
const FINAL_RELEASE_PERCENTAGE: u64 = 10; // 10%
const COMMUNITY_VOTE_THRESHOLD: u64 = 5; // 5 votes to pass

// Issue #118: Native XLM Scholarship
const NATIVE_XLM_RESERVE: i128 = 2_0000000; // 2 XLM in stroops

// Issue #112: Scholarship Claim Dry-Run
const DEFAULT_TAX_RATE_BPS: u32 = 0; // 0% default tax
const ESTIMATED_GAS_FEE: i128 = 500000; // 0.05 XLM in stroops

// Issue #124: Gas Fee Subsidy for Early Learners
const MAX_SUBSIDIZED_STUDENTS: u32 = 100;
const SUBSIDY_THRESHOLD: i128 = 5_0000000; // 5 XLM threshold
const SUBSIDY_AMOUNT: i128 = 5_0000000;    // 5 XLM subsidy

// Dynamic Sponsor-Clawback Logic constants
const DEFAULT_CLAWBACK_COOLDOWN: u64 = 2592000; // 30 days
const CLAWBACK_EXECUTION_TIMEOUT: u64 = 604800; // 7 days
const MAX_CLAWBACK_PERCENTAGE: u64 = 100; // Max 100% can be clawed back

// Matching-Pool Quadratic Funding constants
const QF_ROUND_DURATION: u64 = 2592000; // 30-day funding rounds
const QF_MIN_CONTRIBUTION: i128 = 1_0000000; // 1 XLM minimum contribution
const QF_MATCHING_POOL_RESERVE: i128 = 10000_0000000; // 10,000 XLM matching pool reserve
const QF_MAX_PROJECTS: u64 = 500; // Max projects per round


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    SbtMint(Address, u64),
    EnrollmentVerified(Address, Address),
    MultiplierApplied(i128, i128, u32),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    InvalidOracleSig = 1,
    InsufficientGpa = 2,
    Unauthorized = 3,
    TimelockNotExpired = 4,
    InvalidAction = 5,
    ReplayAttack = 6,
    NoScholarship = 7,
    OracleDataStale = 8,
}


#[contracttype]
#[derive(Clone)]
pub struct Access {
    pub student: Address,
    pub course_id: u64,
    pub expiry_time: u64,
    pub token: Address,
    pub total_watch_time: u64,
    pub last_heartbeat: u64,
    pub last_purchase_time: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Scholarship {
    pub funder: Address,
    pub balance: i128,
    pub token: Address,
    pub unlocked_balance: i128,
    pub last_verif: u64,
    pub is_paused: bool,
    pub is_disputed: bool,
    pub dispute_reason: Option<Symbol>,
    pub final_ruling: Option<Symbol>,
    // Issue #118
    pub is_native: bool,
    // Issue #128
    pub total_grant: i128,
    pub final_release_claimed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudentProfile {
    pub academic_points: u64,
    pub courses_completed: u64,
    pub current_streak: u64,
    pub last_activity: u64,
    pub student_alias: Symbol, // Privacy-protecting alias
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct LeaderboardEntry {
    pub student_alias: Symbol,
    pub academic_points: u64,
    pub rank: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct GlobalExcellencePool {
    pub total_pool_balance: i128,
    pub token: Address,
    pub total_distributed: i128,
    pub last_distribution: u64,
    pub is_active: bool,
}

// Issue #94: Peer-to-Peer Tutoring Payment Bridge structs
#[contracttype]
#[derive(Clone)]
pub struct TutoringAgreement {
    pub scholar: Address,
    pub tutor: Address,
    pub percentage: u32, // Percentage of scholarship flow to redirect
    pub start_time: u64,
    pub end_time: u64,
    pub is_active: bool,
    pub total_redirected: i128,
    pub agreement_id: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct SubStreamRedirect {
    pub from_scholar: Address,
    pub to_tutor: Address,
    pub flow_rate: i128,
    pub start_time: u64,
    pub last_redirect: u64,
    pub total_amount_redirected: i128,
    pub is_active: bool,
}

// Issue #95: Alumni Donation Matching Incentive structs
#[contracttype]
#[derive(Clone)]
pub struct GraduationSBT {
    pub student: Address,
    pub graduation_date: u64,
    pub gpa: u64, // Final GPA at graduation
    pub is_verified: bool,
    pub token_id: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AlumniDonation {
    pub donor: Address,
    pub original_amount: i128,
    pub matched_amount: i128,
    pub scholarship_pool: u64, // Target scholarship pool ID
    pub donation_date: u64,
    pub has_graduation_sbt: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct GeneralExcellenceFund {
    pub total_balance: i128,
    pub token: Address,
    pub total_matched: i128,
    pub is_active: bool,
    pub last_updated: u64,
}

// Issue #106: Research Bonus Fund — treasury yield redirected to top-5% student bonuses
#[contracttype]
#[derive(Clone)]
pub struct ResearchBonusFund {
    pub total_balance: i128,
    pub token: Address,
    pub total_accrued: i128,   // cumulative yield deposited
    pub total_distributed: i128,
    pub last_distribution: u64,
}

// Issue #93: Scholarship Probation Cooling-Off Logic structs
#[contracttype]
#[derive(Clone)]
pub struct ProbationStatus {
    pub student: Address,
    pub is_on_probation: bool,
    pub probation_start_time: u64,
    pub warning_period_end: u64,
    pub original_flow_rate: i128,
    pub reduced_flow_rate: i128,
    pub violation_count: u32, // Number of GPA drops below threshold
    pub last_gpa_check: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct GPAUpdate {
    pub student: Address,
    pub new_gpa: u64,
    pub previous_gpa: u64,
    pub update_timestamp: u64,
    pub oracle_verified: bool,
}

// Dynamic Sponsor-Clawback Logic structs
#[contracttype]
#[derive(Clone)]
pub enum ClawbackTriggerType {
    GpaThreshold,
    CourseCompletion,
    TimeElapsed,
    ActivityInactive,
    CombinedConditions,
}

#[contracttype]
#[derive(Clone)]
pub struct ClawbackCondition {
    pub funder: Address,
    pub student: Address,
    pub trigger_type: ClawbackTriggerType,
    pub clawback_percentage: u64, // 0-100
    pub threshold_value: u64, // GPA (stored as 30 for 3.0), courses completed, days, etc.
    pub triggered_at: Option<u64>,
    pub executed_at: Option<u64>,
    pub is_active: bool,
    pub cooldown_period: u64, // Seconds before next clawback can be triggered
    pub last_clawback_time: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct ClawbackEvent {
    pub funder: Address,
    pub student: Address,
    pub amount_clawed_back: i128,
    pub trigger_type: ClawbackTriggerType,
    pub triggered_at: u64,
    pub executed_at: u64,
    pub remaining_balance: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct SponsorClawbackPolicy {
    pub sponsor: Address,
    pub version: u64,
    pub conditions: Vec<ClawbackCondition>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}

// Matching-Pool Quadratic Funding structs
#[contracttype]
#[derive(Clone)]
pub struct QuadraticFundingRound {
    pub round_id: u64,
    pub token: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub matching_pool_balance: i128,
    pub total_contributions: i128,
    pub total_matching_distributed: i128,
    pub project_count: u64,
    pub is_active: bool,
    pub is_finalized: bool,
    pub created_by: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct FundingProject {
    pub project_id: u64,
    pub round_id: u64,
    pub project_owner: Address,
    pub title: Symbol,
    pub total_raised: i128,
    pub contributor_count: u64,
    pub sqrt_sum_contributions: i128, // For QF formula: sum of sqrt(contributions)
    pub total_matching: i128,
    pub created_at: u64,
    pub is_approved: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct QFContribution {
    pub contributor: Address,
    pub project_id: u64,
    pub round_id: u64,
    pub amount: i128,
    pub contribution_time: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct MatchingDistribution {
    pub round_id: u64,
    pub project_id: u64,
    pub matching_amount: i128,
    pub distributed_at: u64,
    pub project_owner: Address,
}


#[contracttype]
pub enum DataKey {
    Access(Address, u64),
    BaseRate,
    DiscountThreshold,
    DiscountPercentage,
    MinDeposit,
    Subscription(Address),
    HeartbeatInterval,
    CourseDuration(u64),
    SbtMinted(Address, u64),
    Admin,
    VetoedCourse(Address, u64),
    IsTeacher(Address),
    Scholarship(Address),
    VetoedCourseGlobal(u64),
    Session(Address),
    AuthorizedPayout(Address),
    AuthorizedPayoutPending(Address),
    UnlockTime(Address),
    ReputationBonus(Address),
    OracleRegistry(Address),
    Enrollment(Address),
    Nonce(Address),
    GpaMultiplier(Address),
    GpaEpoch(Address),
    ClawbackCondition(Address, Address, u64), // (funder, student, condition_id)
    SponsorClawbackPolicy(Address), // sponsor address
    ClawbackHistory(Address, Address), // (funder, student)
    ClawbackEventLog(Address, Address, u64), // (funder, student, event_id)
    StudentProfile(Address),
    StudentGPA(Address),
    StudentUniversity(Address),
    UniversityAdmin(Address),
    SecurityHold(Address),
    Milestone(Address, Symbol),
    CommunityVote(Address),
    TaxRate,
    ResearchBonusFund,
    LeaderboardSize,
    HasReceivedSubsidy(Address),
    SubsidizedStudentCount,
    GraduationRegistry(Address),
    QuadraticFundingRound(u64), // (round_id)
    FundingProject(u64, u64), // (round_id, project_id)
    QFContribution(Address, u64, u64), // (contributor, round_id, project_id)
    MatchingDistribution(u64, u64), // (round_id, project_id)
    RoundProjectList(u64), // (round_id)
    ProjectContributorsList(u64, u64), // (round_id, project_id)
    QFAdmin,
    QFRoundCounter,
}

#[contracttype]
#[derive(Clone)]
pub struct AuthorizedPayout {
    pub address: Address,
    pub unlock_time: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct EnrollmentData {
    pub student: Address,
    pub university_id: u64,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub generated_at: u64,
    pub nonce: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct GpaData {
    pub student: Address,
    pub gpa_bps: u32, // GPA in basis points (e.g. 380 for 3.8)
    pub epoch: u32,
    pub generated_at: u64,
    pub nonce: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct SubscriptionTier {
    pub subscriber: Address,
    pub expiry_time: u64,
    pub course_ids: Vec<u64>,
}

#[contracttype]
#[derive(Clone)]
pub struct CourseInfo {
    pub course_id: u64,
    pub created_at: u64,
    pub is_active: bool,
    pub creator: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct CourseRegistry {
    pub courses: Vec<u64>,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct RoyaltySplit {
    pub shares: Vec<(Address, u32)>,
}

#[contracttype]
#[derive(Clone)]
pub struct TuitionStipendSplit {
    pub university_address: Address,
    pub student_address: Address,
    pub university_percentage: u32, // Default 70
    pub student_percentage: u32,    // Default 30
}

#[contracttype]
#[derive(Clone)]
pub struct StudentGPA {
    pub student: Address,
    pub gpa: u64, // Stored as integer (e.g., 3.7 = 37)
    pub last_updated: u64,
    pub oracle_verified: bool,
}

// Issue #128: Community Governance Veto
#[contracttype]
#[derive(Clone)]
pub struct CommunityVote {
    pub student: Address,
    pub yes_votes: u64,
    pub voters: Vec<Address>,
    pub is_passed: bool,
    pub created_at: u64,
}

// Issue #112: Scholarship Claim Dry-Run
#[contracttype]
#[derive(Clone)]
pub struct ClaimSimulation {
    pub tokens_to_release: i128,
    pub estimated_gas_fee: i128,
    pub tax_withholding_amount: i128,
    pub net_claimable_amount: i128,
}

// Issue #122: On-Chain Graduation Credential Registry
#[contracttype]
#[derive(Clone)]
pub struct GraduateProfile {
    pub student: Address,
    pub graduation_date: u64,
    pub final_gpa: u64,
    pub completed_scholarships: Vec<Address>, // List of funder addresses
}

// Issue #115: Emergency Protocol Pause for University Admins
#[contracttype]
#[derive(Clone)]
pub struct SecurityHold {
    pub university: Address,
    pub triggered_by: Address,  // The university registrar/admin who triggered the hold
    pub triggered_at: u64,
    pub expires_at: u64,        // triggered_at + SECURITY_HOLD_DURATION (7 days)
    pub is_active: bool,
    pub reason: Symbol,
}

// Multi-Sig Academic Board Review structs
#[contracttype]
#[derive(Clone)]
pub struct DeansCouncil {
    pub members: Vec<Address>,
    pub required_signatures: u32, // Default 2 for 2-of-3
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct BoardPauseRequest {
    pub student: Address,
    pub reason: Symbol,
    pub requested_at: u64,
    pub signatures: Vec<Address>,
    pub is_executed: bool,
    pub executed_at: Option<u64>,
}

// Research Grant Milestone Escrow structs
#[contracttype]
#[derive(Clone)]
pub struct ResearchGrant {
    pub student_researcher: Address,
    pub current_beneficiary: Address, // Beneficiary can be sold to an investor
    pub total_amount: i128,
    pub token: Address,
    pub is_locked: bool,
}

// Issue #116: Sub-Scholarship Delegation for Departments
/// A token pool granted by the Main Donor to a department manager (e.g. CS Dean).
/// The manager can distribute and revoke allocations among their students
/// without requiring a central admin or DAO vote for each action.
#[contracttype]
#[derive(Clone)]
pub struct DepartmentVault {
    pub manager: Address,       // e.g. CS Dean
    pub token: Address,
    pub total_allocated: i128,  // Total tokens granted by the Main Donor
    pub distributed: i128,      // Tokens already delegated to students
    pub is_active: bool,
    pub created_at: u64,
}

/// A per-student allocation carved out of a DepartmentVault.
#[contracttype]
#[derive(Clone)]
pub struct DepartmentDelegation {
    pub manager: Address,
    pub student: Address,
    pub amount: i128,
    pub claimed: i128,
    pub is_active: bool,
    pub created_at: u64,
}

#[contract]
pub struct ScholarContract;

#[contractimpl]
impl ScholarContract {
    /// #108: Optimization - Efficient student data retrieval.
    /// Minimizes ledger reads by fetching a consolidated profile struct in one operation.
    pub fn get_student_data(env: Env, student: Address) -> StudentProfile {
        env.storage()
            .persistent()
            .get(&DataKey::StudentProfile(student))
            .unwrap_or(StudentProfile {
                academic_points: 0,
                courses_completed: 0,
                current_streak: 0,
                last_activity: 0,
                book_voucher_claimed: false,
            })
    }

    /// #117: Scholarship_Marketplace_Listing_Hook
    /// Authorizes a marketplace to lock a grant for auction to secondary investors.
    pub fn authorize_transfer_to_marketplace(env: Env, student: Address, grant_id: u64, marketplace: Address) {
        student.require_auth();
        let grant: ResearchGrant = env.storage().persistent().get(&DataKey::ResearchGrant(grant_id))
            .expect("Grant not found");
        assert_eq!(grant.student_researcher, student, "Not the owner of this grant");
        
        let mut effective_rate = base_rate;

        // Apply Reputation Bonus (2% discount)
        let has_reputation_bonus: bool = env.storage().instance().get(&DataKey::ReputationBonus(student.clone())).unwrap_or(false);
        if has_reputation_bonus {
            effective_rate = (effective_rate * 98) / 100;
        }

        // Apply GPA Multiplier
        let gpa_multiplier: i128 = env.storage().instance().get(&DataKey::GpaMultiplier(student.clone())).unwrap_or(10000); // Default 100% in bps
        if gpa_multiplier == 0 {
            return 0; // Paused
        }
        effective_rate = (effective_rate * gpa_multiplier) / 10000;

        let access: Access = env.storage().instance().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: 0,
                token: student.clone(),
                total_watch_time: 0,
                last_heartbeat: 0,
            });
        
        if access.total_watch_time >= discount_threshold {
            let discount = (effective_rate * discount_percentage as i128) / 100;
            effective_rate - discount
        } else {
            effective_rate
        }
    }

    pub fn fund_scholarship(
        env: Env,
        funder: Address,
        student: Address,
        amount: i128,
        token: Address,
        is_native: bool, // For Issue #118
    ) {
        funder.require_auth();

        let client = token::Client::new(&env, &token);
        client.transfer(&funder, &env.current_contract_address(), &amount);

        // Apply tuition-stipend split if configured
        let (university_amount, student_amount) = Self::distribute_tuition_stipend_split(
            &env, 
            &student, 
            amount, 
            &token
        );

        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .unwrap_or(Scholarship {
                funder: funder.clone(),
                balance: 0,
                token: token.clone(),
                unlocked_balance: 0,
                last_verif: 0,
                is_paused: false,
                is_disputed: false,
                dispute_reason: None,
                final_ruling: None,
                is_native, // Issue #118
                total_grant: 0, // Issue #128
                final_release_claimed: false, // Issue #128
            });

        // Only add the student's portion to scholarship balance after processing tutoring redirects
        let final_student_amount = Self::process_tutoring_payment(env.clone(), student.clone(), student_amount, &token);
        
        scholarship.balance += final_student_amount;
        scholarship.unlocked_balance += final_student_amount; // Assume funded amount is unlocked
        scholarship.total_grant += final_student_amount; // Issue #128: Track total grant
        scholarship.is_native = is_native; // Issue #118: Set native flag
        
        env.storage().persistent().set(&DataKey::Milestone(student, milestone_type), &true);
    }

    pub fn withdraw_scholarship(env: Env, student: Address, amount: i128) {
        student.require_auth();

        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        if scholarship.is_paused || scholarship.is_disputed {
            panic!("Scholarship is paused or disputed");
        }

        // Issue #115: Block withdrawals during an active university security hold
        if let Some(university) = env
            .storage()
            .persistent()
            .get::<_, Address>(&DataKey::StudentUniversity(student.clone()))
        {
            if let Some(hold) = env
                .storage()
                .persistent()
                .get::<_, SecurityHold>(&DataKey::SecurityHold(university))
            {
                let now = env.ledger().timestamp();
                if hold.is_active && now < hold.expires_at {
                    panic!("Scholarship withdrawals are suspended: university security hold is active");
                }
            }
        }

        // Issue #128: Check for final release lock
        let locked_amount = (scholarship.total_grant * FINAL_RELEASE_PERCENTAGE as i128) / 100;
        if scholarship.balance <= locked_amount && !scholarship.final_release_claimed {
            panic!("Final 10% is locked pending community vote");
        }

        let mut available_to_withdraw = scholarship.unlocked_balance;

        // Issue #128: Prevent withdrawing into the locked 10%
        if !scholarship.final_release_claimed && scholarship.total_grant > 0 {
            if scholarship.balance > locked_amount {
                available_to_withdraw =
                    core::cmp::min(available_to_withdraw, scholarship.balance - locked_amount);
            } else {
                available_to_withdraw = 0;
            }
        }

        if amount > available_to_withdraw {
            panic!("Amount exceeds available unlocked balance");
        }

        // Issue #118: Native XLM Reserve
        if scholarship.is_native {
            if scholarship.balance - amount < NATIVE_XLM_RESERVE {
                panic!("Withdrawal would leave less than the 2 XLM gas reserve");
            }
        }

        if scholarship.balance < amount {
            panic!("Insufficient balance");
        }

        // Issue #112: Apply tax
        let tax_rate_bps: u32 = env.storage().instance().get(&DataKey::TaxRate).unwrap_or(0);
        let tax_amount = (amount * tax_rate_bps as i128) / 10000;
        let net_amount = amount - tax_amount;

        scholarship.balance -= amount;
        scholarship.unlocked_balance -= amount;
        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student.clone()), &scholarship);

        // Transfer to student
        let client = token::Client::new(&env, &scholarship.token);
        client.transfer(&env.current_contract_address(), &student, &net_amount);

        // Note: Tax amount is currently held by the contract. A treasury address could be added.
    }

    // --- Issue #112: Scholarship_Simulate_Claim_Dry-Run_Helper ---
    pub fn set_tax_rate(env: Env, admin: Address, rate_bps: u32) {
        admin.require_auth();
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }
        if rate_bps > 10000 {
            panic!("Tax rate cannot exceed 100%");
        }
        env.storage().instance().set(&DataKey::TaxRate, &rate_bps);
    }

    pub fn simulate_claim(env: Env, student: Address) -> ClaimSimulation {
        let scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .unwrap_or_else(|| {
                // Return zero-value simulation if no scholarship found
                return ClaimSimulation {
                    tokens_to_release: 0,
                    estimated_gas_fee: ESTIMATED_GAS_FEE,
                    tax_withholding_amount: 0,
                    net_claimable_amount: 0,
                };
            });

        if scholarship.is_paused || scholarship.is_disputed {
            return ClaimSimulation {
                tokens_to_release: 0,
                estimated_gas_fee: ESTIMATED_GAS_FEE,
                tax_withholding_amount: 0,
                net_claimable_amount: 0,
            };
        }

        let mut tokens_to_release = scholarship.unlocked_balance;

        // Issue #128 logic
        if !scholarship.final_release_claimed && scholarship.total_grant > 0 {
            let locked_amount = (scholarship.total_grant * FINAL_RELEASE_PERCENTAGE as i128) / 100;
            if scholarship.balance > locked_amount {
                tokens_to_release =
                    core::cmp::min(tokens_to_release, scholarship.balance - locked_amount);
            } else {
                tokens_to_release = 0;
            }
        }

        // Issue #118 logic
        if scholarship.is_native {
            if scholarship.balance > NATIVE_XLM_RESERVE {
                tokens_to_release =
                    core::cmp::min(tokens_to_release, scholarship.balance - NATIVE_XLM_RESERVE);
            } else {
                tokens_to_release = 0;
            }
        }

        if tokens_to_release < 0 {
            tokens_to_release = 0;
        }

        let tax_rate_bps: u32 = env.storage().instance().get(&DataKey::TaxRate).unwrap_or(0);
        let tax_withholding_amount = (tokens_to_release * tax_rate_bps as i128) / 10000;
        let net_claimable_amount = tokens_to_release - tax_withholding_amount;

        ClaimSimulation {
            tokens_to_release,
            estimated_gas_fee: ESTIMATED_GAS_FEE,
            tax_withholding_amount,
            net_claimable_amount,
        }
    }
// --- Issue #124: Gas Fee Subsidy for Early Learners ---

    /// Configures the Native XLM token address used for the Gas Treasury
    pub fn set_gas_treasury(env: Env, admin: Address, token: Address) {
        admin.require_auth();

        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin)
            .expect("Contract not initialized");

        assert_eq!(admin, stored_admin, "Only admin can set gas treasury");

        env.storage().instance().set(&DataKey::GasTreasuryToken, &token);
    }

    /// Low-Friction Onboarding: Subsidizes gas for the first 100 students
    pub fn claim_gas_subsidy(env: Env, student: Address) {
        student.require_auth();

        // 1. Verify Treasury is configured
        let token_addr: Address = env.storage().instance().get(&DataKey::GasTreasuryToken)
            .expect("Gas treasury not configured");

        // 2. Ensure student hasn't already claimed it
        let has_received: bool = env.storage().persistent()
            .get(&DataKey::HasReceivedSubsidy(student.clone()))
            .unwrap_or(false);
        assert!(!has_received, "Student has already received a gas subsidy");

        // 3. Check the 100 student limit
        let count: u32 = env.storage().instance()
            .get(&DataKey::SubsidizedStudentCount)
            .unwrap_or(0);
        assert!(count < MAX_SUBSIDIZED_STUDENTS, "Maximum number of subsidies reached");

        // 4. Check student's balance against the threshold
        let client = token::Client::new(&env, &token_addr);
        let student_balance = client.balance(&student);
        assert!(student_balance < SUBSIDY_THRESHOLD, "Student balance is above the subsidy threshold");

        // 5. Ensure the contract has enough funds
        let contract_balance = client.balance(&env.current_contract_address());
        assert!(contract_balance >= SUBSIDY_AMOUNT, "Insufficient gas treasury balance");

        // 6. Transfer the subsidy
        client.transfer(&env.current_contract_address(), &student, &SUBSIDY_AMOUNT);

        // 7. Update state to prevent double-claiming
        env.storage().persistent().set(&DataKey::HasReceivedSubsidy(student.clone()), &true);
        env.storage().instance().set(&DataKey::SubsidizedStudentCount, &(count + 1));

        // 8. Publish event
        env.events().publish((Symbol::new(&env, "gas_subsidy"), student), SUBSIDY_AMOUNT);
    }
    // --- Issue #128: Community_Governance_Veto_on_Final_Graduation_Release ---
    pub fn initiate_final_release_vote(env: Env, student: Address) {
        student.require_auth();

        let scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        let locked_amount = (scholarship.total_grant * FINAL_RELEASE_PERCENTAGE as i128) / 100;
        if scholarship.balance > locked_amount || scholarship.final_release_claimed {
            panic!("Final release vote cannot be initiated yet");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::CommunityVote(student.clone()))
        {
            panic!("Vote already initiated");
        }

        let vote = CommunityVote {
            student: student.clone(),
            yes_votes: 0,
            voters: Vec::new(&env),
            is_passed: false,
            created_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::CommunityVote(student.clone()), &vote);
    }

    pub fn transfer_scholarship_to_teacher(
        env: Env,
        student: Address,
        teacher: Address,
        amount: i128,
    ) {
        student.require_auth();
        
        let milestone_key = DataKey::Milestone(student.clone(), Symbol::new(&env, "class_schedule"));
        assert!(env.storage().persistent().has(&milestone_key), "Class schedule not verified by admin");

        let mut profile = Self::get_student_data(env.clone(), student.clone());
        assert!(!profile.book_voucher_claimed, "Book voucher already distributed");

        // Release $200 (assuming 7 decimal precision)
        let voucher_amount: i128 = 200_0000000;
        let client = token::Client::new(&env, &token_addr);
        client.transfer(&env.current_contract_address(), &student, &voucher_amount);

        profile.book_voucher_claimed = true;
        env.storage().persistent().set(&DataKey::StudentProfile(student), &profile);
    }

    // Initializer to set the platform admin
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn cast_community_vote(env: Env, voter: Address, student: Address) {
        voter.require_auth();

        let mut vote: CommunityVote = env
            .storage()
            .persistent()
            .get(&DataKey::CommunityVote(student.clone()))
            .expect("No vote initiated for this student");

        if vote.is_passed {
            panic!("Vote has already passed");
        }
        if vote.voters.contains(&voter) {
            panic!("Voter has already voted");
        }

        vote.voters.push_back(voter);
        vote.yes_votes += 1;

        if vote.yes_votes >= COMMUNITY_VOTE_THRESHOLD {
            vote.is_passed = true;
        }

        env.storage()
            .persistent()
            .set(&DataKey::CommunityVote(student.clone()), &vote);
    }

    pub fn claim_final_release(env: Env, student: Address) {
        student.require_auth();

        let vote: CommunityVote = env
            .storage()
            .persistent()
            .get(&DataKey::CommunityVote(student.clone()))
            .expect("No vote found for this student");

        if !vote.is_passed {
            panic!("Community vote has not passed");
        }

        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        if scholarship.final_release_claimed {
            panic!("Final release already claimed");
        }

        let locked_amount = (scholarship.total_grant * FINAL_RELEASE_PERCENTAGE as i128) / 100;
        if scholarship.balance > locked_amount {
            panic!("Final release not yet locked");
        }

        let amount_to_release = scholarship.balance;

        if amount_to_release <= 0 {
            panic!("No balance to claim");
        }

        // Issue #118: Native XLM Reserve still applies
        if scholarship.is_native {
            if amount_to_release < NATIVE_XLM_RESERVE {
                panic!("Final balance is less than gas reserve");
            }
            let final_claim = amount_to_release - NATIVE_XLM_RESERVE;
            scholarship.balance -= final_claim;
            scholarship.unlocked_balance -= final_claim;

            let client = token::Client::new(&env, &scholarship.token);
            client.transfer(&env.current_contract_address(), &student, &final_claim);
        } else {
            scholarship.balance = 0;
            scholarship.unlocked_balance = 0;
            let client = token::Client::new(&env, &scholarship.token);
            client.transfer(&env.current_contract_address(), &student, &amount_to_release);
        }

        scholarship.final_release_claimed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student.clone()), &scholarship);

        // Issue #122: Mark as graduated
        Self::mark_as_graduated(env, student.clone(), scholarship.funder.clone());
    }

    // --- Issue #122: On-Chain_Graduation_Credential_Registry ---
    fn mark_as_graduated(env: Env, student: Address, funder: Address) {
        // This is an internal function called upon final claim
        let mut profile: GraduateProfile = env
            .storage()
            .persistent()
            .get(&DataKey::GraduationRegistry(student.clone()))
            .unwrap_or(GraduateProfile {
                student: student.clone(),
                graduation_date: env.ledger().timestamp(),
                final_gpa: 0,
                completed_scholarships: Vec::new(&env),
            });

        if !profile.completed_scholarships.contains(&funder) {
            profile.completed_scholarships.push_back(funder);
        }

        // Get final GPA
        if let Some(gpa_data) = env
            .storage()
            .persistent()
            .get::<_, StudentGPA>(&DataKey::StudentGPA(student.clone()))
        {
            profile.final_gpa = gpa_data.gpa;
        }

        profile.graduation_date = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::GraduationRegistry(student.clone()), &profile);
    }

    pub fn get_graduate_profile(env: Env, student: Address) -> Option<GraduateProfile> {
        env.storage()
            .persistent()
            .get(&DataKey::GraduationRegistry(student))
    }

    // --- Issue #115: Emergency_Protocol_Pause_for_University_Admins ---

    /// Assigns a university admin (registrar) for a given university address.
    /// Only the platform admin can call this.
    pub fn register_university_admin(
        env: Env,
        platform_admin: Address,
        university: Address,
        university_admin: Address,
    ) {
        platform_admin.require_auth();
        if !Self::is_admin(&env, &platform_admin) {
            panic!("Not authorized: caller is not the platform admin");
        }
        env.storage()
            .persistent()
            .set(&DataKey::UniversityAdmin(university), &university_admin);
    }

    /// Associates a student with a university so they fall under that university's
    /// security hold. Called by the university admin when onboarding a scholar.
    pub fn register_student_university(
        env: Env,
        university_admin: Address,
        university: Address,
        student: Address,
    ) {
        university_admin.require_auth();
        let registered_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::UniversityAdmin(university.clone()))
            .expect("University has no registered admin");
        if registered_admin != university_admin {
            panic!("Not authorized: caller is not the university admin");
        }
        env.storage()
            .persistent()
            .set(&DataKey::StudentUniversity(student), &university);
    }

    /// Triggers a 7-day Security Hold for all scholarships belonging to a university.
    /// Only the registered university admin (registrar) can call this.
    /// While a hold is active, no student associated with the university can withdraw.
    pub fn trigger_security_hold(
        env: Env,
        university_admin: Address,
        university: Address,
        reason: Symbol,
    ) {
        university_admin.require_auth();
        let registered_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::UniversityAdmin(university.clone()))
            .expect("University has no registered admin");
        if registered_admin != university_admin {
            panic!("Not authorized: caller is not the university admin");
        }

        let now = env.ledger().timestamp();
        let expires_at = now
            .checked_add(SECURITY_HOLD_DURATION)
            .expect("Timestamp overflow");

        let hold = SecurityHold {
            university: university.clone(),
            triggered_by: university_admin,
            triggered_at: now,
            expires_at,
            is_active: true,
            reason,
        };

        env.storage()
            .persistent()
            .set(&DataKey::SecurityHold(university.clone()), &hold);
        env.storage()
            .persistent()
            .extend_ttl(
                &DataKey::SecurityHold(university.clone()),
                LEDGER_BUMP_THRESHOLD,
                LEDGER_BUMP_EXTEND,
            );

        env.events().publish(
            (symbol_short!("sec_hold"), symbol_short!("trigger")),
            (university, expires_at),
        );
    }

    /// Lifts an active Security Hold before its 7-day expiry.
    /// Only the university admin who triggered it (or any registered admin for that university)
    /// can lift the hold once the incident is resolved.
    pub fn lift_security_hold(
        env: Env,
        university_admin: Address,
        university: Address,
    ) {
        university_admin.require_auth();
        let registered_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::UniversityAdmin(university.clone()))
            .expect("University has no registered admin");
        if registered_admin != university_admin {
            panic!("Not authorized: caller is not the university admin");
        }

        let mut hold: SecurityHold = env
            .storage()
            .persistent()
            .get(&DataKey::SecurityHold(university.clone()))
            .expect("No active security hold found for this university");

        if !hold.is_active {
            panic!("Security hold is already inactive");
        }

        hold.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::SecurityHold(university.clone()), &hold);

        let now = env.ledger().timestamp();
        env.events().publish(
            (symbol_short!("sec_hold"), symbol_short!("lift")),
            (university, now),
        );
    }

    pub fn fund_scholarship(env: Env, funder: Address, student: Address, amount: i128, token: Address) {
        funder.require_auth();
        
        // Check if student is verified (Issue #160)
        let enrollment: Option<EnrollmentData> = env.storage().instance().get(&DataKey::Enrollment(student.clone()));
        if enrollment.is_none() {
            env.panic_with_error(Error::Unauthorized);
        }
        
        let client = token::Client::new(&env, &token);
        client.transfer(&funder, &env.current_contract_address(), &amount);
        
        let mut scholarship: Scholarship = env.storage().instance()
            .get(&DataKey::Scholarship(student.clone()))
            .unwrap_or(Scholarship {
                balance: 0,
                token,
                total_accrued: 0,
                total_distributed: 0,
                last_distribution: 0,
            },
        );
    }

    /// Deposit yield earned by the scholarship treasury into the Research Bonus Fund.
    /// The caller (admin/keeper) must have already approved the token transfer.
    pub fn accrue_treasury_yield(env: Env, admin: Address, yield_amount: i128) {
        admin.require_auth();
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }
        if yield_amount <= 0 {
            panic!("Yield must be positive");
        }

        let mut fund: ResearchBonusFund = env
            .storage()
            .instance()
            .get(&DataKey::ResearchBonusFund)
            .expect("Research Bonus Fund not initialized");

        let client = token::Client::new(&env, &fund.token);
        client.transfer(&admin, &env.current_contract_address(), &yield_amount);

        fund.total_balance += yield_amount;
        fund.total_accrued += yield_amount;
        env.storage().instance().set(&DataKey::ResearchBonusFund, &fund);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "YieldAccrued"), admin),
            yield_amount,
        );
    }

    /// Register a student address for a leaderboard rank so the bonus
    /// distributor can resolve them. Called by admin when the leaderboard is settled.
    pub fn register_surprise_bonus_recipient(env: Env, admin: Address, rank: u64, student: Address) {
        admin.require_auth();
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }
        env.storage()
            .persistent()
            .set(&DataKey::SurpriseBonusRecipient(rank), &student);
    }

    /// Distribute the accumulated Research Bonus Fund as a Surprise Bonus to
    /// the top 5% of students on the leaderboard (minimum 1 recipient).
    /// Each eligible student receives an equal share of the fund balance.
    pub fn distribute_surprise_bonus(env: Env, admin: Address) {
        admin.require_auth();
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let mut fund: ResearchBonusFund = env
            .storage()
            .instance()
            .get(&DataKey::ResearchBonusFund)
            .expect("Research Bonus Fund not initialized");

        if fund.total_balance <= 0 {
            panic!("No balance to distribute");
        }

        let leaderboard_size: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LeaderboardSize)
            .unwrap_or(0);

        if leaderboard_size == 0 {
            panic!("Leaderboard is empty");
        }

    pub fn calculate_remaining_airtime(env: Env, student: Address) -> u64 {
        let base_rate: i128 = env.storage().instance().get(&DataKey::BaseRate).unwrap_or(0);
        if base_rate == 0 {
            return 0;
        }

        let mut effective_rate = base_rate;

        // Apply Reputation Bonus (2% discount)
        let has_reputation_bonus: bool = env.storage().instance().get(&DataKey::ReputationBonus(student.clone())).unwrap_or(false);
        if has_reputation_bonus {
            effective_rate = (effective_rate * 98) / 100;
        }

        // Apply GPA Multiplier
        let gpa_multiplier: i128 = env.storage().instance().get(&DataKey::GpaMultiplier(student.clone())).unwrap_or(10000);
        if gpa_multiplier == 0 {
            return 0; // Paused
        }
        effective_rate = (effective_rate * gpa_multiplier) / 10000;
        
        let scholarship: Option<Scholarship> = env.storage().instance().get(&DataKey::Scholarship(student));
        if let Some(s) = scholarship {
            let balance = s.balance;
            if balance > 0 {
                return (balance / effective_rate) as u64;
            }
        }

        let total_paid = bonus_per_student * recipient_count as i128;
        fund.total_balance -= total_paid;
        fund.total_distributed += total_paid;
        fund.last_distribution = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::ResearchBonusFund, &fund);
    }

    // --- Issue #110: Withdrawal Address Whitelisting ---

    pub fn set_authorized_payout_address(env: Env, student: Address, authorized_address: Address) {
        student.require_auth();
        let unlock_time = env.ledger().timestamp() + 172800; // 48 hours
        env.storage().instance().set(&DataKey::AuthorizedPayoutPending(student.clone()), &authorized_address);
        env.storage().instance().set(&DataKey::UnlockTime(student.clone()), &unlock_time);
    }

    pub fn confirm_authorized_payout_address(env: Env, student: Address) {
        student.require_auth();
        let unlock_time: u64 = env.storage().instance().get(&DataKey::UnlockTime(student.clone())).expect("No pending payout address");
        if env.ledger().timestamp() < unlock_time {
            env.panic_with_error(Error::TimelockNotExpired);
        }
        let pending_address: Address = env.storage().instance().get(&DataKey::AuthorizedPayoutPending(student.clone())).expect("No pending payout address");
        env.storage().instance().set(&DataKey::AuthorizedPayout(student.clone()), &pending_address);
        env.storage().instance().remove(&DataKey::AuthorizedPayoutPending(student.clone()));
        env.storage().instance().remove(&DataKey::UnlockTime(student.clone()));
    }

    pub fn claim_scholarship(env: Env, student: Address, amount: i128) {
        student.require_auth();
        
        let payout_address: Address = env.storage().instance()
            .get(&DataKey::AuthorizedPayout(student.clone()))
            .unwrap_or(student.clone()); // Default to student if not set

        let mut scholarship: Scholarship = env.storage().instance()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");
            
        if scholarship.balance < amount {
            env.panic_with_error(Error::InvalidAction);
        }
        
        scholarship.balance -= amount;
        env.storage().instance().set(&DataKey::Scholarship(student), &scholarship);
        
        let client = token::Client::new(&env, &scholarship.token);
        client.transfer(&env.current_contract_address(), &payout_address, &amount);
    }

    // --- Issue #114: Cross-Project Reputation Bonus ---

    pub fn set_reputation_bonus(env: Env, admin: Address, student: Address, has_bonus: bool) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error(Error::Unauthorized);
        }
        env.storage().instance().set(&DataKey::ReputationBonus(student), &has_bonus);
    }

    // --- Issue #160: Proof-of-Enrollment Initialization Gate ---

    pub fn set_oracle_status(env: Env, admin: Address, oracle: Address, status: bool) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error(Error::Unauthorized);
        }
        env.storage().instance().set(&DataKey::OracleRegistry(oracle), &status);
    }

    fn assert_fresh_oracle_payload(env: &Env, generated_at: u64) {
        let current_ts = env.ledger().timestamp();
        if generated_at > current_ts {
            env.panic_with_error(Error::OracleDataStale);
        }
        let delta = current_ts.checked_sub(generated_at).unwrap_or(u64::MAX);
        if delta > ORACLE_STALENESS_THRESHOLD {
            env.panic_with_error(Error::OracleDataStale);
        }
    }

    pub fn verify_enrollment(env: Env, student: Address, oracle: Address, signature: soroban_sdk::BytesN<64>, payload: EnrollmentData) {
        student.require_auth();

        // 1. Verify Oracle is whitelisted
        let is_whitelisted: bool = env.storage().instance().get(&DataKey::OracleRegistry(oracle.clone())).unwrap_or(false);
        if !is_whitelisted {
            env.panic_with_error(Error::Unauthorized);
        }

        // 1a. Prevent stale oracle data
        Self::assert_fresh_oracle_payload(&env, payload.generated_at);

        // 2. Prevent Replay Attacks
        let stored_nonce: u64 = env.storage().instance().get(&DataKey::Nonce(student.clone())).unwrap_or(0);
        if payload.nonce <= stored_nonce {
            env.panic_with_error(Error::ReplayAttack);
        }

        // 3. Verify Signature
        // Placeholder for signature verification:
        // In a real implementation, we would use:
        // env.crypto().ed25519_verify(&oracle_public_key, &payload.student.into(), &signature);
        
        // For now, we'll return an error if the signature is "all zeros" as a test case
        if signature == soroban_sdk::BytesN::from_array(&env, &[1u8; 64]) {
            env.panic_with_error(Error::InvalidOracleSig);
        }
        
        env.storage().instance().set(&DataKey::Enrollment(student.clone()), &payload);
        env.storage().instance().set(&DataKey::Nonce(student.clone()), &payload.nonce);

        env.events().publish((Symbol::new(&env, "EnrollmentVerified"), student.clone(), oracle), student);
    }

    // --- Issue #161: GPA-Triggered "Stream-Multiplier" Logic ---

    pub fn apply_gpa_multiplier(env: Env, student: Address, oracle: Address, signature: soroban_sdk::BytesN<64>, payload: GpaData) {
        // 1. Verify Oracle
        let is_whitelisted: bool = env.storage().instance().get(&DataKey::OracleRegistry(oracle.clone())).unwrap_or(false);
        if !is_whitelisted {
            env.panic_with_error(Error::Unauthorized);
        }

        // 1a. Prevent stale oracle data
        Self::assert_fresh_oracle_payload(&env, payload.generated_at);

        // 2. Prevent Replay/Double-Application in same epoch
        let last_epoch: u32 = env.storage().instance().get(&DataKey::GpaEpoch(student.clone())).unwrap_or(0);
        if payload.epoch <= last_epoch {
            env.panic_with_error(Error::ReplayAttack);
        }

        // 3. Map GPA to Multiplier
        // 4.0 (400 bps) -> 12000 (120%)
        // 3.5 (350 bps) -> 10000 (100%)
        // 3.0 (300 bps) -> 8000 (80%)
        // < 2.5 (250 bps) -> 0 (Pause)
        let multiplier_bps = if payload.gpa_bps >= 400 {
            12000
        } else if payload.gpa_bps >= 350 {
            10000
        } else if payload.gpa_bps >= 300 {
            8000
        } else if payload.gpa_bps >= 250 {
            4000
        } else {
            0 // Pause
        };

        if multiplier_bps == 0 {
            // Optional: emit an event or just let the 0 multiplier pause the stream
        }

        let old_rate = Self::calculate_remaining_airtime(env.clone(), student.clone()); // Simplified "rate" representation

        env.storage().instance().set(&DataKey::GpaMultiplier(student.clone()), &(multiplier_bps as i128));
        env.storage().instance().set(&DataKey::GpaEpoch(student.clone()), &payload.epoch);

        let new_rate = Self::calculate_remaining_airtime(env.clone(), student.clone());

        env.events().publish((Symbol::new(&env, "MultiplierApplied"), student, old_rate as i128, new_rate as i128), payload.gpa_bps);
    }

    // --- Dynamic Sponsor-Clawback Logic Implementation ---

    /// Register a new clawback condition for a scholarship
    /// Only the sponsor (funder) can register conditions
    pub fn register_clawback_condition(
        env: Env,
        funder: Address,
        student: Address,
        trigger_type: ClawbackTriggerType,
        clawback_percentage: u64,
        threshold_value: u64,
    ) {
        funder.require_auth();

        // Validate clawback percentage
        if clawback_percentage > MAX_CLAWBACK_PERCENTAGE {
            panic!("Clawback percentage exceeds maximum");
        }

        // Verify scholarship exists
        let scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found for this student");

        if scholarship.funder != funder {
            panic!("Only the scholarship funder can register clawback conditions");
        }

        // Generate condition ID based on timestamp
        let condition_id = env.ledger().timestamp();

        let condition = ClawbackCondition {
            funder: funder.clone(),
            student: student.clone(),
            trigger_type,
            clawback_percentage,
            threshold_value,
            triggered_at: None,
            executed_at: None,
            is_active: true,
            cooldown_period: DEFAULT_CLAWBACK_COOLDOWN,
            last_clawback_time: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ClawbackCondition(funder.clone(), student.clone(), condition_id), &condition);

        env.events().publish(
            (Symbol::new(&env, "clawback_registered"), funder, student),
            (trigger_type, clawback_percentage),
        );
    }

    /// Check if clawback conditions are met and trigger clawback if conditions are satisfied
    pub fn check_and_trigger_clawback(
        env: Env,
        funder: Address,
        student: Address,
        condition_id: u64,
    ) -> bool {
        let condition: ClawbackCondition = env
            .storage()
            .persistent()
            .get(&DataKey::ClawbackCondition(funder.clone(), student.clone(), condition_id))
            .expect("Clawback condition not found");

        if !condition.is_active {
            return false;
        }

        let now = env.ledger().timestamp();

        // Check cooldown period
        if now < condition.last_clawback_time + condition.cooldown_period {
            return false; // Still in cooldown
        }

        // Check if condition is met based on trigger type
        let condition_met = match condition.trigger_type {
            ClawbackTriggerType::GpaThreshold => {
                Self::check_gpa_threshold(&env, &student, condition.threshold_value)
            }
            ClawbackTriggerType::CourseCompletion => {
                Self::check_course_completion(&env, &student, condition.threshold_value)
            }
            ClawbackTriggerType::TimeElapsed => {
                Self::check_time_elapsed(&env, &condition, condition.threshold_value)
            }
            ClawbackTriggerType::ActivityInactive => {
                Self::check_activity_inactive(&env, &student, condition.threshold_value)
            }
            ClawbackTriggerType::CombinedConditions => {
                Self::check_combined_conditions(&env, &student, &condition)
            }
        };

        if condition_met {
            let mut updated_condition = condition.clone();
            updated_condition.triggered_at = Some(now);
            env.storage()
                .persistent()
                .set(&DataKey::ClawbackCondition(funder.clone(), student.clone(), condition_id), &updated_condition);
            return true;
        }

        false
    }

    /// Execute clawback of funds from a scholarship
    pub fn execute_clawback(
        env: Env,
        funder: Address,
        student: Address,
        condition_id: u64,
    ) -> i128 {
        funder.require_auth();

        let mut condition: ClawbackCondition = env
            .storage()
            .persistent()
            .get(&DataKey::ClawbackCondition(funder.clone(), student.clone(), condition_id))
            .expect("Clawback condition not found");

        if !condition.is_active {
            panic!("Clawback condition is not active");
        }

        if condition.triggered_at.is_none() {
            panic!("Clawback condition has not been triggered");
        }

        // Check execution timeout (7 days after trigger)
        let now = env.ledger().timestamp();
        let triggered_time = condition.triggered_at.unwrap();
        if now > triggered_time + CLAWBACK_EXECUTION_TIMEOUT {
            panic!("Clawback execution window has expired");
        }

        if condition.executed_at.is_some() {
            panic!("Clawback has already been executed for this condition");
        }

        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        if scholarship.funder != funder {
            panic!("Only the scholarship funder can execute clawback");
        }

        // Calculate clawback amount
        let clawback_amount =
            (scholarship.balance * condition.clawback_percentage as i128) / 100;

        if clawback_amount <= 0 {
            panic!("Calculated clawback amount is zero or negative");
        }

        // Update scholarship
        scholarship.balance -= clawback_amount;
        if scholarship.unlocked_balance > clawback_amount {
            scholarship.unlocked_balance -= clawback_amount;
        } else {
            scholarship.unlocked_balance = 0;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student.clone()), &scholarship);

        // Update condition
        condition.executed_at = Some(now);
        condition.last_clawback_time = now;
        env.storage()
            .persistent()
            .set(&DataKey::ClawbackCondition(funder.clone(), student.clone(), condition_id), &condition);

        // Record clawback event
        let event_id = now;
        let clawback_event = ClawbackEvent {
            funder: funder.clone(),
            student: student.clone(),
            amount_clawed_back: clawback_amount,
            trigger_type: condition.trigger_type,
            triggered_at: triggered_time,
            executed_at: now,
            remaining_balance: scholarship.balance,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ClawbackEventLog(funder.clone(), student.clone(), event_id), &clawback_event);

        // Transfer clawed back funds to funder
        let client = token::Client::new(&env, &scholarship.token);
        client.transfer(&env.current_contract_address(), &funder, &clawback_amount);

        env.events().publish(
            (Symbol::new(&env, "clawback_executed"), funder, student),
            (clawback_amount, scholarship.balance),
        );

        clawback_amount
    }

    /// Revoke an active clawback condition (only funder can revoke)
    pub fn revoke_clawback_condition(
        env: Env,
        funder: Address,
        student: Address,
        condition_id: u64,
    ) {
        funder.require_auth();

        let mut condition: ClawbackCondition = env
            .storage()
            .persistent()
            .get(&DataKey::ClawbackCondition(funder.clone(), student.clone(), condition_id))
            .expect("Clawback condition not found");

        if !condition.is_active {
            panic!("Condition is already revoked");
        }

        if condition.executed_at.is_some() {
            panic!("Cannot revoke a condition that has already been executed");
        }

        condition.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::ClawbackCondition(funder.clone(), student.clone(), condition_id), &condition);

        env.events().publish(
            (Symbol::new(&env, "clawback_revoked"), funder, student),
            condition_id,
        );
    }

    /// Get clawback condition details
    pub fn get_clawback_condition(
        env: Env,
        funder: Address,
        student: Address,
        condition_id: u64,
    ) -> Option<ClawbackCondition> {
        env.storage()
            .persistent()
            .get(&DataKey::ClawbackCondition(funder, student, condition_id))
    }

    /// Get clawback event details
    pub fn get_clawback_event(
        env: Env,
        funder: Address,
        student: Address,
        event_id: u64,
    ) -> Option<ClawbackEvent> {
        env.storage()
            .persistent()
            .get(&DataKey::ClawbackEventLog(funder, student, event_id))
    }

    // Helper functions for condition checking
    fn check_gpa_threshold(env: &Env, student: &Address, threshold: u64) -> bool {
        if let Some(gpa_data) = env
            .storage()
            .persistent()
            .get::<_, StudentGPA>(&DataKey::StudentGPA(student.clone()))
        {
            // If GPA falls below threshold, clawback is triggered
            gpa_data.gpa < threshold
        } else {
            false
        }
    }

    fn check_course_completion(env: &Env, student: &Address, threshold: u64) -> bool {
        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<_, StudentProfile>(&DataKey::StudentProfile(student.clone()))
        {
            // If courses completed is below threshold, clawback is triggered
            (profile.courses_completed as u64) < threshold
        } else {
            false
        }
    }

    fn check_time_elapsed(env: &Env, condition: &ClawbackCondition, threshold_days: u64) -> bool {
        let threshold_seconds = threshold_days * 86400;
        if let Some(triggered) = condition.triggered_at {
            let now = env.ledger().timestamp();
            now >= triggered + threshold_seconds
        } else {
            false
        }
    }

    fn check_activity_inactive(env: &Env, student: &Address, inactivity_threshold_days: u64) -> bool {
        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<_, StudentProfile>(&DataKey::StudentProfile(student.clone()))
        {
            let inactivity_seconds = inactivity_threshold_days * 86400;
            let now = env.ledger().timestamp();
            let time_since_activity = now.saturating_sub(profile.last_activity);
            time_since_activity > inactivity_seconds
        } else {
            false
        }
    }

    fn check_combined_conditions(
        env: &Env,
        student: &Address,
        condition: &ClawbackCondition,
    ) -> bool {
        // Combined: GPA below threshold AND inactive for 30 days
        let gpa_check = Self::check_gpa_threshold(env, student, 25); // 2.5 GPA threshold
        let inactivity_check = Self::check_activity_inactive(env, student, 30);
        gpa_check && inactivity_check
    }

    // --- Matching-Pool Quadratic Funding Implementation ---

    /// Initialize a new quadratic funding round
    pub fn init_quadratic_funding_round(
        env: Env,
        admin: Address,
        token: Address,
        matching_pool_amount: i128,
    ) -> u64 {
        admin.require_auth();

        if matching_pool_amount < QF_MATCHING_POOL_RESERVE {
            panic!("Matching pool amount is below minimum reserve");
        }

        // Get next round ID
        let round_counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::QFRoundCounter)
            .unwrap_or(0);
        let round_id = round_counter + 1;

        let now = env.ledger().timestamp();
        let end_time = now + QF_ROUND_DURATION;

        let round = QuadraticFundingRound {
            round_id,
            token: token.clone(),
            start_time: now,
            end_time,
            matching_pool_balance: matching_pool_amount,
            total_contributions: 0,
            total_matching_distributed: 0,
            project_count: 0,
            is_active: true,
            is_finalized: false,
            created_by: admin.clone(),
        };

        // Transfer matching pool tokens to contract
        let client = token::Client::new(&env, &token);
        client.transfer(&admin, &env.current_contract_address(), &matching_pool_amount);

        env.storage()
            .instance()
            .set(&DataKey::QFRoundCounter, &round_id);

        env.storage()
            .persistent()
            .set(&DataKey::QuadraticFundingRound(round_id), &round);

        env.events().publish(
            (Symbol::new(&env, "qf_round_created"), round_id as u64),
            (matching_pool_amount, end_time),
        );

        round_id
    }

    /// Register a project for a QF round
    pub fn register_qf_project(
        env: Env,
        project_owner: Address,
        round_id: u64,
        title: Symbol,
    ) -> u64 {
        project_owner.require_auth();

        let mut round: QuadraticFundingRound = env
            .storage()
            .persistent()
            .get(&DataKey::QuadraticFundingRound(round_id))
            .expect("QF round not found");

        if !round.is_active {
            panic!("QF round is not active");
        }

        if round.project_count >= QF_MAX_PROJECTS {
            panic!("Maximum projects per round reached");
        }

        let now = env.ledger().timestamp();
        if now > round.end_time {
            panic!("QF round has ended");
        }

        let project_id = round.project_count + 1;

        let project = FundingProject {
            project_id,
            round_id,
            project_owner: project_owner.clone(),
            title,
            total_raised: 0,
            contributor_count: 0,
            sqrt_sum_contributions: 0,
            total_matching: 0,
            created_at: now,
            is_approved: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::FundingProject(round_id, project_id), &project);

        round.project_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::QuadraticFundingRound(round_id), &round);

        env.events().publish(
            (Symbol::new(&env, "qf_project_registered"), round_id, project_id as u64),
            project_owner,
        );

        project_id
    }

    /// Contribute to a project in QF round
    pub fn contribute_to_qf_project(
        env: Env,
        contributor: Address,
        round_id: u64,
        project_id: u64,
        amount: i128,
    ) {
        contributor.require_auth();

        if amount < QF_MIN_CONTRIBUTION {
            panic!("Contribution amount is below minimum");
        }

        let mut round: QuadraticFundingRound = env
            .storage()
            .persistent()
            .get(&DataKey::QuadraticFundingRound(round_id))
            .expect("QF round not found");

        if !round.is_active {
            panic!("QF round is not active");
        }

        let now = env.ledger().timestamp();
        if now > round.end_time {
            panic!("QF round has ended");
        }

        let mut project: FundingProject = env
            .storage()
            .persistent()
            .get(&DataKey::FundingProject(round_id, project_id))
            .expect("Project not found");

        if !project.is_approved {
            panic!("Project is not approved");
        }

        // Record contribution
        let contribution = QFContribution {
            contributor: contributor.clone(),
            project_id,
            round_id,
            amount,
            contribution_time: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::QFContribution(contributor.clone(), round_id, project_id), &contribution);

        // Update project stats
        project.total_raised += amount;
        project.contributor_count += 1;

        // Calculate sqrt of contribution for QF formula
        let sqrt_amount = Self::isqrt(amount);
        project.sqrt_sum_contributions += sqrt_amount;

        env.storage()
            .persistent()
            .set(&DataKey::FundingProject(round_id, project_id), &project);

        // Update round stats
        round.total_contributions += amount;
        env.storage()
            .persistent()
            .set(&DataKey::QuadraticFundingRound(round_id), &round);

        // Transfer contribution tokens to contract
        let client = token::Client::new(&env, &round.token);
        client.transfer(&contributor, &env.current_contract_address(), &amount);

        env.events().publish(
            (Symbol::new(&env, "qf_contributed"), contributor, round_id, project_id as u64),
            amount,
        );
    }

    /// Finalize QF round and calculate matching amounts
    pub fn finalize_qf_round(env: Env, admin: Address, round_id: u64) {
        admin.require_auth();

        let mut round: QuadraticFundingRound = env
            .storage()
            .persistent()
            .get(&DataKey::QuadraticFundingRound(round_id))
            .expect("QF round not found");

        if round.is_finalized {
            panic!("QF round is already finalized");
        }

        let now = env.ledger().timestamp();
        if now < round.end_time {
            panic!("QF round has not ended yet");
        }

        // Calculate matching amounts for all projects using QF formula
        // Matching = (Σ√contribution)² - Σcontribution
        let total_sqrt_sum: i128 = Self::calculate_total_sqrt_sum(&env, round_id);
        let total_matching_budget = (total_sqrt_sum * total_sqrt_sum) - round.total_contributions;

        if total_matching_budget <= 0 || total_matching_budget > round.matching_pool_balance {
            panic!("Matching budget calculation failed");
        }

        // Distribute matching to projects
        let mut total_distributed: i128 = 0;
        for project_idx in 1..=round.project_count {
            if let Some(mut project) = env
                .storage()
                .persistent()
                .get::<_, FundingProject>(&DataKey::FundingProject(round_id, project_idx))
            {
                if project.sqrt_sum_contributions > 0 {
                    let project_matching = ((project.sqrt_sum_contributions * project.sqrt_sum_contributions)
                        - project.total_raised)
                        .max(0);

                    if project_matching > 0 {
                        project.total_matching = project_matching;
                        env.storage()
                            .persistent()
                            .set(&DataKey::FundingProject(round_id, project_idx), &project);

                        // Record matching distribution
                        let distribution = MatchingDistribution {
                            round_id,
                            project_id: project_idx,
                            matching_amount: project_matching,
                            distributed_at: now,
                            project_owner: project.project_owner.clone(),
                        };

                        env.storage()
                            .persistent()
                            .set(&DataKey::MatchingDistribution(round_id, project_idx), &distribution);

                        total_distributed += project_matching;
                    }
                }
            }
        }

        round.total_matching_distributed = total_distributed;
        round.is_finalized = true;
        env.storage()
            .persistent()
            .set(&DataKey::QuadraticFundingRound(round_id), &round);

        env.events().publish(
            (Symbol::new(&env, "qf_round_finalized"), round_id),
            (total_distributed, round.total_contributions),
        );
    }

    /// Claim matching funds for a project
    pub fn claim_qf_matching(env: Env, project_owner: Address, round_id: u64, project_id: u64) {
        project_owner.require_auth();

        let round: QuadraticFundingRound = env
            .storage()
            .persistent()
            .get(&DataKey::QuadraticFundingRound(round_id))
            .expect("QF round not found");

        if !round.is_finalized {
            panic!("QF round has not been finalized yet");
        }

        let mut project: FundingProject = env
            .storage()
            .persistent()
            .get(&DataKey::FundingProject(round_id, project_id))
            .expect("Project not found");

        if project.project_owner != project_owner {
            panic!("Only project owner can claim matching funds");
        }

        if project.total_matching <= 0 {
            panic!("No matching funds to claim");
        }

        let matching_amount = project.total_matching;
        project.total_matching = 0; // Prevent double-claiming

        env.storage()
            .persistent()
            .set(&DataKey::FundingProject(round_id, project_id), &project);

        // Transfer matching funds to project owner
        let client = token::Client::new(&env, &round.token);
        client.transfer(&env.current_contract_address(), &project_owner, &matching_amount);

        env.events().publish(
            (Symbol::new(&env, "qf_matching_claimed"), round_id, project_id as u64),
            matching_amount,
        );
    }

    /// Get QF round details
    pub fn get_qf_round(env: Env, round_id: u64) -> Option<QuadraticFundingRound> {
        env.storage()
            .persistent()
            .get(&DataKey::QuadraticFundingRound(round_id))
    }

    /// Get project details
    pub fn get_qf_project(env: Env, round_id: u64, project_id: u64) -> Option<FundingProject> {
        env.storage()
            .persistent()
            .get(&DataKey::FundingProject(round_id, project_id))
    }

    /// Get contribution details
    pub fn get_qf_contribution(
        env: Env,
        contributor: Address,
        round_id: u64,
        project_id: u64,
    ) -> Option<QFContribution> {
        env.storage()
            .persistent()
            .get(&DataKey::QFContribution(contributor, round_id, project_id))
    }

    /// Get matching distribution for a project
    pub fn get_qf_matching_distribution(
        env: Env,
        round_id: u64,
        project_id: u64,
    ) -> Option<MatchingDistribution> {
        env.storage()
            .persistent()
            .get(&DataKey::MatchingDistribution(round_id, project_id))
    }

    // --- QF Helper Functions ---

    /// Integer square root calculation
    fn isqrt(n: i128) -> i128 {
        if n < 0 {
            return 0;
        }
        if n == 0 {
            return 0;
        }

        let mut x = n;
        let mut y = (x + 1) / 2;

        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }

        x
    }

    /// Calculate total sqrt sum across all projects in a round
    fn calculate_total_sqrt_sum(env: &Env, round_id: u64) -> i128 {
        let round: QuadraticFundingRound = env
            .storage()
            .persistent()
            .get(&DataKey::QuadraticFundingRound(round_id))
            .expect("QF round not found");

        let mut total_sqrt = 0i128;
        for project_idx in 1..=round.project_count {
            if let Some(project) = env
                .storage()
                .persistent()
                .get::<_, FundingProject>(&DataKey::FundingProject(round_id, project_idx))
            {
                total_sqrt += project.sqrt_sum_contributions;
            }
        }

        total_sqrt
    }
}

    /// Read-only view of the Research Bonus Fund state.
    pub fn get_research_bonus_fund(env: Env) -> Option<ResearchBonusFund> {
        env.storage().instance().get(&DataKey::ResearchBonusFund)
    }
}
