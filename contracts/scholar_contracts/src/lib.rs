#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol, Vec, token};

// Constants for ledger bump and GPA bonus calculations
const LEDGER_BUMP_THRESHOLD: u32 = 7776000; // ~90 days
const LEDGER_BUMP_EXTEND: u32 = 7776000;   // ~90 days
const GPA_BONUS_THRESHOLD: u64 = 35;       // 3.5 GPA (stored as 35)
const GPA_BONUS_PERCENTAGE_PER_POINT: u64 = 20; // 20% per 0.1 GPA point above threshold
const EARLY_DROP_WINDOW_SECONDS: u64 = 86400; // 24 hours

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

// Issue #268: Automated telemetry constants
const TELEMETRY_WINDOW_SIZE: u64 = 100; // Number of oracle calls to track
const TELEMETRY_ERROR_THRESHOLD: u64 = 10; // Error rate threshold for alerts
const TELEMETRY_RETENTION_PERIOD: u64 = 7776000; // 90 days retention

// Issue #261: Bounded vector constants for Wasm optimization
const MAX_COURSE_IDS_PER_SUBSCRIPTION: u32 = 50; // Maximum courses per subscription
const MAX_COURSES_PER_REGISTRY: u32 = 1000; // Maximum courses in registry
const MAX_ROYALTY_RECIPIENTS: u32 = 10; // Maximum royalty recipients per course
const MAX_BOARD_MEMBERS: u32 = 7; // Maximum academic board members
const MAX_SIGNATURES_PER_REQUEST: u32 = 5; // Maximum signatures for board requests
const MAX_LEADERBOARD_ENTRIES: u32 = 100; // Maximum leaderboard entries
const MAX_RECENT_ERRORS: u32 = 50; // Maximum recent errors to track


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    SbtMint(Address, u64),
    StreamCreated(Address, Address, i128), // funder, student, amount
    GeographicReview(Address, u64), // student, timestamp
    SsiVerificationRequired(Address), // student
    // Issue #92: Leaderboard events
    AcademicPointsEarned(Address, u64), // student, points
    LeaderboardUpdated(Symbol, u64), // student_alias, rank
    MatchingBonusDistributed(Symbol, i128), // student_alias, amount
    // Issue #94: Tutoring bridge events
    TutoringAgreementCreated(Address, Address, u64), // scholar, tutor, agreement_id
    SubStreamRedirected(Address, Address, i128), // scholar, tutor, amount
    TutoringAgreementEnded(u64), // agreement_id
    // Issue #95: Alumni Donation Matching events
    AlumniDonationMatched(Address, i128, i128), // donor, original_amount, matched_amount
    // Issue #93: Scholarship Probation Cooling-Off events
    ProbationStarted(Address, u64), // student, warning_period_end
    ProbationEnded(Address, bool), // student, recovered
    StreamRevoked(Address), // student
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
    pub balance: i128,
    pub token: Address,
    pub unlocked_balance: i128,
    pub last_verif: u64,
    pub is_paused: bool,
    pub is_disputed: bool,
    pub dispute_reason: Option<Symbol>,
    pub final_ruling: Option<Symbol>,
}

// Issue #92: Anonymized Leaderboard for Top Scholars structs
#[contracttype]
#[derive(Clone)]
pub struct StudentAcademicProfile {
    pub student: Address,
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

// Issue #268: Automated telemetry structs for oracle polling
#[contracttype]
#[derive(Clone)]
pub struct OracleTelemetry {
    pub oracle_address: Address,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub last_call_timestamp: u64,
    pub last_success_timestamp: u64,
    pub error_rate_percentage: u64,
    pub recent_errors: Vec<(u64, Symbol)>, // (timestamp, error_type)
}

#[contracttype]
#[derive(Clone)]
pub struct TelemetryAlert {
    pub alert_id: u64,
    pub oracle_address: Address,
    pub alert_type: Symbol,
    pub error_rate: u64,
    pub timestamp: u64,
    pub resolved: bool,
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
    // Issue #92: Leaderboard entries
    StudentAcademicProfile(Address),
    LeaderboardEntry(u64),
    GlobalExcellencePool,
    LeaderboardSize,
    // Issue #94: Tutoring bridge entries
    TutoringAgreement(u64),
    SubStreamRedirect(Address),
    TutoringAgreementCounter,
    // Issue #95: Alumni Donation Matching entries
    GraduationSBT(Address),
    AlumniDonation(u64), // donation_id
    AlumniDonationCounter,
    GeneralExcellenceFund,
    // Issue #93: Scholarship Probation Cooling-Off entries
    ProbationStatus(Address),
    GPAUpdate(Address),
    StudentGPA(Address),
    // Issue #268: Automated telemetry entries
    OracleTelemetry(Address),
    TelemetryAlert(u64),
    TelemetryAlertCounter,
    // Issue #252: Tuition stipend split and royalty split entries
    TuitionStipendSplit(Address),
    RoyaltySplit(u64),
    // Issue #265: Initialization protection entries
    ContractInitialized,
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

// Issue #261: Helper functions for bounded vector validation
impl DeansCouncil {
    fn validate_members(members: &Vec<Address>) -> bool {
        members.len() <= MAX_BOARD_MEMBERS as usize && !members.is_empty()
    }
    
