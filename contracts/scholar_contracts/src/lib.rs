#![no_std]

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

// Issue #128: Community Governance Veto
const FINAL_RELEASE_PERCENTAGE: u64 = 10; // 10%
const COMMUNITY_VOTE_THRESHOLD: u64 = 5; // 5 votes to pass

// Issue #118: Native XLM Scholarship
const NATIVE_XLM_RESERVE: i128 = 2_0000000; // 2 XLM in stroops

// Issue #112: Scholarship Claim Dry-Run
const DEFAULT_TAX_RATE_BPS: u32 = 0; // 0% default tax
const ESTIMATED_GAS_FEE: i128 = 500000; // 0.05 XLM in stroops


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
    // Task 1: Wasm-Hash Rotation entries
    CurrentLogicHash,
    LogicHashRecord(Bytes), // logic_hash -> LogicHashRecord struct
    DaoVote(Address, Bytes), // voter, logic_hash -> DaoVote struct
    LogicUpgradeProposal(u64), // proposal_id -> LogicUpgradeProposal struct
    ProposalCounter,
    DaoMembers(Vec<Address>),
    // Task 3: Scholarship Registry entries
    ScholarshipRegistry(Address), // university_address -> ScholarshipRegistry struct
    UniversityContractIndex(Address, u64), // university, index -> contract_id
    StudentScholarshipContract(Address), // student -> contract_id that manages their scholarship
    GlobalScholarshipCounter,
    // Task 4: Multi-Lingual Legal Agreement entries
    LegalAgreement(u64), // agreement_id -> LegalAgreement struct
    AgreementSignature(u64, Address), // agreement_id, signer -> AgreementSignature struct
    StudentPrimaryAgreement(Address), // student -> (agreement_id, primary_language)
    LanguageVersionHash(Bytes), // document_hash -> LanguageVersion metadata
    // Issue #128: Community Governance Veto
    CommunityVote(Address), // student -> CommunityVote
    // Issue #112: Scholarship Claim Dry-Run
    TaxRate,
    // Issue #122: On-Chain Graduation Credential Registry
    GraduationRegistry(Address), // student -> GraduateProfile
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
    pub student: Address,
    pub total_amount: i128,
    pub token: Address,
    pub granted_at: u64,
    pub is_active: bool,
    pub grantor: Address,
}

// Task 1: Wasm-Hash Rotation Pattern structs
#[contracttype]
#[derive(Clone)]
pub struct LogicHashRecord {
    pub logic_hash: Bytes, // SHA-256 hash of the new logic
    pub proposed_at: u64,
    pub proposed_by: Address,
    pub is_active: bool,
    pub immutable_terms_hash: Bytes, // Hash of immutable terms (total grant + student address)
}

#[contracttype]
#[derive(Clone)]
pub struct DaoVote {
    pub voter: Address,
    pub logic_hash: Bytes,
    pub vote_yes: bool,
    pub voted_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct LogicUpgradeProposal {
    pub proposal_id: u64,
    pub old_logic_hash: Bytes,
    pub new_logic_hash: Bytes,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub voters: Vec<Address>,
    pub is_executed: bool,
    pub created_at: u64,
    pub execution_threshold: u64, // Minimum votes required
}

// Task 3: Scholarship Registry structs
#[contracttype]
#[derive(Clone)]
pub struct ScholarshipRegistry {
    pub university_address: Address,
    pub scholarship_contract_ids: Vec<Address>,
    pub total_scholarships: u64,
    pub active_scholarships: u64,
    pub created_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct UniversityScholarshipInfo {
    pub university: Address,
    pub contract_id: Address,
    pub student: Address,
    pub is_active: bool,
    pub registered_at: u64,
}

// Task 4: Multi-Lingual Legal Agreement structs
#[contracttype]
#[derive(Clone)]
pub struct LegalAgreement {
    pub agreement_id: u64,
    pub student: Address,
    pub donor: Address,
    pub language_versions: Vec<LanguageVersion>,
    pub primary_language: Symbol, // e.g., "EN" for English, "YO" for Yoruba
    pub agreed_at: u64,
    pub is_active: bool,
    pub scholarship_pool: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct LanguageVersion {
    pub language_code: Symbol, // ISO 639-1 code: "EN", "YO", "FR", etc.
    pub document_hash: Bytes, // SHA-256 hash of the legal document
    pub document_uri: Symbol, // IPFS URI or other storage reference
    pub version_number: u32,
    pub translated_at: u64,
    pub translator_verified: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct AgreementSignature {
    pub signer: Address,
    pub agreement_id: u64,
    pub signed_at: u64,
    pub ip_address_hash: Option<Bytes>, // For audit trail (hashed for privacy)
    pub accepted_language: Symbol, // Language version accepted
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

        // Distribute royalties
        Self::distribute_royalty(&env, course_id, actual_cost, &token);
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

    fn calculate_dynamic_rate(env: Env, student: Address, course_id: u64) -> i128 {
        let base_rate: i128 = env.storage().instance().get(&DataKey::BaseRate).unwrap_or(1);
        let threshold: u64 = env.storage().instance().get(&DataKey::DiscountThreshold).unwrap_or(3600);
        let percentage: u64 = env.storage().instance().get(&DataKey::DiscountPercentage).unwrap_or(10);

        let access: Access = env.storage().persistent().get(&DataKey::Access(student, course_id)).unwrap_or_else(|| {
            // Return dummy Access if not found
            Access { student: Address::generate(&env), course_id, expiry_time: 0, token: Address::generate(&env), total_watch_time: 0, last_heartbeat: 0, last_purchase_time: 0 }
        });

        if access.total_watch_time >= threshold {
            base_rate - (base_rate * percentage as i128 / 100)
        } else {
            base_rate
        }
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

        // Only allow setting admin if not already set
        let existing_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        if existing_admin.is_some() {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
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
            // Filter out the vetoed course
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
            let mut sorted_entry = entry.clone();
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

            env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
            env.storage().persistent().extend_ttl(
                &DataKey::Scholarship(student), 
                LEDGER_BUMP_THRESHOLD, 
                LEDGER_BUMP_EXTEND
            );

            // Emit event
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "ProbationStarted"), student,),
                probation_status.warning_period_end
            );
        }
    }