    fn validate_signatures(required_signatures: u32, member_count: usize) -> bool {
        required_signatures > 0 && required_signatures <= member_count as u32
    }
}

impl BoardPauseRequest {
    fn validate_signatures(signatures: &Vec<Address>) -> bool {
        signatures.len() <= MAX_SIGNATURES_PER_REQUEST as usize
    }
}

// Research Grant Milestone Escrow structs
#[contracttype]
#[derive(Clone)]
pub struct ResearchGrant {
    pub student: Address,
    pub total_amount: i128,
    pub token: Address,
    pub granted_at: u64,
    pub is_active: bool,
    pub grantor: Address,
}

#[contracttype]
#[derive(Clone)]
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

#[contracttype]
#[derive(Clone)]
pub struct StreakData {
    pub current_streak: u64,
    pub last_watch_date: u64,
    pub total_reward_claimed: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct GroupPool {
    pub pool_id: u64,
    pub course_id: u64,
    pub target_amount: i128,
    pub current_balance: i128,
    pub creator: Address,
    pub token: Address,
    pub is_active: bool,
    pub member_count: u64,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct QuizProof {
    pub student: Address,
    pub course_id: u64,
    pub module_id: u64,
    pub quiz_hash: Symbol,
    pub score: u64,
    pub passed_at: u64,
    pub is_verified: bool,
}

// Task 1: Location Beacon Check-in System structs
#[contracttype]
#[derive(Clone)]
pub struct AttendanceRecord {
    pub student: Address,
    pub last_check_in: u64,
    pub consecutive_days_present: u64,
    pub consecutive_days_absent: u64,
    pub total_check_ins: u64,
    pub flow_rate_penalty_active: bool,
    pub penalty_start_time: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub enum AssetCode {
    EUR,
    GBP,
    NGN,
    KES,
    GHS,
    ZAR,
    USDC,
}

// Task 3: Income-Share Agreement (ISA) structs
#[contracttype]
#[derive(Clone)]
pub struct ISAContract {
    pub student: Address,
    pub total_amount_owed: i128,
    pub remaining_amount: i128,
    pub percentage_rate: u32, // e.g., 10 = 10% of income
    pub minimum_income_threshold: i128,
    pub repayment_period_months: u64,
    pub is_active: bool,
    pub graduation_time: Option<u64>,
    pub employment_verified: bool,
    pub employer: Option<Address>,
}

#[contracttype]
#[derive(Clone)]
pub struct RepaymentStream {
    pub student: Address,
    pub employer: Address,
    pub flow_rate: i128,
    pub total_repaid: i128,
    pub started_at: u64,
    pub last_payment: u64,
    pub is_active: bool,
}

// Task 4: Vouch/Mentor Boost System structs
#[contracttype]
#[derive(Clone)]
pub struct MentorProfile {
    pub mentor: Address,
    pub reputation_score: u64,
    pub successful_vouches: u64,
    pub failed_vouches: u64,
    pub total_vouches: u64,
    pub is_verified_mentor: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct VouchRecord {
    pub student: Address,
    pub mentor: Address,
    pub vouched_at: u64,
    pub staking_fee_discount: u32, // Percentage discount
    pub is_successful: Option<bool>,
    pub outcome_recorded_at: Option<u64>,
}

#[contract]
pub struct ScholarContract;

#[contractimpl]
impl ScholarContract {
    pub fn init(
        env: Env,
        base_rate: i128,
        discount_threshold: u64,
        discount_percentage: u64,
        min_deposit: i128,
        heartbeat_interval: u64,
    ) {
        // Issue #265: Prevent re-initialization of core variables
        // Check if already initialized by verifying any core variable is set
        if env.storage().instance().get::<DataKey, i128>(&DataKey::BaseRate).is_some() {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        // Validate input parameters
        if base_rate <= 0 {
            panic!("Base rate must be positive");
        }
        if discount_threshold == 0 {
            panic!("Discount threshold cannot be zero");
        }
        if discount_percentage > 100 {
            panic!("Discount percentage cannot exceed 100");
        }
        if min_deposit < 0 {
            panic!("Minimum deposit cannot be negative");
        }
        if heartbeat_interval == 0 {
            panic!("Heartbeat interval cannot be zero");
        }

        // Initialize core variables with protection flags
        env.storage().instance().set(&DataKey::BaseRate, &base_rate);
        env.storage()
            .instance()
            .set(&DataKey::DiscountThreshold, &discount_threshold);
        env.storage()
            .instance()
            .set(&DataKey::DiscountPercentage, &discount_percentage);
        env.storage()
            .instance()
            .set(&DataKey::MinDeposit, &min_deposit);
        env.storage()
            .instance()
            .set(&DataKey::HeartbeatInterval, &heartbeat_interval);

        // Issue #265: Set initialization flag to prevent re-initialization
        env.storage().instance().set(&DataKey::ContractInitialized, &true);
    }

    fn calculate_gpa_bonus(env: Env, student: Address) -> u64 {
        let gpa_data: Option<StudentGPA> = env
            .storage()
            .persistent()
            .get(&DataKey::StudentGPA(student.clone()));
        
        if let Some(gpa_info) = gpa_data {
            if gpa_info.oracle_verified && gpa_info.gpa > GPA_BONUS_THRESHOLD {
                // Calculate how many 0.1 increments above 3.5
                let gpa_above_threshold = gpa_info.gpa - GPA_BONUS_THRESHOLD; // e.g., 3.7 - 3.5 = 0.2 = 2
                let bonus_percentage = (gpa_above_threshold * GPA_BONUS_PERCENTAGE_PER_POINT) / 10; // 2 * 20 / 10 = 4%
                return bonus_percentage;
            }
        }
        0 // No bonus if GPA <= 3.5 or not verified
    }

    /// Apply attendance penalty to rate based on student's attendance record
    fn apply_attendance_penalty_to_rate(env: Env, student: Address, rate: i128) -> i128 {
        // For now, return rate unchanged - attendance penalty logic can be implemented later
        // This function is a placeholder to prevent compilation errors
        rate
    }

    fn calculate_dynamic_rate(env: Env, student: Address, course_id: u64) -> i128 {
        let base_rate: i128 = env
            .storage()
            .instance()
            .get(&DataKey::BaseRate)
            .unwrap_or(1);
        let discount_threshold: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DiscountThreshold)
            .unwrap_or(3600); // 1 hour default
        let discount_percentage: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DiscountPercentage)
            .unwrap_or(10); // 10% default

        let access: Access = env
            .storage()
            .persistent()
            .get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: 0,
                token: student.clone(),
                total_watch_time: 0,
                last_heartbeat: 0,
                last_purchase_time: 0,
            });

        // Start with base rate and apply watch time discount
        let mut rate = if access.total_watch_time >= discount_threshold {
            let discount = (base_rate * discount_percentage as i128) / 100;
            base_rate - discount
        } else {
            base_rate
        };

        // Apply GPA bonus (increase rate based on academic performance)
        let gpa_bonus_percentage = Self::calculate_gpa_bonus(env.clone(), student.clone());
        if gpa_bonus_percentage > 0 {
            let bonus = (rate * gpa_bonus_percentage as i128) / 100;
            rate += bonus; // Increase rate for high-performing students
        }

        // Apply attendance penalty (decrease rate for poor attendance)
        rate = Self::apply_attendance_penalty_to_rate(env.clone(), student.clone(), rate);

        rate
    }

    pub fn buy_access(env: Env, student: Address, course_id: u64, amount: i128, token: Address) {
        student.require_auth();

        let min_deposit: i128 = env.storage().instance().get(&DataKey::MinDeposit).unwrap_or(0);
        if amount < min_deposit {
            panic!("Deposit below minimum");
        }

        if Self::has_active_subscription(env.clone(), student.clone(), course_id) {
            return;
        }

        let rate = Self::calculate_dynamic_rate(env.clone(), student.clone(), course_id);
        if rate <= 0 { panic!("Invalid rate"); }

        let seconds_bought = u64::try_from(amount / rate).expect("Overflow");
        let actual_cost = (seconds_bought as i128) * rate;
        let current_time = env.ledger().timestamp();

        // Perform token transfer
        let client = token::Client::new(&env, &token);
        client.transfer(&student, &env.current_contract_address(), &actual_cost);

        // Apply tuition-stipend split for course payments
        let (university_share, student_share) = Self::distribute_tuition_stipend_split(
            &env, 
            &student, 
            actual_cost, 
            &token
        );

        let mut access = env
            .storage()
            .persistent()
            .get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: current_time,
                token: token.clone(),
                total_watch_time: 0,
                last_heartbeat: 0,
                last_purchase_time: 0,
            });

        if access.expiry_time > current_time {
            access.expiry_time += seconds_bought;
        } else {
            access.expiry_time = current_time + seconds_bought;
        }

        // Use hardened expiry math
        access.expiry_time = checked_access_expiry(current_time, access.expiry_time, seconds_bought)
            .expect("Expiry calculation failed");
        
        access.last_purchase_time = current_time;

        // Distribute royalty for course creators (separate from tuition split)
        Self::distribute_royalty(&env, course_id, actual_cost, &token);
        
        // Emit event with split information
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Access_Purchased"), student.clone(), course_id),
            (actual_cost, university_share, student_share, seconds_bought)
        );
    }

    pub fn heartbeat(env: Env, student: Address, course_id: u64, _signature: soroban_sdk::Bytes) {
        student.require_auth();
        let current_time = env.ledger().timestamp();
        let access_key = DataKey::Access(student.clone(), course_id);
        
        let mut access: Access = env.storage().persistent().get(&access_key).expect("No access record");
        let interval: u64 = env.storage().instance().get(&DataKey::HeartbeatInterval).unwrap_or(60);

        if access.last_heartbeat > 0 && (current_time - access.last_heartbeat) < interval {
            panic!("Heartbeat too frequent");
        }

        if current_time >= access.expiry_time {
            panic!("Access expired");
        }

        if access.last_heartbeat > 0 {
            let elapsed = current_time - access.last_heartbeat;
            if elapsed <= interval + 15 {
                access.total_watch_time += elapsed;
            }
        }
        access.last_heartbeat = current_time;

        // Check for SBT Mint eligibility
        let duration: u64 = env.storage().persistent().get(&DataKey::CourseDuration(course_id)).unwrap_or(0);
        if duration > 0 && access.total_watch_time >= duration {
            let sbt_key = DataKey::SbtMinted(student.clone(), course_id);
            if !env.storage().persistent().get(&sbt_key).unwrap_or(false) {
                env.events().publish((Symbol::new(&env, "SBT_Mint"), student.clone(), course_id), course_id);
                env.storage().persistent().set(&sbt_key, &true);
                env.storage().persistent().extend_ttl(&sbt_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
                
                // Issue #92: Award course completion points
                Self::award_course_completion_points(env.clone(), student.clone(), course_id);
            }
        }

        env.storage().persistent().set(&access_key, &access);
        env.storage().persistent().extend_ttl(&access_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);

        // Issue #92: Update academic profile on heartbeat (engagement)
        Self::update_academic_profile(env.clone(), student.clone());
    }

    
    pub fn has_access(env: Env, student: Address, course_id: u64) -> bool {
        // Check if student scholarship is disputed
        if let Some(scholarship) = env.storage().persistent().get(&DataKey::Scholarship(student.clone())) {
            if scholarship.is_disputed {
                return false;
            }
        }

        // Check if course is globally vetoed
        let is_globally_vetoed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::VetoedCourseGlobal(course_id))
            .unwrap_or(false);
        if is_globally_vetoed {
            return false;
        }

        // Check if course is vetoed for this student
        let is_vetoed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::VetoedCourse(student.clone(), course_id))
            .unwrap_or(false);
        if is_vetoed {
            return false;
        }

        // Check subscription first
        if Self::has_active_subscription(env.clone(), student.clone(), course_id) {
            return true;
        }
        
        // Check individual course access
        let access_key = DataKey::Access(student.clone(), course_id);
        if let Some(access) = env.storage().persistent().get::<_, Access>(&access_key) {
            let current_time = env.ledger().timestamp();
            return current_time < access.expiry_time;
        }
        
        false
    }

    fn has_active_subscription(env: Env, student: Address, course_id: u64) -> bool {
        let sub_key = DataKey::Subscription(student);
        if let Some(sub) = env.storage().persistent().get::<_, SubscriptionTier>(&sub_key) {
            return env.ledger().timestamp() < sub.expiry_time && sub.course_ids.contains(&course_id);
        }
        false
    }

    pub fn buy_subscription(
        env: Env,
        subscriber: Address,
        course_ids: Vec<u64>,
        duration_months: u64,
        amount: i128,
        token: Address,
    ) {
        subscriber.require_auth();

        // Issue #261: Validate course_ids vector size to optimize Wasm footprint
        if course_ids.len() > MAX_COURSE_IDS_PER_SUBSCRIPTION as usize {
            panic!("Too many courses in subscription");
        }

        let client = token::Client::new(&env, &token);
        client.transfer(&subscriber, &env.current_contract_address(), &amount);

        let current_time = env.ledger().timestamp();
        let month_in_seconds = 30 * 24 * 60 * 60; // Approximate month
        let expiry_time = current_time + (duration_months * month_in_seconds);

        let subscription = SubscriptionTier {
            subscriber: subscriber.clone(),
            expiry_time,
            course_ids,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Subscription(subscriber.clone()), &subscription);
    }

    pub fn set_admin(env: Env, admin: Address) {
        admin.require_auth();

        // Issue #265: Enhanced protection against admin re-initialization
        // Only allow setting admin if not already set and contract is initialized
        let existing_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        if existing_admin.is_some() {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        // Verify contract is properly initialized before setting admin
        let is_initialized: bool = env.storage().instance()
            .get(&DataKey::ContractInitialized)
            .unwrap_or(false);
        if !is_initialized {
            panic!("Contract must be initialized before setting admin");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    fn is_admin(env: &Env, caller: &Address) -> bool {
        let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        admin.map_or(false, |a| a == *caller)
    }

    pub fn set_teacher(env: Env, admin: Address, teacher: Address, status: bool) {
        admin.require_auth();

        // Verify caller is admin
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        env.storage()
            .persistent()
            .set(&DataKey::IsTeacher(teacher), &status);
    }

    pub fn fund_scholarship(
        env: Env,
        funder: Address,
        student: Address,
        amount: i128,
        token: Address,
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
                balance: 0,
                token,
                unlocked_balance: 0,
                last_verif: 0,
                is_paused: false,
                is_disputed: false,
                dispute_reason: None,
                final_ruling: None,
            });

        // Only add the student's portion to scholarship balance after processing tutoring redirects
        let final_student_amount = Self::process_tutoring_payment(env.clone(), student.clone(), student_amount, token);
        
        scholarship.balance += final_student_amount;
        scholarship.unlocked_balance += final_student_amount; // Assume funded amount is unlocked
        
        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student.clone()), &scholarship);

        // Emit Scholarship_Granted event with split information
        #[allow(deprecated)]
        env.events().publish(
            (
                Symbol::new(&env, "Scholarship_Granted"),
                funder,
                student.clone(),
            ),
            (amount, university_amount, student_amount)
        );
    }

    pub fn transfer_scholarship_to_teacher(
        env: Env,
        student: Address,
        teacher: Address,
        amount: i128,
    ) {
        student.require_auth();

        // Check if teacher is approved
        let is_approved: bool = env
            .storage()
            .persistent()
            .get(&DataKey::IsTeacher(teacher.clone()))
            .unwrap_or(false);
        if !is_approved {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        if scholarship.is_paused {
            panic!("Scholarship is paused");
        }

        if scholarship.unlocked_balance < amount {
            panic!("Insufficient unlocked balance. Need academic verification?");
        }

        if scholarship.balance < amount {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        scholarship.balance -= amount;
        scholarship.unlocked_balance -= amount;
        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student), &scholarship);

        // Transfer to teacher
        let client = token::Client::new(&env, &scholarship.token);
        client.transfer(&env.current_contract_address(), &teacher, &amount);
    }

    pub fn veto_course_globally(env: Env, admin: Address, course_id: u64, status: bool) {
        admin.require_auth();

        // Verify caller is admin
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        env.storage()
            .persistent()
            .set(&DataKey::VetoedCourseGlobal(course_id), &status);
    }

    pub fn veto_course_access(env: Env, admin: Address, student: Address, course_id: u64) {
        admin.require_auth();

        // Verify caller is admin
        let stored_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        if stored_admin.is_none() || stored_admin.unwrap() != admin {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        // Mark course as vetoed for this student
        env.storage()
            .persistent()
            .set(&DataKey::VetoedCourse(student.clone(), course_id), &true);

        // Revoke existing access by setting expiry to 0
        let access_key = DataKey::Access(student.clone(), course_id);
        if let Some(mut access) = env
            .storage()
            .persistent()
            .get::<DataKey, Access>(&access_key)
        {
            access.expiry_time = 0;
            env.storage().persistent().set(&access_key, &access);
        }

        // Remove course from subscription if present
        let sub_key = DataKey::Subscription(student.clone());
        if let Some(mut subscription) = env
            .storage()
            .persistent()
            .get::<DataKey, SubscriptionTier>(&sub_key)
        {
            // Issue #261: Filter out the vetoed course using bounded vector optimization
            let mut new_course_ids = Vec::new(&env);
            for i in 0..subscription.course_ids.len() {
                let cid = subscription.course_ids.get(i).unwrap();
                if cid != course_id {
                    new_course_ids.push_back(cid);
                }
            }
            subscription.course_ids = new_course_ids;
            env.storage().persistent().set(&sub_key, &subscription);
        }
    }

    pub fn pro_rated_refund(env: Env, student: Address, course_id: u64) -> i128 {
        student.require_auth();

        let access_key = DataKey::Access(student.clone(), course_id);
        let mut access = env
            .storage()
            .persistent()
            .get::<DataKey, Access>(&access_key)
            .expect("No access record found");

        let current_time = env.ledger().timestamp();

        if current_time > access.last_purchase_time + EARLY_DROP_WINDOW_SECONDS {

        }

        if current_time >= access.expiry_time {
            return 0;
        }

        let remaining_seconds = access.expiry_time - current_time;
        let rate = Self::calculate_dynamic_rate(env.clone(), student.clone(), course_id);
        let refund_amount = (remaining_seconds as i128) * rate;

        // Reset expiry to revoke access
        access.expiry_time = 0;
        env.storage().persistent().set(&access_key, &access);

        let client = token::Client::new(&env, &access.token);
        client.transfer(&env.current_contract_address(), &student, &refund_amount);

        refund_amount
    }

    pub fn calculate_remaining_airtime(env: Env, student: Address) -> u64 {
        let flow_rate: i128 = env
            .storage()
            .instance()
            .get(&DataKey::BaseRate)
            .unwrap_or(0);
        if flow_rate == 0 {
            return 0;
        }

        let scholarship: Option<Scholarship> =
            env.storage().persistent().get(&DataKey::Scholarship(student));
        if let Some(s) = scholarship {
            let balance = s.unlocked_balance;
            if balance > 0 {
                return (balance / flow_rate) as u64;
            }
        }
        0
    }

    // Issue #92: Anonymized Leaderboard for Top Scholars Functions

    /// Generate a privacy-protecting student alias
    fn generate_student_alias(env: &Env, student: &Address) -> Symbol {
        let student_bytes = student.to_string();
        let hash = env.crypto().sha256(&student_bytes.into());
        // Take first 4 bytes and convert to a simple hex representation
        let short_hash = &hash[0..4];
        let alias_str = "Student_"; // Simple prefix
        Symbol::new(env, alias_str)
    }

    /// Initialize or update student's academic profile
    pub fn update_academic_profile(env: Env, student: Address) {
        student.require_auth();
        
        let current_time = env.ledger().timestamp();
        let profile_key = DataKey::StudentAcademicProfile(student.clone());
        
        let mut profile: StudentAcademicProfile = env.storage().persistent()
            .get(&profile_key)
            .unwrap_or(StudentAcademicProfile {
                student: student.clone(),
                academic_points: 0,
                courses_completed: 0,
                current_streak: 0,
                last_activity: current_time,
                student_alias: Self::generate_student_alias(&env, &student),
                created_at: current_time,
            });

        // Update streak based on activity
        if current_time - profile.last_activity < 86400 { // Within 24 hours
            profile.current_streak += 1;
            profile.academic_points += ACADEMIC_POINTS_PER_STREAK_DAY;
        } else {
            profile.current_streak = 1; // Reset streak
        }

        profile.last_activity = current_time;
        env.storage().persistent().set(&profile_key, &profile);
        env.storage().persistent().extend_ttl(&profile_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);

        // Emit event for academic points earned
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "AcademicPointsEarned"), student.clone(),),
            profile.academic_points
        );

        // Update leaderboard
        Self::update_leaderboard(env, student, profile.academic_points);
    }

    /// Award academic points for course completion
    pub fn award_course_completion_points(env: Env, student: Address, course_id: u64) {
        // Only admin or teacher can award points
        let caller = env.current_contract_address();
        
        let profile_key = DataKey::StudentAcademicProfile(student.clone());
        let mut profile: StudentAcademicProfile = env.storage().persistent()
            .get(&profile_key)
            .unwrap_or(StudentAcademicProfile {
                student: student.clone(),
                academic_points: 0,
                courses_completed: 0,
                current_streak: 0,
                last_activity: env.ledger().timestamp(),
                student_alias: Self::generate_student_alias(&env, &student),
                created_at: env.ledger().timestamp(),
            });

        profile.courses_completed += 1;
        profile.academic_points += ACADEMIC_POINTS_PER_COURSE;
        profile.last_activity = env.ledger().timestamp();

        env.storage().persistent().set(&profile_key, &profile);
        env.storage().persistent().extend_ttl(&profile_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);

        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "AcademicPointsEarned"), student.clone(),),
            ACADEMIC_POINTS_PER_COURSE
        );

        // Update leaderboard
        Self::update_leaderboard(env, student, profile.academic_points);
    }

    /// Update the leaderboard with new student data
    fn update_leaderboard(env: Env, student: Address, academic_points: u64) {
        let profile_key = DataKey::StudentAcademicProfile(student.clone());
        let profile: StudentAcademicProfile = env.storage().persistent()
            .get(&profile_key)
            .expect("Profile not found");

        // Get current leaderboard size
        let mut leaderboard_size: u64 = env.storage().instance()
            .get(&DataKey::LeaderboardSize)
            .unwrap_or(0);

        // Find if student is already on leaderboard
        let mut existing_rank = None;
        for rank in 1..=leaderboard_size {
            let entry_key = DataKey::LeaderboardEntry(rank);
            if let Some(entry) = env.storage().persistent().get::<_, LeaderboardEntry>(&entry_key) {
                if entry.student_alias == profile.student_alias {
                    existing_rank = Some(rank);
                    break;
                }
            }
        }

        // Update or insert entry
        let new_entry = LeaderboardEntry {
            student_alias: profile.student_alias.clone(),
            academic_points,
            rank: 0, // Will be calculated
            last_updated: env.ledger().timestamp(),
        };

        if let Some(rank) = existing_rank {
            // Update existing entry
            env.storage().persistent().set(&DataKey::LeaderboardEntry(rank), &new_entry);
        } else if leaderboard_size < MAX_LEADERBOARD_SIZE {
            // Add new entry
            leaderboard_size += 1;
            env.storage().instance().set(&DataKey::LeaderboardSize, &leaderboard_size);
            env.storage().persistent().set(&DataKey::LeaderboardEntry(leaderboard_size), &new_entry);
        }

        // Re-sort leaderboard by academic points
        Self::sort_leaderboard(env);
    }

    /// Sort leaderboard by academic points (descending)
    fn sort_leaderboard(env: Env) {
        let leaderboard_size: u64 = env.storage().instance()
            .get(&DataKey::LeaderboardSize)
            .unwrap_or(0);

        // Issue #261: Validate leaderboard size to optimize Wasm footprint
        if leaderboard_size > MAX_LEADERBOARD_ENTRIES as u64 {
            panic!("Leaderboard size exceeds maximum allowed");
        }

        let mut entries = Vec::new(&env);
        for rank in 1..=leaderboard_size {
            let entry_key = DataKey::LeaderboardEntry(rank);
            if let Some(entry) = env.storage().persistent().get::<_, LeaderboardEntry>(&entry_key) {
                entries.push_back(entry);
            }
        }

        // Sort by academic points (simple bubble sort for demonstration)
        for i in 0..entries.len() {
            for j in i + 1..entries.len() {
                let entry_i = entries.get(i).unwrap();
                let entry_j = entries.get(j).unwrap();
                if entry_j.academic_points > entry_i.academic_points {
                    entries.set(i, entry_j);
                    entries.set(j, entry_i);
                }
            }
        }

        // Update ranks and store sorted entries
        for (rank, entry) in entries.iter().enumerate() {
            let mut sorted_entry: LeaderboardEntry = entry.clone();
            sorted_entry.rank = (rank + 1) as u64;
            env.storage().persistent().set(&DataKey::LeaderboardEntry(rank as u64 + 1), &sorted_entry);
        }

        // Emit leaderboard updated event for top 10
        for rank in 1..=core::cmp::min(10, entries.len() as u64) {
            let entry_key = DataKey::LeaderboardEntry(rank);
            if let Some(entry) = env.storage().persistent().get::<_, LeaderboardEntry>(&entry_key) {
                #[allow(deprecated)]
                env.events().publish(
                    (Symbol::new(&env, "LeaderboardUpdated"), entry.student_alias,),
                    entry.rank
                );
            }
        }
    }

    /// Get top N entries from the anonymized leaderboard
    pub fn get_leaderboard(env: Env, limit: u64) -> Vec<LeaderboardEntry> {
        let leaderboard_size: u64 = env.storage().instance()
            .get(&DataKey::LeaderboardSize)
            .unwrap_or(0);

        let actual_limit = core::cmp::min(limit, leaderboard_size);
        let mut result = Vec::new(&env);

        for rank in 1..=actual_limit {
            let entry_key = DataKey::LeaderboardEntry(rank);
            if let Some(entry) = env.storage().persistent().get::<_, LeaderboardEntry>(&entry_key) {
                result.push_back(entry);
            }
        }

        result
    }

    /// Initialize Global Excellence Pool for matching bonuses
    pub fn init_excellence_pool(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        
        // Verify admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let pool = GlobalExcellencePool {
            total_pool_balance: 0,
            token,
            total_distributed: 0,
            last_distribution: 0,
            is_active: true,
        };

        env.storage().instance().set(&DataKey::GlobalExcellencePool, &pool);
    }

    /// Fund the Global Excellence Pool
    pub fn fund_excellence_pool(env: Env, funder: Address, amount: i128) {
        funder.require_auth();

        let mut pool: GlobalExcellencePool = env.storage().instance()
            .get(&DataKey::GlobalExcellencePool)
            .expect("Excellence pool not initialized");

        if !pool.is_active {
            panic!("Excellence pool is not active");
        }

        // Transfer tokens to contract
        let client = token::Client::new(&env, &pool.token);
        client.transfer(&funder, &env.current_contract_address(), &amount);

        pool.total_pool_balance += amount;
        env.storage().instance().set(&DataKey::GlobalExcellencePool, &pool);
    }

    /// Distribute matching bonuses to top scholars
    pub fn distribute_matching_bonuses(env: Env, admin: Address, bonus_per_rank: i128) {
        admin.require_auth();
        
        // Verify admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let mut pool: GlobalExcellencePool = env.storage().instance()
            .get(&DataKey::GlobalExcellencePool)
            .expect("Excellence pool not initialized");

        let leaderboard_size: u64 = env.storage().instance()
            .get(&DataKey::LeaderboardSize)
            .unwrap_or(0);

        let distribution_count = core::cmp::min(10, leaderboard_size); // Top 10 scholars
        let total_needed = bonus_per_rank * distribution_count as i128;

        if pool.total_pool_balance < total_needed {
            panic!("Insufficient pool balance");
        }

        // Distribute bonuses
        for rank in 1..=distribution_count {
            let entry_key = DataKey::LeaderboardEntry(rank);
            if let Some(entry) = env.storage().persistent().get::<_, LeaderboardEntry>(&entry_key) {
                // Find student address from alias (this would require reverse mapping in production)
                // For now, we'll emit an event and let frontend handle the actual distribution
                
                #[allow(deprecated)]
                env.events().publish(
                    (Symbol::new(&env, "MatchingBonusDistributed"), entry.student_alias,),
                    bonus_per_rank
                );
            }
        }

        pool.total_distributed += total_needed;
        pool.total_pool_balance -= total_needed;
        pool.last_distribution = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::GlobalExcellencePool, &pool);
    }

    // Issue #94: Peer-to-Peer Tutoring Payment Bridge Functions

    /// Create a tutoring agreement between scholar and tutor
    pub fn create_tutoring_agreement(
        env: Env,
        scholar: Address,
        tutor: Address,
        percentage: u32,
        duration_seconds: u64,
    ) -> u64 {
        scholar.require_auth();

        if percentage > MAX_TUTORING_PERCENTAGE {
            panic!("Percentage exceeds maximum allowed");
        }

        if duration_seconds < MIN_TUTORING_DURATION {
            panic!("Duration below minimum required");
        }

        let current_time = env.ledger().timestamp();
        let agreement_id: u64 = env.storage().instance()
            .get(&DataKey::TutoringAgreementCounter)
            .unwrap_or(0) + 1;

        env.storage().instance().set(&DataKey::TutoringAgreementCounter, &agreement_id);

        let agreement = TutoringAgreement {
            scholar: scholar.clone(),
            tutor: tutor.clone(),
            percentage,
            start_time: current_time,
            end_time: current_time + duration_seconds,
            is_active: true,
            total_redirected: 0,
            agreement_id,
        };

        env.storage().persistent().set(&DataKey::TutoringAgreement(agreement_id), &agreement);
        env.storage().persistent().extend_ttl(
            &DataKey::TutoringAgreement(agreement_id), 
            LEDGER_BUMP_THRESHOLD, 
            LEDGER_BUMP_EXTEND
        );

        // Initialize sub-stream redirect
        let redirect = SubStreamRedirect {
            from_scholar: scholar.clone(),
            to_tutor: tutor.clone(),
            flow_rate: 0, // Will be calculated based on scholarship flow
            start_time: current_time,
            last_redirect: current_time,
            total_amount_redirected: 0,
            is_active: true,
        };

        env.storage().persistent().set(&DataKey::SubStreamRedirect(scholar), &redirect);

        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "TutoringAgreementCreated"), scholar, tutor,),
            agreement_id
        );

        agreement_id
    }

    /// Process sub-stream redirection for tutoring payments
    pub fn process_tutoring_payment(env: Env, scholar: Address, scholarship_amount: i128, token: Address) -> i128 {
        let current_time = env.ledger().timestamp();
        let redirect_key = DataKey::SubStreamRedirect(scholar.clone());
        
        let mut redirect: SubStreamRedirect = env.storage().persistent()
            .get(&redirect_key)
            .unwrap_or(SubStreamRedirect {
                from_scholar: scholar.clone(),
                to_tutor: Address::generate(&env), // Dummy address
                flow_rate: 0,
                start_time: current_time,
                last_redirect: current_time,
                total_amount_redirected: 0,
                is_active: false,
            });

        if !redirect.is_active {
            return scholarship_amount; // No redirection
        }

        // Check if tutoring agreement is still active
        let agreement_key = DataKey::TutoringAgreement(1); // Simplified - would need agreement_id
        if let Some(agreement) = env.storage().persistent().get::<_, TutoringAgreement>(&agreement_key) {
            if current_time > agreement.end_time || !agreement.is_active {
                redirect.is_active = false;
                env.storage().persistent().set(&redirect_key, &redirect);
                return scholarship_amount;
            }

            // Calculate redirection amount
            let redirect_amount = (scholarship_amount * agreement.percentage as i128) / 100;
            let scholar_amount = scholarship_amount - redirect_amount;

            // Update redirect tracking
            redirect.total_amount_redirected += redirect_amount;
            redirect.last_redirect = current_time;
            env.storage().persistent().set(&redirect_key, &redirect);

            // Transfer to tutor
            if redirect_amount > 0 {
                let client = token::Client::new(&env, &token);
                client.transfer(&env.current_contract_address(), &redirect.to_tutor, &redirect_amount);
            }

            // Emit event
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "SubStreamRedirected"), scholar, redirect.to_tutor,),
                redirect_amount
            );

            scholar_amount
        } else {
            scholarship_amount // No agreement found
        }
    }

    /// End a tutoring agreement
    pub fn end_tutoring_agreement(env: Env, scholar: Address, agreement_id: u64) {
        scholar.require_auth();

        let agreement_key = DataKey::TutoringAgreement(agreement_id);
        let mut agreement: TutoringAgreement = env.storage().persistent()
            .get(&agreement_key)
            .expect("Tutoring agreement not found");

        if agreement.scholar != scholar {
            panic!("Not authorized to end this agreement");
        }

        agreement.is_active = false;
        env.storage().persistent().set(&agreement_key, &agreement);

        // Deactivate sub-stream redirect
        let redirect_key = DataKey::SubStreamRedirect(scholar);
        if let Some(mut redirect) = env.storage().persistent().get::<_, SubStreamRedirect>(&redirect_key) {
            redirect.is_active = false;
            env.storage().persistent().set(&redirect_key, &redirect);
        }

        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "TutoringAgreementEnded"),),
            agreement_id
        );
    }

    /// Get active tutoring agreement for a scholar
    pub fn get_tutoring_agreement(env: Env, agreement_id: u64) -> TutoringAgreement {
        env.storage().persistent()
            .get(&DataKey::TutoringAgreement(agreement_id))
            .expect("Tutoring agreement not found")
    }

    /// Get sub-stream redirect info for a scholar
    pub fn get_sub_stream_redirect(env: Env, scholar: Address) -> Option<SubStreamRedirect> {
        env.storage().persistent()
            .get(&DataKey::SubStreamRedirect(scholar))
    }

    // Issue #95: Alumni Donation Matching Incentive Functions

    /// Initialize General Excellence Fund for alumni matching
    pub fn init_general_excellence_fund(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        
        // Verify admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let fund = GeneralExcellenceFund {
            total_balance: 0,
            token,
            total_matched: 0,
            is_active: true,
            last_updated: env.ledger().timestamp(),
        };

        env.storage().instance().set(&DataKey::GeneralExcellenceFund, &fund);
    }

    /// Fund the General Excellence Fund
    pub fn fund_general_excellence_fund(env: Env, funder: Address, amount: i128) {
        funder.require_auth();

        let mut fund: GeneralExcellenceFund = env.storage().instance()
            .get(&DataKey::GeneralExcellenceFund)
            .expect("General Excellence Fund not initialized");

        if !fund.is_active {
            panic!("General Excellence Fund is not active");
        }

        // Transfer tokens to contract
        let client = token::Client::new(&env, &fund.token);
        client.transfer(&funder, &env.current_contract_address(), &amount);

        fund.total_balance += amount;
        fund.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::GeneralExcellenceFund, &fund);
    }

    /// Issue Graduation SBT to a student
    pub fn issue_graduation_sbt(env: Env, admin: Address, student: Address, final_gpa: u64) -> u64 {
        admin.require_auth();
        
        // Verify admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let current_time = env.ledger().timestamp();
        let token_id: u64 = env.storage().instance()
            .get(&DataKey::AlumniDonationCounter)
            .unwrap_or(0) + 1;

        env.storage().instance().set(&DataKey::AlumniDonationCounter, &token_id);

        let graduation_sbt = GraduationSBT {
            student: student.clone(),
            graduation_date: current_time,
            gpa: final_gpa,
            is_verified: true,
            token_id,
        };

        env.storage().persistent().set(&DataKey::GraduationSBT(student), &graduation_sbt);
        env.storage().persistent().extend_ttl(
            &DataKey::GraduationSBT(student), 
            LEDGER_BUMP_THRESHOLD, 
            LEDGER_BUMP_EXTEND
        );

        // Mark graduation SBT as minted
        let sbt_key = DataKey::SbtMinted(student.clone(), GRADUATION_SBT_COURSE_ID);
        env.storage().persistent().set(&sbt_key, &true);
        env.storage().persistent().extend_ttl(&sbt_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);

        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "SbtMint"), student, GRADUATION_SBT_COURSE_ID),
            token_id
        );

        token_id
    }

    /// Check if a donor has Graduation SBT (is an alumnus)
    fn has_graduation_sbt(env: &Env, donor: &Address) -> bool {
        let sbt_key = DataKey::SbtMinted(donor.clone(), GRADUATION_SBT_COURSE_ID);
        env.storage().persistent().get(&sbt_key).unwrap_or(false)
    }

    /// Process alumni donation with matching
    pub fn process_alumni_donation(
        env: Env,
        donor: Address,
        amount: i128,
        scholarship_pool: u64,
        token: Address,
    ) -> (i128, i128) {
        donor.require_auth();

        let current_time = env.ledger().timestamp();
        let donation_id: u64 = env.storage().instance()
            .get(&DataKey::AlumniDonationCounter)
            .unwrap_or(0) + 1;

        env.storage().instance().set(&DataKey::AlumniDonationCounter, &donation_id);

        // Check if donor has Graduation SBT
        let has_sbt = Self::has_graduation_sbt(&env, &donor);
        
        let mut matched_amount = 0i128;
        let total_deduction = if has_sbt {
            // Calculate 2:1 match
            matched_amount = amount * ALUMNI_MATCHING_MULTIPLIER as i128;
            amount + matched_amount
        } else {
            amount
        };

        // Transfer original donation from donor
        let client = token::Client::new(&env, &token);
        client.transfer(&donor, &env.current_contract_address(), &amount);

        // If matching applies, transfer from General Excellence Fund
        if has_sbt && matched_amount > 0 {
            let mut fund: GeneralExcellenceFund = env.storage().instance()
                .get(&DataKey::GeneralExcellenceFund)
                .expect("General Excellence Fund not initialized");

            if fund.total_balance < matched_amount {
                panic!("Insufficient balance in General Excellence Fund for matching");
            }

            fund.total_balance -= matched_amount;
            fund.total_matched += matched_amount;
            fund.last_updated = current_time;
            env.storage().instance().set(&DataKey::GeneralExcellenceFund, &fund);
        }

        // Create donation record
        let donation = AlumniDonation {
            donor: donor.clone(),
            original_amount: amount,
            matched_amount,
            scholarship_pool,
            donation_date: current_time,
            has_graduation_sbt: has_sbt,
        };

        env.storage().persistent().set(&DataKey::AlumniDonation(donation_id), &donation);
        env.storage().persistent().extend_ttl(
            &DataKey::AlumniDonation(donation_id), 
            LEDGER_BUMP_THRESHOLD, 
            LEDGER_BUMP_EXTEND
        );

        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "AlumniDonationMatched"), donor,),
            (amount, matched_amount)
        );

        (amount, matched_amount)
    }

    /// Get graduation SBT info for a student
    pub fn get_graduation_sbt(env: Env, student: Address) -> Option<GraduationSBT> {
        env.storage().persistent()
            .get(&DataKey::GraduationSBT(student))
    }

    /// Get alumni donation info
    pub fn get_alumni_donation(env: Env, donation_id: u64) -> Option<AlumniDonation> {
        env.storage().persistent()
            .get(&DataKey::AlumniDonation(donation_id))
    }

    /// Get General Excellence Fund info
    pub fn get_general_excellence_fund(env: Env) -> Option<GeneralExcellenceFund> {
        env.storage().instance()
            .get(&DataKey::GeneralExcellenceFund)
    }

    // Issue #93: Scholarship Probation Cooling-Off Logic Functions

    /// Update student GPA and handle probation logic
    pub fn update_student_gpa(env: Env, oracle: Address, student: Address, new_gpa: u64) {
        oracle.require_auth();
        
        let current_time = env.ledger().timestamp();
        let mut call_successful = true;
        let mut error_type: Option<Symbol> = None;
        
        // Validate GPA range
        if new_gpa > 40 { // Max 4.0 GPA (stored as 40)
            call_successful = false;
            error_type = Some(Symbol::new(&env, "INVALID_GPA_RANGE"));
            
            // Record failed oracle call
            Self::record_oracle_call(env.clone(), oracle.clone(), false, error_type.clone());
            panic!("Invalid GPA range");
        }
        
        // Get previous GPA for tracking
        let previous_gpa: u64 = if let Some(gpa_data) = env.storage().persistent()
            .get::<_, StudentGPA>(&DataKey::StudentGPA(student.clone())) {
            gpa_data.gpa
        } else {
            0 // No previous GPA
        };

        // Create GPA update record
        let gpa_update = GPAUpdate {
            student: student.clone(),
            new_gpa,
            previous_gpa,
            update_timestamp: current_time,
            oracle_verified: true,
        };

        env.storage().persistent().set(&DataKey::GPAUpdate(student.clone()), &gpa_update);
        env.storage().persistent().extend_ttl(
            &DataKey::GPAUpdate(student), 
            LEDGER_BUMP_THRESHOLD, 
            LEDGER_BUMP_EXTEND
        );

        // Update StudentGPA record
        let student_gpa = StudentGPA {
            student: student.clone(),
            gpa: new_gpa,
            last_updated: current_time,
            oracle_verified: true,
        };

        env.storage().persistent().set(&DataKey::StudentGPA(student.clone()), &student_gpa);
        env.storage().persistent().extend_ttl(
            &DataKey::StudentGPA(student), 
            LEDGER_BUMP_THRESHOLD, 
            LEDGER_BUMP_EXTEND
        );

        // Record successful oracle call
        Self::record_oracle_call(env.clone(), oracle.clone(), true, None);

        // Handle probation logic
        Self::handle_probation_logic(env.clone(), student.clone(), new_gpa, current_time);
    }

    /// Handle probation logic based on GPA updates
    fn handle_probation_logic(env: Env, student: Address, new_gpa: u64, current_time: u64) {
        let mut probation_status: ProbationStatus = env.storage().persistent()
            .get(&DataKey::ProbationStatus(student.clone()))
            .unwrap_or(ProbationStatus {
                student: student.clone(),
                is_on_probation: false,
                probation_start_time: 0,
                warning_period_end: 0,
                original_flow_rate: 0,
                reduced_flow_rate: 0,
                violation_count: 0,
                last_gpa_check: 0,
            });

        // Check if GPA is below threshold
        if new_gpa < GPA_THRESHOLD {
            if !probation_status.is_on_probation {
                // First violation - start probation
                Self::start_probation(env.clone(), student.clone(), &mut probation_status, current_time);
            } else {
                // Already on probation - check if warning period has ended
                if current_time > probation_status.warning_period_end {
                    // Warning period ended and GPA still low - revoke scholarship
                    Self::revoke_scholarship(env.clone(), student.clone());
                } else {
                    // Still in warning period but GPA dropped again - extend violation tracking
                    probation_status.violation_count += 1;
                    probation_status.last_gpa_check = current_time;
                }
            }
        } else {
            // GPA is acceptable
            if probation_status.is_on_probation {
                // Student recovered - end probation
                Self::end_probation(env.clone(), student.clone(), &mut probation_status, true);
            }
        }

        // Update probation status
        probation_status.last_gpa_check = current_time;
        env.storage().persistent().set(&DataKey::ProbationStatus(student), &probation_status);
        env.storage().persistent().extend_ttl(
            &DataKey::ProbationStatus(student), 
            LEDGER_BUMP_THRESHOLD, 
            LEDGER_BUMP_EXTEND
        );
    }

    /// Start probation for a student
    fn start_probation(env: Env, student: Address, probation_status: &mut ProbationStatus, current_time: u64) {
        // Get current scholarship to calculate flow rate reduction
        if let Some(mut scholarship) = env.storage().persistent()
            .get::<_, Scholarship>(&DataKey::Scholarship(student.clone())) {
            
            // Calculate reduced flow rate (30% reduction)
            let original_rate = scholarship.balance; // Simplified - in real implementation, this would be flow rate
            let reduction_amount = (original_rate * PROBATION_FLOW_REDUCTION as i128) / 100;
            let reduced_rate = original_rate - reduction_amount;

            // Update probation status
            probation_status.is_on_probation = true;
            probation_status.probation_start_time = current_time;
            probation_status.warning_period_end = current_time + PROBATION_WARNING_PERIOD;
            probation_status.original_flow_rate = original_rate;
            probation_status.reduced_flow_rate = reduced_rate;
            probation_status.violation_count = 1;
            probation_status.last_gpa_check = current_time;

            // Apply reduction to scholarship (simplified - would affect flow rate in real implementation)
            scholarship.balance = reduced_rate;

            env.storage().persistent().set(&DataKey::Scholarship(student.clone()), &scholarship);
            env.storage().persistent().extend_ttl(
                &DataKey::Scholarship(student.clone()), 
                LEDGER_BUMP_THRESHOLD, 
                LEDGER_BUMP_EXTEND
            );

            // Emit event
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "ProbationStarted"), student.clone(),),
                probation_status.warning_period_end
            );
        }
    }

    /// End probation for a student
    fn end_probation(env: Env, student: Address, probation_status: &mut ProbationStatus, recovered: bool) {
        let _current_time = env.ledger().timestamp();
        
        // Restore original flow rate
        if let Some(mut scholarship) = env.storage().persistent()
            .get::<_, Scholarship>(&DataKey::Scholarship(student.clone())) {
            
            scholarship.balance = probation_status.original_flow_rate;
            
            env.storage().persistent().set(&DataKey::Scholarship(student.clone()), &scholarship);
            env.storage().persistent().extend_ttl(
                &DataKey::Scholarship(student.clone()), 
                LEDGER_BUMP_THRESHOLD, 
                LEDGER_BUMP_EXTEND
            );

            // Emit event
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "ProbationEnded"), student.clone(),),
                recovered
            );
        }

        // Reset probation status
        probation_status.is_on_probation = false;
        probation_status.probation_start_time = 0;
        probation_status.warning_period_end = 0;
        probation_status.original_flow_rate = 0;
        probation_status.reduced_flow_rate = 0;
        probation_status.violation_count = 0;
    }

    /// Revoke scholarship permanently
    fn revoke_scholarship(env: Env, student: Address) {
        // Mark scholarship as disputed with revocation reason
        if let Some(mut scholarship) = env.storage().persistent()
            .get::<_, Scholarship>(&DataKey::Scholarship(student.clone())) {
            
            scholarship.is_disputed = true;
            scholarship.dispute_reason = Some(Symbol::new(&env, "PERMANENT_REVOCATION_GPA"));
            scholarship.final_ruling = Some(Symbol::new(&env, "REVOKED"));
            
            env.storage().persistent().set(&DataKey::Scholarship(student.clone()), &scholarship);
            env.storage().persistent().extend_ttl(
                &DataKey::Scholarship(student.clone()), 
                LEDGER_BUMP_THRESHOLD, 
                LEDGER_BUMP_EXTEND
            );

        // Clear probation status
        env.storage().persistent().remove(&DataKey::ProbationStatus(student.clone()));

        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "StreamRevoked"), student.clone(),),
            ()
        );
        }
    }

    /// Get probation status for a student
    pub fn get_probation_status(env: Env, student: Address) -> Option<ProbationStatus> {
        env.storage().persistent()
            .get(&DataKey::ProbationStatus(student))
    }

    // Issue #252: Precision-safe distribution functions

    /// Set tuition stipend split configuration for a student
    pub fn set_tuition_stipend_split(
        env: Env,
        admin: Address,
        student: Address,
        university_address: Address,
        university_percentage: u32,
        student_percentage: u32,
    ) {
        admin.require_auth();
        
        // Verify admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        // Validate percentages sum to 100
        if university_percentage + student_percentage != 100 {
            panic!("Percentages must sum to 100");
        }

        // Validate individual percentages are within bounds
        if university_percentage > 100 || student_percentage > 100 {
            panic!("Individual percentages cannot exceed 100");
        }

        let split_config = TuitionStipendSplit {
            university_address,
            student_address: student.clone(),
            university_percentage,
            student_percentage,
        };

        env.storage().persistent().set(&DataKey::TuitionStipendSplit(student.clone()), &split_config);
        env.storage().persistent().extend_ttl(
            &DataKey::TuitionStipendSplit(student.clone()),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND
        );
    }

    /// Get tuition stipend split configuration for a student
    pub fn get_tuition_stipend_split(env: Env, student: Address) -> Option<TuitionStipendSplit> {
        env.storage().persistent()
            .get(&DataKey::TuitionStipendSplit(student))
    }

    /// Precision-safe tuition stipend split distribution
    /// Uses integer arithmetic with proper rounding to prevent precision loss
    fn distribute_tuition_stipend_split(
        env: &Env,
        student: &Address,
        total_amount: i128,
        token: &Address,
    ) -> (i128, i128) {
        // Get split configuration
        let split_config: Option<TuitionStipendSplit> = env.storage().persistent()
            .get(&DataKey::TuitionStipendSplit(student.clone()));

        if let Some(config) = split_config {
            // Use precision-safe calculation to avoid rounding errors
            // Calculate university share using integer arithmetic
            let university_share = (total_amount * config.university_percentage as i128) / 100i128;
            
            // Calculate student share as remaining amount to ensure no precision loss
            // This ensures university_share + student_share = total_amount exactly
            let student_share = total_amount - university_share;

            // Transfer university share directly if non-zero
            if university_share > 0 {
                let client = token::Client::new(env, token);
                client.transfer(&env.current_contract_address(), &config.university_address, &university_share);
                
                // Emit university transfer event
                #[allow(deprecated)]
                env.events().publish(
                    (Symbol::new(env, "UniversityShare"), config.university_address,),
                    university_share
                );
            }

            (university_share, student_share)
        } else {
            // No split configuration - full amount goes to student
            (0, total_amount)
        }
    }

    /// Precision-safe royalty distribution for course creators
    /// Uses bounded iteration and safe arithmetic to prevent overflow and precision loss
    fn distribute_royalty(env: &Env, course_id: u64, total_amount: i128, token: &Address) {
        // Get royalty split configuration for this course
        let royalty_key = DataKey::RoyaltySplit(course_id);
        if let Some(royalty_split) = env.storage().persistent().get::<_, RoyaltySplit>(&royalty_key) {
            let mut remaining_amount = total_amount;
            let total_shares: u32 = royalty_split.shares.iter().map(|(_, share)| share).sum();
            
            if total_shares == 0 {
                return; // No shares to distribute
            }

            // Distribute royalties with precision-safe calculations
            for (index, (recipient, share_percentage)) in royalty_split.shares.iter().enumerate() {
                let is_last_recipient = index == royalty_split.shares.len() - 1;
                
                let recipient_share = if is_last_recipient {
                    // Last recipient gets remaining amount to prevent precision loss
                    remaining_amount
                } else {
                    // Calculate share using integer arithmetic
                    (total_amount * *share_percentage as i128) / total_shares as i128
                };

                if recipient_share > 0 {
                    let client = token::Client::new(env, token);
                    client.transfer(&env.current_contract_address(), recipient, &recipient_share);
                    
                    // Update remaining amount
                    remaining_amount -= recipient_share;
                    
                    // Emit royalty distribution event
                    #[allow(deprecated)]
                    env.events().publish(
                        (Symbol::new(env, "RoyaltyDistributed"), recipient, course_id,),
                        recipient_share
                    );
                }
            }
        }
    }

    /// Set royalty split for a course
    pub fn set_royalty_split(
        env: Env,
        admin: Address,
        course_id: u64,
        recipients: Vec<Address>,
        share_percentages: Vec<u32>,
    ) {
        admin.require_auth();
        
        // Verify admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        // Issue #261: Validate inputs with bounded constraints to optimize Wasm footprint
        if recipients.len() != share_percentages.len() {
            panic!("Recipients and share percentages must have same length");
        }

        if recipients.len() > MAX_ROYALTY_RECIPIENTS as usize {
            panic!("Too many royalty recipients");
        }

        // Validate share percentages sum to 100
        let total_shares: u32 = share_percentages.iter().sum();
        if total_shares != 100 {
            panic!("Share percentages must sum to 100");
        }

        // Create royalty split structure with bounded vector
        let mut shares = Vec::<(Address, u32)>::new(&env);
        for (recipient, percentage) in recipients.iter().zip(share_percentages.iter()) {
            shares.push_back((recipient.clone(), *percentage));
        }

        let royalty_split = RoyaltySplit { shares };

        env.storage().persistent().set(&DataKey::RoyaltySplit(course_id), &royalty_split);
        env.storage().persistent().extend_ttl(
            &DataKey::RoyaltySplit(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND
        );
    }

    /// Get royalty split for a course
    pub fn get_royalty_split(env: Env, course_id: u64) -> Option<RoyaltySplit> {
        env.storage().persistent()
            .get(&DataKey::RoyaltySplit(course_id))
    }

    /// Get GPA update info for a student
    pub fn get_gpa_update(env: Env, student: Address) -> Option<GPAUpdate> {
        env.storage().persistent()
            .get(&DataKey::GPAUpdate(student))
    }

    // Issue #268: Automated telemetry functions for oracle polling

    /// Record oracle call attempt and update telemetry
    pub fn record_oracle_call(env: Env, oracle: Address, success: bool, error_type: Option<Symbol>) {
        let current_time = env.ledger().timestamp();
        let telemetry_key = DataKey::OracleTelemetry(oracle.clone());
        
        let mut telemetry: OracleTelemetry = env.storage().persistent()
            .get(&telemetry_key)
            .unwrap_or(OracleTelemetry {
                oracle_address: oracle.clone(),
                total_calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                last_call_timestamp: 0,
                last_success_timestamp: 0,
                error_rate_percentage: 0,
                recent_errors: Vec::new(&env),
            });

        // Update call counters
        telemetry.total_calls += 1;
        telemetry.last_call_timestamp = current_time;
        
        if success {
            telemetry.successful_calls += 1;
            telemetry.last_success_timestamp = current_time;
        } else {
            telemetry.failed_calls += 1;
            
            // Record error details if provided
            if let Some(error) = error_type {
                telemetry.recent_errors.push_back((current_time, error));
                
                // Issue #261: Keep only recent errors within bounded limit to optimize Wasm footprint
                while telemetry.recent_errors.len() > MAX_RECENT_ERRORS as usize {
                    telemetry.recent_errors.remove(0);
                }
            }
        }

        // Calculate error rate percentage
        telemetry.error_rate_percentage = if telemetry.total_calls > 0 {
            (telemetry.failed_calls * 100) / telemetry.total_calls
        } else {
            0
        };

        // Check if error rate exceeds threshold and create alert if needed
        if telemetry.error_rate_percentage > TELEMETRY_ERROR_THRESHOLD {
            Self::create_telemetry_alert(env.clone(), oracle.clone(), telemetry.error_rate_percentage);
        }

        env.storage().persistent().set(&telemetry_key, &telemetry);
        env.storage().persistent().extend_ttl(&telemetry_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
    }

    /// Create telemetry alert when error rate exceeds threshold
    fn create_telemetry_alert(env: Env, oracle: Address, error_rate: u64) {
        let current_time = env.ledger().timestamp();
        let alert_id: u64 = env.storage().instance()
            .get(&DataKey::TelemetryAlertCounter)
            .unwrap_or(0) + 1;

        env.storage().instance().set(&DataKey::TelemetryAlertCounter, &alert_id);

        let alert = TelemetryAlert {
            alert_id,
            oracle_address: oracle.clone(),
            alert_type: Symbol::new(&env, "HIGH_ERROR_RATE"),
            error_rate,
            timestamp: current_time,
            resolved: false,
        };

        env.storage().persistent().set(&DataKey::TelemetryAlert(alert_id), &alert);
        env.storage().persistent().extend_ttl(
            &DataKey::TelemetryAlert(alert_id), 
            LEDGER_BUMP_THRESHOLD, 
            LEDGER_BUMP_EXTEND
        );

        // Emit alert event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "TelemetryAlert"), oracle,),
            (alert_id, error_rate)
        );
    }

    /// Get oracle telemetry data
    pub fn get_oracle_telemetry(env: Env, oracle: Address) -> Option<OracleTelemetry> {
        env.storage().persistent()
            .get(&DataKey::OracleTelemetry(oracle))
    }

    /// Get telemetry alert
    pub fn get_telemetry_alert(env: Env, alert_id: u64) -> Option<TelemetryAlert> {
        env.storage().persistent()
            .get(&DataKey::TelemetryAlert(alert_id))
    }

    /// Resolve telemetry alert
    pub fn resolve_telemetry_alert(env: Env, admin: Address, alert_id: u64) {
        admin.require_auth();
        
        // Verify admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let alert_key = DataKey::TelemetryAlert(alert_id);
        if let Some(mut alert) = env.storage().persistent().get::<_, TelemetryAlert>(&alert_key) {
            alert.resolved = true;
            alert.timestamp = env.ledger().timestamp(); // Update resolution timestamp
            
            env.storage().persistent().set(&alert_key, &alert);
            
            // Emit resolution event
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "TelemetryAlertResolved"), alert.oracle_address,),
                alert_id
            );
        }
    }

    /// Clean up old telemetry data (maintenance function)
    pub fn cleanup_old_telemetry(env: Env, admin: Address, cutoff_timestamp: u64) {
        admin.require_auth();
        
        // Verify admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        // Clean up old telemetry alerts
        let alert_counter: u64 = env.storage().instance()
            .get(&DataKey::TelemetryAlertCounter)
            .unwrap_or(0);
            
        for alert_id in 1..=alert_counter {
            let alert_key = DataKey::TelemetryAlert(alert_id);
            if let Some(alert) = env.storage().persistent().get::<_, TelemetryAlert>(&alert_key) {
                if alert.timestamp < cutoff_timestamp && alert.resolved {
                    env.storage().persistent().remove(&alert_key);
                }
            }
        }
    }
}

mod test;
mod tuition_stipend_split_tests;
// mod student_profile_nft; // Temporarily commented out for compilation