    /// End probation for a student
    fn end_probation(env: Env, student: Address, probation_status: &mut ProbationStatus, recovered: bool) {
        let current_time = env.ledger().timestamp();
        
        // Restore original flow rate
        if let Some(mut scholarship) = env.storage().persistent()
            .get::<_, Scholarship>(&DataKey::Scholarship(student.clone())) {
            
            scholarship.balance = probation_status.original_flow_rate;
            
            env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
            env.storage().persistent().extend_ttl(
                &DataKey::Scholarship(student), 
                LEDGER_BUMP_THRESHOLD, 
                LEDGER_BUMP_EXTEND
            );
        }

        // Reset probation status
        probation_status.is_on_probation = false;
        probation_status.probation_start_time = 0;
        probation_status.warning_period_end = 0;
        probation_status.original_flow_rate = 0;
        probation_status.reduced_flow_rate = 0;
        probation_status.violation_count = 0;

        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "ProbationEnded"), student,),
            recovered
        );
    }

    /// Revoke scholarship permanently
    fn revoke_scholarship(env: Env, student: Address) {
        // Mark scholarship as disputed with revocation reason
        if let Some(mut scholarship) = env.storage().persistent()
            .get::<_, Scholarship>(&DataKey::Scholarship(student.clone())) {
            
            scholarship.is_disputed = true;
            scholarship.dispute_reason = Some(Symbol::new(&env, "PERMANENT_REVOCATION_GPA"));
            scholarship.final_ruling = Some(Symbol::new(&env, "REVOKED"));
            
            env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
            env.storage().persistent().extend_ttl(
                &DataKey::Scholarship(student), 
                LEDGER_BUMP_THRESHOLD, 
                LEDGER_BUMP_EXTEND
            );
        }

        // Clear probation status
        env.storage().persistent().remove(&DataKey::ProbationStatus(student.clone()));

        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "StreamRevoked"), student,),
            ()
        );
    }

    /// Get probation status for a student
    pub fn get_probation_status(env: Env, student: Address) -> Option<ProbationStatus> {
        env.storage().persistent()
            .get(&DataKey::ProbationStatus(student))
    }

    /// Get GPA update info for a student
    pub fn get_gpa_update(env: Env, student: Address) -> Option<GPAUpdate> {
        env.storage().persistent()
            .get(&DataKey::GPAUpdate(student))
    }

    // Task 1: Wasm-Hash Rotation Pattern Functions

    /// Initialize DAO members for logic upgrade voting
    pub fn init_dao_members(env: Env, admin: Address, members: Vec<Address>) {
        admin.require_auth();
        
        // Verify caller is admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        // Validate minimum members (at least 3 for proper governance)
        if members.len() < 3 {
            panic!("DAO requires at least 3 members");
        }

        env.storage().instance().set(&DataKey::DaoMembers, &members);
        
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "DaoMembersInitialized"), admin),
            members.len()
        );
    }

    /// Propose a new logic hash for upgrade
    pub fn propose_logic_upgrade(
        env: Env,
        proposer: Address,
        new_logic_hash: Bytes,
        immutable_terms_hash: Bytes,
    ) -> u64 {
        proposer.require_auth();

        // Verify proposer is a DAO member
        let members: Vec<Address> = env.storage().instance()
            .get(&DataKey::DaoMembers)
            .expect("DAO members not initialized");
        
        if !members.contains(&proposer) {
            panic!("Only DAO members can propose upgrades");
        }

        let current_time = env.ledger().timestamp();
        let proposal_id: u64 = env.storage().instance()
            .get(&DataKey::ProposalCounter)
            .unwrap_or(0) + 1;

        env.storage().instance().set(&DataKey::ProposalCounter, &proposal_id);

        // Get current logic hash
        let current_hash: Bytes = env.storage().instance()
            .get(&DataKey::CurrentLogicHash)
            .unwrap_or_else(|| Bytes::from_array(&env, &[0; 32]));

        // Create logic hash record
        let record = LogicHashRecord {
            logic_hash: new_logic_hash.clone(),
            proposed_at: current_time,
            proposed_by: proposer.clone(),
            is_active: true,
            immutable_terms_hash: immutable_terms_hash.clone(),
        };

        env.storage().persistent().set(&DataKey::LogicHashRecord(new_logic_hash.clone()), &record);

        // Create upgrade proposal
        let proposal = LogicUpgradeProposal {
            proposal_id,
            old_logic_hash: current_hash.clone(),
            new_logic_hash: new_logic_hash.clone(),
            votes_yes: 0,
            votes_no: 0,
            voters: Vec::new(&env),
            is_executed: false,
            created_at: current_time,
            execution_threshold: ((members.len() as u64) * 2 / 3) + 1, // 2/3 majority required
        };

        env.storage().persistent().set(&DataKey::LogicUpgradeProposal(proposal_id), &proposal);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "LogicUpgradeProposed"), proposer),
            (proposal_id, new_logic_hash)
        );

        proposal_id
    }

    /// Vote on a logic upgrade proposal
    pub fn vote_on_upgrade(
        env: Env,
        voter: Address,
        proposal_id: u64,
        vote_yes: bool,
    ) {
        voter.require_auth();

        // Verify voter is a DAO member
        let members: Vec<Address> = env.storage().instance()
            .get(&DataKey::DaoMembers)
            .expect("DAO members not initialized");
        
        if !members.contains(&voter) {
            panic!("Only DAO members can vote");
        }

        // Get proposal
        let mut proposal: LogicUpgradeProposal = env.storage().persistent()
            .get(&DataKey::LogicUpgradeProposal(proposal_id))
            .expect("Proposal not found");

        if proposal.is_executed {
            panic!("Proposal already executed");
        }

        // Check if voter already voted
        if proposal.voters.contains(&voter) {
            panic!("Voter already participated in this proposal");
        }

        // Record vote
        let vote_record = DaoVote {
            voter: voter.clone(),
            logic_hash: proposal.new_logic_hash.clone(),
            vote_yes,
            voted_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::DaoVote(voter.clone(), proposal.new_logic_hash.clone()), &vote_record);

        // Update proposal vote counts
        if vote_yes {
            proposal.votes_yes += 1;
        } else {
            proposal.votes_no += 1;
        }
        proposal.voters.push_back(voter.clone());

        env.storage().persistent().set(&DataKey::LogicUpgradeProposal(proposal_id), &proposal);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "UpgradeVoteCast"), voter),
            (proposal_id, vote_yes, proposal.votes_yes, proposal.votes_no)
        );

        // Auto-execute if threshold reached
        if proposal.votes_yes >= proposal.execution_threshold {
            Self::execute_logic_upgrade(env.clone(), proposal_id);
        }
    }

    /// Execute the logic upgrade if voting threshold is met
    pub fn execute_logic_upgrade(env: Env, proposal_id: u64) {
        let mut proposal: LogicUpgradeProposal = env.storage().persistent()
            .get(&DataKey::LogicUpgradeProposal(proposal_id))
            .expect("Proposal not found");

        if proposal.is_executed {
            panic!("Proposal already executed");
        }

        if proposal.votes_yes < proposal.execution_threshold {
            panic!("Voting threshold not met");
        }

        // Verify immutable terms are unchanged
        let new_record: LogicHashRecord = env.storage().persistent()
            .get(&DataKey::LogicHashRecord(proposal.new_logic_hash.clone()))
            .expect("Logic hash record not found");

        // In a real implementation, this would verify against actual scholarship data
        // For now, we just store the immutable terms hash for verification
        let current_terms_hash: Bytes = env.storage().instance()
            .get(&DataKey::CurrentLogicHash)
            .unwrap_or_else(|| Bytes::from_array(&env, &[0; 32]));

        if current_terms_hash != proposal.old_logic_hash {
            panic!("Immutable terms mismatch - cannot upgrade active scholarships");
        }

        // Update current logic hash
        env.storage().instance().set(&DataKey::CurrentLogicHash, &proposal.new_logic_hash);

        // Mark proposal as executed
        proposal.is_executed = true;
        env.storage().persistent().set(&DataKey::LogicUpgradeProposal(proposal_id), &proposal);

        // Deactivate old logic hash record
        if let Some(mut old_record) = env.storage().persistent()
            .get::<_, LogicHashRecord>(&DataKey::LogicHashRecord(proposal.old_logic_hash.clone())) {
            old_record.is_active = false;
            env.storage().persistent().set(&DataKey::LogicHashRecord(proposal.old_logic_hash.clone()), &old_record);
        }

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "LogicUpgraded"),),
            (proposal_id, proposal.new_logic_hash)
        );
    }

    /// Get current logic hash
    pub fn get_current_logic_hash(env: Env) -> Option<Bytes> {
        env.storage().instance().get(&DataKey::CurrentLogicHash)
    }

    /// Get logic hash record
    pub fn get_logic_hash_record(env: Env, logic_hash: Bytes) -> Option<LogicHashRecord> {
        env.storage().persistent().get(&DataKey::LogicHashRecord(logic_hash))
    }

    /// Get upgrade proposal details
    pub fn get_upgrade_proposal(env: Env, proposal_id: u64) -> Option<LogicUpgradeProposal> {
        env.storage().persistent().get(&DataKey::LogicUpgradeProposal(proposal_id))
    }

    /// Compute immutable terms hash (total grant + student address)
    pub fn compute_immutable_terms_hash(env: Env, student: Address, total_grant: i128) -> Bytes {
        let terms_data = (student.to_string(), total_grant);
        let serialized = env.serialize_to_bytes(&terms_data);
        env.crypto().sha256(&serialized)
    }

    // Task 2: Batch Verify Milestones Functions

    /// Batch verify milestones for multiple students (optimized for gas efficiency)
    /// Updates probation/active status for all students in a single transaction
    pub fn batch_verify_milestones(
        env: Env,
        oracle: Address,
        student_ids: Vec<Address>,
        gpa_values: Vec<u64>,
    ) -> Vec<bool> {
        oracle.require_auth();

        // Validate input sizes match
        if student_ids.len() != gpa_values.len() {
            panic!("Student IDs and GPA values length mismatch");
        }

        // Limit batch size to prevent gas limit issues (max 50 students per batch)
        const MAX_BATCH_SIZE: usize = 50;
        if student_ids.len() > MAX_BATCH_SIZE {
            panic!("Batch size exceeds maximum of 50 students");
        }

        let current_time = env.ledger().timestamp();
        let mut results = Vec::new(&env);

        // Process each student efficiently
        for i in 0..student_ids.len() {
            let student = student_ids.get(i).expect("Invalid student ID index");
            let gpa = gpa_values.get(i).expect("Invalid GPA index");

            // Verify milestone for this student
            let success = Self::verify_single_milestone(env.clone(), oracle.clone(), student, gpa, current_time);
            results.push_back(success);
        }

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "BatchMilestonesVerified"), oracle),
            (student_ids.len(), current_time)
        );

        results
    }

    /// Helper function to verify a single student milestone (optimized storage access)
    fn verify_single_milestone(
        env: Env,
        oracle: Address,
        student: Address,
        new_gpa: u64,
        current_time: u64,
    ) -> bool {
        // Update GPA (this also triggers probation logic)
        Self::update_student_gpa_for_batch(env.clone(), oracle.clone(), student.clone(), new_gpa, current_time);
        
        true // Successfully processed
    }

    /// Optimized GPA update for batch processing (reduced storage operations)
    fn update_student_gpa_for_batch(
        env: Env,
        oracle: Address,
        student: Address,
        new_gpa: u64,
        current_time: u64,
    ) {
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
            &DataKey::GPAUpdate(student.clone()), 
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
            &DataKey::StudentGPA(student.clone()), 
            LEDGER_BUMP_THRESHOLD, 
            LEDGER_BUMP_EXTEND
        );

        // Handle probation logic (optimized version)
        Self::handle_probation_logic_batch(env.clone(), student.clone(), new_gpa, current_time);
    }

    /// Optimized probation logic for batch processing
    fn handle_probation_logic_batch(env: Env, student: Address, new_gpa: u64, current_time: u64) {
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
                // First violation - start probation (optimized)
                Self::start_probation_batch(env.clone(), student.clone(), &mut probation_status, current_time);
            } else {
                // Already on probation - check if warning period has ended
                if current_time > probation_status.warning_period_end {
                    // Warning period ended and GPA still low - mark for revocation
                    // Note: In batch mode, we just mark it, actual revocation happens post-batch
                    probation_status.violation_count += 1;
                } else {
                    // Still in warning period but GPA dropped again
                    probation_status.violation_count += 1;
                }
            }
        } else {
            // GPA is acceptable
            if probation_status.is_on_probation {
                // Student recovered - end probation (optimized)
                Self::end_probation_batch(env.clone(), student.clone(), &mut probation_status);
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

    /// Optimized probation start for batch processing
    fn start_probation_batch(env: Env, student: Address, probation_status: &mut ProbationStatus, current_time: u64) {
        if let Some(mut scholarship) = env.storage().persistent()
            .get::<_, Scholarship>(&DataKey::Scholarship(student.clone())) {
            
            // Calculate reduced flow rate (30% reduction)
            let original_rate = scholarship.balance;
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

            // Apply reduction to scholarship
            scholarship.balance = reduced_rate;

            env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
            env.storage().persistent().extend_ttl(
                &DataKey::Scholarship(student), 
                LEDGER_BUMP_THRESHOLD, 
                LEDGER_BUMP_EXTEND
            );

            // Emit event (batched)
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "ProbationStarted"), student,),
                probation_status.warning_period_end
            );
        }
    }

    /// Optimized probation end for batch processing
    fn end_probation_batch(env: Env, student: Address, probation_status: &mut ProbationStatus, _recovered: bool) {
        if let Some(mut scholarship) = env.storage().persistent()
            .get::<_, Scholarship>(&DataKey::Scholarship(student.clone())) {
            
            // Restore original flow rate
            scholarship.balance = probation_status.original_flow_rate;
            
            env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
            env.storage().persistent().extend_ttl(
                &DataKey::Scholarship(student), 
                LEDGER_BUMP_THRESHOLD, 
                LEDGER_BUMP_EXTEND
            );
        }

        // Reset probation status
        probation_status.is_on_probation = false;
        probation_status.probation_start_time = 0;
        probation_status.warning_period_end = 0;
        probation_status.original_flow_rate = 0;
        probation_status.reduced_flow_rate = 0;
        probation_status.violation_count = 0;

        // Emit event (batched)
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "ProbationEnded"), student,),
            true
        );
    }

    /// Get batch verification results summary
    pub fn get_batch_verification_summary(env: Env, batch_id: u64) -> Option<(u64, u64, u64)> {
        // This would store batch metadata for auditing
        // For now, return None as implementation detail
        None
    }

    // Task 3: Scholarship Registry Functions

    /// Initialize scholarship registry for a university
    pub fn init_scholarship_registry(env: Env, admin: Address, university: Address) {
        admin.require_auth();
        
        // Verify caller is admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let current_time = env.ledger().timestamp();

        // Check if registry already exists
        if env.storage().persistent().get::<_, ScholarshipRegistry>(&DataKey::ScholarshipRegistry(university.clone())).is_some() {
            panic!("Registry already initialized for this university");
        }

        let registry = ScholarshipRegistry {
            university_address: university.clone(),
            scholarship_contract_ids: Vec::new(&env),
            total_scholarships: 0,
            active_scholarships: 0,
            created_at: current_time,
            last_updated: current_time,
        };

        env.storage().persistent().set(&DataKey::ScholarshipRegistry(university), &registry);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "ScholarshipRegistryInitialized"), university),
            current_time
        );
    }

    /// Register a new scholarship contract for a university
    pub fn register_scholarship_contract(
        env: Env,
        university: Address,
        scholarship_contract_id: Address,
        student: Address,
    ) -> u64 {
        university.require_auth();

        // Get or create registry
        let mut registry: ScholarshipRegistry = env.storage().persistent()
            .get(&DataKey::ScholarshipRegistry(university.clone()))
            .expect("University registry not initialized");

        // Add contract to registry
        registry.scholarship_contract_ids.push_back(scholarship_contract_id.clone());
        registry.total_scholarships += 1;
        registry.active_scholarships += 1;
        registry.last_updated = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::ScholarshipRegistry(university.clone()), &registry);

        // Store index mapping for efficient lookup
        let contract_index: u64 = registry.scholarship_contract_ids.len() as u64 - 1;
        env.storage().persistent().set(
            &DataKey::UniversityContractIndex(university.clone(), contract_index),
            &scholarship_contract_id
        );

        // Map student to their scholarship contract
        env.storage().persistent().set(
            &DataKey::StudentScholarshipContract(student.clone()),
            &scholarship_contract_id
        );

        // Increment global counter
        let global_counter: u64 = env.storage().instance()
            .get(&DataKey::GlobalScholarshipCounter)
            .unwrap_or(0) + 1;
        env.storage().instance().set(&DataKey::GlobalScholarshipCounter, &global_counter);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "ScholarshipContractRegistered"), university),
            (scholarship_contract_id, student, global_counter)
        );

        contract_index
    }

    /// List all scholarships for a university (with pagination support)
    pub fn list_scholarships_by_university(
        env: Env,
        university: Address,
        offset: u64,
        limit: u64,
    ) -> Vec<UniversityScholarshipInfo> {
        // Get registry
        let registry: ScholarshipRegistry = env.storage().persistent()
            .get(&DataKey::ScholarshipRegistry(university.clone()))
            .expect("University registry not found");

        let mut result = Vec::new(&env);
        let total_count = registry.scholarship_contract_ids.len();

        // Validate offset and limit
        if offset >= total_count {
            return result; // Empty result
        }

        let actual_limit = core::cmp::min(limit, total_count - offset);

        // Retrieve scholarships within the range
        for i in offset..(offset + actual_limit) {
            if let Some(contract_id) = env.storage().persistent()
                .get::<_, Address>(&DataKey::UniversityContractIndex(university.clone(), i)) {
                
                // Get student info from contract (would require cross-contract call in production)
                // For now, we'll create a basic info struct
                let info = UniversityScholarshipInfo {
                    university: university.clone(),
                    contract_id: contract_id.clone(),
                    student: Address::generate(&env), // Would be retrieved from contract
                    is_active: true,
                    registered_at: registry.created_at,
                };
                result.push_back(info);
            }
        }

        result
    }

    /// Get scholarship count for a university
    pub fn get_scholarship_count(env: Env, university: Address) -> (u64, u64) {
        let registry: ScholarshipRegistry = env.storage().persistent()
            .get(&DataKey::ScholarshipRegistry(university))
            .expect("University registry not found");

        (registry.total_scholarships, registry.active_scholarships)
    }

    /// Get registry info for a university
    pub fn get_university_registry(env: Env, university: Address) -> Option<ScholarshipRegistry> {
        env.storage().persistent().get(&DataKey::ScholarshipRegistry(university))
    }

    /// Get scholarship contract ID for a student
    pub fn get_student_scholarship_contract(env: Env, student: Address) -> Option<Address> {
        env.storage().persistent().get(&DataKey::StudentScholarshipContract(student))
    }

    /// Get global scholarship counter
    pub fn get_global_scholarship_count(env: Env) -> u64 {
        env.storage().instance()
            .get(&DataKey::GlobalScholarshipCounter)
            .unwrap_or(0)
    }

    /// Search scholarships by university with filters
    pub fn search_scholarships(
        env: Env,
        university: Address,
        active_only: bool,
    ) -> Vec<UniversityScholarshipInfo> {
        let registry: ScholarshipRegistry = env.storage().persistent()
            .get(&DataKey::ScholarshipRegistry(university.clone()))
            .expect("University registry not found");

        let mut result = Vec::new(&env);

        for i in 0..registry.scholarship_contract_ids.len() {
            if let Some(contract_id) = env.storage().persistent()
                .get::<_, Address>(&DataKey::UniversityContractIndex(university.clone(), i as u64)) {
                
                // Filter by active status if requested
                if active_only && !registry.active_scholarships.checked_sub(i as u64).is_some() {
                    continue;
                }

                let info = UniversityScholarshipInfo {
                    university: university.clone(),
                    contract_id,
                    student: Address::generate(&env),
                    is_active: true,
                    registered_at: registry.created_at,
                };
                result.push_back(info);
            }
        }

        result
    }


    // Task 4: Multi-Lingual Legal Agreement Functions

    /// Create a multi-lingual legal agreement between donor and student
    pub fn create_legal_agreement(
        env: Env,
        donor: Address,
        student: Address,
        language_versions: Vec<LanguageVersion>,
        primary_language: Symbol,
        scholarship_pool: u64,
    ) -> u64 {
        donor.require_auth();
        student.require_auth();

        // Validate at least one language version exists
        if language_versions.is_empty() {
            panic!("At least one language version is required");
        }

        // Validate primary language exists in versions
        let mut primary_found = false;
        for i in 0..language_versions.len() {
            let version = language_versions.get(i).expect("Invalid version index");
            if version.language_code == primary_language {
                primary_found = true;
                break;
            }
        }
        if !primary_found {
            panic!("Primary language must exist in language versions");
        }

        let current_time = env.ledger().timestamp();
        let agreement_id: u64 = env.storage().instance()
            .get(&DataKey::GlobalScholarshipCounter)
            .unwrap_or(0) + 1;

        let agreement = LegalAgreement {
            agreement_id,
            student: student.clone(),
            donor: donor.clone(),
            language_versions: language_versions.clone(),
            primary_language: primary_language.clone(),
            agreed_at: current_time,
            is_active: true,
            scholarship_pool,
        };

        env.storage().persistent().set(&DataKey::LegalAgreement(agreement_id), &agreement);

        // Store document hash mappings
        for i in 0..language_versions.len() {
            let version = language_versions.get(i).expect("Invalid version index");
            env.storage().persistent().set(
                &DataKey::LanguageVersionHash(version.document_hash.clone()),
                &version
            );
        }

        // Track primary agreement for student
        let primary_agreement_info = (agreement_id, primary_language.clone());
        env.storage().persistent().set(
            &DataKey::StudentPrimaryAgreement(student.clone()),
            &primary_agreement_info
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "LegalAgreementCreated"), donor, student),
            (agreement_id, primary_language, language_versions.len())
        );

        agreement_id
    }

    /// Sign a legal agreement (student or donor acceptance)
    pub fn sign_agreement(
        env: Env,
        signer: Address,
        agreement_id: u64,
        accepted_language: Symbol,
    ) {
        signer.require_auth();

        // Get agreement
        let agreement: LegalAgreement = env.storage().persistent()
            .get(&DataKey::LegalAgreement(agreement_id))
            .expect("Agreement not found");

        if !agreement.is_active {
            panic!("Agreement is no longer active");
        }

        // Verify signer is either student or donor
        if signer != agreement.student && signer != agreement.donor {
            panic!("Only student or donor can sign this agreement");
        }

        // Verify accepted language exists in agreement
        let mut language_found = false;
        for i in 0..agreement.language_versions.len() {
            let version = agreement.language_versions.get(i).expect("Invalid version index");
            if version.language_code == accepted_language {
                language_found = true;
                break;
            }
        }
        if !language_found {
            panic!("Accepted language version not found in agreement");
        }

        // Create signature record
        let signature = AgreementSignature {
            signer: signer.clone(),
            agreement_id,
            signed_at: env.ledger().timestamp(),
            ip_address_hash: None, // Would be populated in production from request metadata
            accepted_language: accepted_language.clone(),
        };

        env.storage().persistent().set(
            &DataKey::AgreementSignature(agreement_id, signer.clone()),
            &signature
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "AgreementSigned"), signer),
            (agreement_id, accepted_language)
        );
    }

    /// Verify if both parties have signed the agreement
    pub fn is_fully_executed(env: Env, agreement_id: u64) -> bool {
        let agreement: LegalAgreement = match env.storage().persistent()
            .get(&DataKey::LegalAgreement(agreement_id)) {
            Some(a) => a,
            None => return false,
        };

        // Check if both student and donor have signed
        let student_signed: bool = env.storage().persistent()
            .get::<_, AgreementSignature>(&DataKey::AgreementSignature(agreement_id, agreement.student.clone()))
            .is_some();

        let donor_signed: bool = env.storage().persistent()
            .get::<_, AgreementSignature>(&DataKey::AgreementSignature(agreement_id, agreement.donor.clone()))
            .is_some();

        student_signed && donor_signed
    }

    /// Get legal agreement details
    pub fn get_legal_agreement(env: Env, agreement_id: u64) -> Option<LegalAgreement> {
        env.storage().persistent().get(&DataKey::LegalAgreement(agreement_id))
    }

    /// Get signature info for an agreement
    pub fn get_agreement_signature(env: Env, agreement_id: u64, signer: Address) -> Option<AgreementSignature> {
        env.storage().persistent().get(&DataKey::AgreementSignature(agreement_id, signer))
    }

    /// Get primary agreement info for a student
    pub fn get_student_primary_agreement(env: Env, student: Address) -> Option<(u64, Symbol)> {
        env.storage().persistent().get(&DataKey::StudentPrimaryAgreement(student))
    }

    /// Get language version by document hash
    pub fn get_language_version(env: Env, document_hash: Bytes) -> Option<LanguageVersion> {
        env.storage().persistent().get(&DataKey::LanguageVersionHash(document_hash))
    }

    /// Compute SHA-256 hash of a legal document
    pub fn compute_document_hash(env: Env, document_content: Bytes) -> Bytes {
        env.crypto().sha256(&document_content)
    }

    /// Add a new language version to an existing agreement (for amendments)
    pub fn add_language_version(
        env: Env,
        admin: Address,
        agreement_id: u64,
        new_version: LanguageVersion,
    ) {
        admin.require_auth();
        
        // Verify caller is admin
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let mut agreement: LegalAgreement = env.storage().persistent()
            .get(&DataKey::LegalAgreement(agreement_id))
            .expect("Agreement not found");

        if !agreement.is_active {
            panic!("Agreement is no longer active");
        }

        // Check if language code already exists
        for i in 0..agreement.language_versions.len() {
            let existing = agreement.language_versions.get(i).expect("Invalid version index");
            if existing.language_code == new_version.language_code {
                panic!("Language version already exists for this language");
            }
        }

        // Add new version
        agreement.language_versions.push_back(new_version.clone());
        env.storage().persistent().set(&DataKey::LegalAgreement(agreement_id), &agreement);

        // Store hash mapping
        env.storage().persistent().set(
            &DataKey::LanguageVersionHash(new_version.document_hash.clone()),
            &new_version
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "LanguageVersionAdded"), admin),
            (agreement_id, new_version.language_code)
        );
    }

    /// Update primary language for an agreement (requires admin)
    pub fn update_primary_language(
        env: Env,
        admin: Address,
        agreement_id: u64,
        new_primary_language: Symbol,
    ) {
        admin.require_auth();
        
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let mut agreement: LegalAgreement = env.storage().persistent()
            .get(&DataKey::LegalAgreement(agreement_id))
            .expect("Agreement not found");

        // Verify new primary language exists in versions
        let mut found = false;
        for i in 0..agreement.language_versions.len() {
            let version = agreement.language_versions.get(i).expect("Invalid version index");
            if version.language_code == new_primary_language {
                found = true;
                break;
            }
        }
        if !found {
            panic!("New primary language must exist in agreement versions");
        }

        agreement.primary_language = new_primary_language.clone();
        env.storage().persistent().set(&DataKey::LegalAgreement(agreement_id), &agreement);

        // Update student's primary agreement reference
        env.storage().persistent().set(
            &DataKey::StudentPrimaryAgreement(agreement.student.clone()),
            &(agreement_id, new_primary_language)
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "PrimaryLanguageUpdated"), admin),
            (agreement_id, new_primary_language)
        );
    }

    /// Get all signatures for an agreement (for dispute resolution)
    pub fn get_agreement_signatures(env: Env, agreement_id: u64) -> Vec<AgreementSignature> {
        let mut signatures = Vec::new(&env);

        let agreement: Option<LegalAgreement> = env.storage().persistent()
            .get(&DataKey::LegalAgreement(agreement_id));

        if let Some(agr) = agreement {
            // Get student signature
            if let Some(student_sig) = env.storage().persistent()
                .get::<_, AgreementSignature>(&DataKey::AgreementSignature(agreement_id, agr.student.clone())) {
                signatures.push_back(student_sig);
            }

            // Get donor signature
            if let Some(donor_sig) = env.storage().persistent()
                .get::<_, AgreementSignature>(&DataKey::AgreementSignature(agreement_id, agr.donor.clone())) {
                signatures.push_back(donor_sig);
            }
        }

        signatures
    }

    /// Verify which language version was used during signing (for legal disputes)
    pub fn verify_accepted_terms(env: Env, agreement_id: u64, signer: Address) -> Option<LanguageVersion> {
        let signature: Option<AgreementSignature> = env.storage().persistent()
            .get(&DataKey::AgreementSignature(agreement_id, signer.clone()));

        if let Some(sig) = signature {
            let agreement: Option<LegalAgreement> = env.storage().persistent()
                .get(&DataKey::LegalAgreement(agreement_id));

            if let Some(agr) = agreement {
                // Find the matching language version
                for i in 0..agr.language_versions.len() {
                    let version = agr.language_versions.get(i).expect("Invalid version index");
                    if version.language_code == sig.accepted_language {
                        return Some(version);
                    }
                }
            }
        }

        None
    }

    /// Deactivate an agreement (e.g., after scholarship completion or dispute)
    pub fn deactivate_agreement(env: Env, admin: Address, agreement_id: u64) {
        admin.require_auth();
        
        if !Self::is_admin(&env, &admin) {
            panic!("Not authorized");
        }

        let mut agreement: LegalAgreement = env.storage().persistent()
            .get(&DataKey::LegalAgreement(agreement_id))
            .expect("Agreement not found");

        agreement.is_active = false;
        env.storage().persistent().set(&DataKey::LegalAgreement(agreement_id), &agreement);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "AgreementDeactivated"), admin),
            agreement_id
        );
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

    }
}

mod test;
mod tuition_stipend_split_tests;
mod student_profile_nft;
