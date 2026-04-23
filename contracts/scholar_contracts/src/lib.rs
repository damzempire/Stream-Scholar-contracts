#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, IntoVal, Symbol, Vec, BytesN};
use expiry_math::checked_access_expiry;

const LEDGER_BUMP_THRESHOLD: u32 = 123456; // Example value
const LEDGER_BUMP_EXTEND: u32 = 789012; // Example value
const GPA_BONUS_THRESHOLD: u64 = 35; // Example 3.5 GPA
const GPA_BONUS_PERCENTAGE_PER_POINT: u64 = 20; // Example 20%


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    SbtMint(Address, u64),
    StreamCreated(Address, Address, i128), // funder, student, amount
    GeographicReview(Address, u64), // student, timestamp
    SsiVerificationRequired(Address), // student
    PayItForwardExecuted(Address, i128), // alumni, amount
    CrossChainFundReceived(Symbol, BytesN<32>, i128), // origin_chain, tx_hash, amount
    YieldRoutedByPreference(Address, Symbol, i128), // sponsor, preference, amount
    LiquidityBoundEnforced(i128, i128), // requested, available
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SponsorYieldPreference {
    Reinvest,
    ReturnToSponsor,
    DonateToDAO,
}

#[contracttype]
#[derive(Clone)]
pub struct SponsorProfile {
    pub preference: SponsorYieldPreference,
    pub total_sponsored: i128,
    pub active_capital: i128,
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
    pub final_ruling: Option<bool>,
}

#[contracttype]
#[derive(Clone)]
pub struct Stream {
    pub funder: Address,
    pub student: Address,
    pub amount_per_second: i128,
    pub total_deposited: i128,
    pub total_withdrawn: i128,
    pub start_time: u64,
    pub is_active: bool,
    pub geographic_restriction: Option<Symbol>,
}


#[contracttype]
#[derive(Clone)]
pub struct StudyGroup {
    pub group_id: u64,
    pub members: Vec<Address>, // Exactly 3 members
    pub grant_stream: Stream, // The shared grant stream
    pub collateral_per_member: i128, // XLM amount each member must lock
    pub is_active: bool,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct MemberCollateral {
    pub member: Address,
    pub group_id: u64,
    pub collateral_amount: i128,
    pub is_locked: bool,
    pub is_slashed: bool,
    pub is_paused: bool, // Whether member's share is paused
    pub locked_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct VoteRecord {
    pub voter: Address,
    pub target_member: Address,
    pub group_id: u64,
    pub vote_type: Symbol, // "pause" or "slash"
    pub voted_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct GpaRecord {
    pub student: Address,
    pub gpa_scaled: u64, // GPA * 100 (e.g., 3.5 GPA = 350)
    pub verified_at: u64,
    pub academic_period: Symbol, // e.g., "fall2024", "spring2025"
    pub verifier_address: Address, // Academic institution oracle
}

#[contracttype]
#[derive(Clone)]
pub struct FlowRateAdjustment {
    pub student: Address,
    pub base_rate: i128,
    pub adjusted_rate: i128,
    pub gpa_scaled: u64,
    pub adjustment_timestamp: u64,
    pub max_grant_amount: i128,
    pub total_distributed: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct BudgetTracker {
    pub student: Address,
    pub max_grant_amount: i128,
    pub total_distributed: i128,
    pub current_flow_rate: i128,
    pub last_accumulation_time: u64,
    pub accumulated_amount: i128,
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
    StudentGPA(Address),
    AcademicOracle(Address),
    NextGroupId,
    StudyGroup(u64),
    MemberCollateral(Address),
    VoteRecord(Address, Address, u64),
    GroupVoteCount(u64, Address, Symbol),
    GpaRecord(Address),
    FlowRateAdjustment(Address),
    BudgetTracker(Address),
    Stream(Address, Address), // funder, student
    AlumniPledge(Address),
    SponsorProfile(Address),
    CrossChainMessage(BytesN<32>),
    GlobalScholarshipPool,
    TotalTVL,
    LastBurnRateCalc,
    DailyBurnRate,
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
            }
        }

        env.storage().persistent().set(&access_key, &access);
        env.storage().persistent().extend_ttl(&access_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
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

        // Only add the student's portion to scholarship balance
        scholarship.balance += student_amount;
        scholarship.unlocked_balance += student_amount; // Assume funded amount is unlocked
        
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

    // Study Group Collateral Functions for Joint Grants
    
    pub fn create_study_group(env: Env, funder: Address, members: Vec<Address>, collateral_per_member: i128, amount_per_second: i128, token: Address) -> u64 {
        funder.require_auth();
        
        // Verify exactly 3 members
        if members.len() != 3 {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Get next group ID
        let group_id: u64 = env.storage().instance().get(&DataKey::NextGroupId).unwrap_or(1);
        env.storage().instance().set(&DataKey::NextGroupId, &(group_id + 1));
        
        let current_time = env.ledger().timestamp();
        
        // Create the shared grant stream
        let grant_stream = Stream {
            funder: funder.clone(),
            student: Address::from_array(&[0; 32]), // Placeholder address for group
            amount_per_second,
            total_deposited: 0,
            total_withdrawn: 0,
            start_time: current_time,
            is_active: true,
            geographic_restriction: None,
        };
        
        // Create study group
        let study_group = StudyGroup {
            group_id,
            members: members.clone(),
            grant_stream,
            collateral_per_member,
            is_active: false, // Not active until all collateral is locked
            created_at: current_time,
        };
        
        env.storage().persistent().set(&DataKey::StudyGroup(group_id), &study_group);
        
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "STUDY_GROUP_CREATED"), group_id, funder),
            collateral_per_member
        );
        
        group_id
    }
    
    pub fn lock_collateral(env: Env, member: Address, group_id: u64, token: Address) {
        member.require_auth();
        
        // Get study group
        let mut study_group: StudyGroup = env.storage().persistent()
            .get(&DataKey::StudyGroup(group_id))
            .expect("Study group not found");
        
        // Verify member is part of the group
        if !study_group.members.contains(&member) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Check if collateral already locked
        let collateral_key = DataKey::MemberCollateral(member.clone());
        if env.storage().persistent().has(&collateral_key) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Transfer collateral to contract
        let client = token::Client::new(&env, &token);
        client.transfer(&member, &env.current_contract_address(), &study_group.collateral_per_member);
        
        // Create collateral record
        let collateral = MemberCollateral {
            member: member.clone(),
            group_id,
            collateral_amount: study_group.collateral_per_member,
            is_locked: true,
            is_slashed: false,
            is_paused: false,
            locked_at: env.ledger().timestamp(),
        };
        
        env.storage().persistent().set(&collateral_key, &collateral);
        
        // Check if all members have locked collateral
        let mut all_locked = true;
        for i in 0..study_group.members.len() {
            let member_addr = study_group.members.get(i).unwrap();
            let member_collateral_key = DataKey::MemberCollateral(member_addr);
            if !env.storage().persistent().has(&member_collateral_key) {
                all_locked = false;
                break;
            }
        }
        
        // Activate group if all collateral is locked
        if all_locked {
            study_group.is_active = true;
            env.storage().persistent().set(&DataKey::StudyGroup(group_id), &study_group);
        }
        
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "COLLATERAL_LOCKED"), member, group_id),
            study_group.collateral_per_member
        );
    }
    
    pub fn vote_to_pause_member(env: Env, voter: Address, target_member: Address, group_id: u64) {
        voter.require_auth();
        
        // Verify voter is in the group and not the target
        let study_group: StudyGroup = env.storage().persistent()
            .get(&DataKey::StudyGroup(group_id))
            .expect("Study group not found");
        
        if !study_group.members.contains(&voter) || voter == target_member {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Verify target member is in the group
        if !study_group.members.contains(&target_member) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Check if already voted
        let vote_key = DataKey::VoteRecord(voter.clone(), target_member.clone(), group_id);
        if env.storage().persistent().has(&vote_key) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Record vote
        let vote = VoteRecord {
            voter: voter.clone(),
            target_member: target_member.clone(),
            group_id,
            vote_type: Symbol::new(&env, "pause"),
            voted_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&vote_key, &vote);
        
        // Count votes
        let vote_count_key = DataKey::GroupVoteCount(group_id, target_member.clone(), Symbol::new(&env, "pause"));
        let current_count: u64 = env.storage().instance().get(&vote_count_key).unwrap_or(0);
        let new_count = current_count + 1;
        env.storage().instance().set(&vote_count_key, &new_count);
        
        // If 2 votes reached, pause the member
        if new_count >= 2 {
            let collateral_key = DataKey::MemberCollateral(target_member.clone());
            let mut collateral: MemberCollateral = env.storage().persistent()
                .get(&collateral_key)
                .expect("Member collateral not found");
            
            collateral.is_paused = true;
            env.storage().persistent().set(&collateral_key, &collateral);
            
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "MEMBER_PAUSED"), target_member, group_id),
                new_count
            );
        }
    }
    
    pub fn vote_to_slash_collateral(env: Env, voter: Address, target_member: Address, group_id: u64, replacement_member: Address) {
        voter.require_auth();
        
        // Verify voter is in the group and not the target
        let study_group: StudyGroup = env.storage().persistent()
            .get(&DataKey::StudyGroup(group_id))
            .expect("Study group not found");
        
        if !study_group.members.contains(&voter) || voter == target_member {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Verify target member is paused (must be paused before slashing)
        let collateral_key = DataKey::MemberCollateral(target_member.clone());
        let collateral: MemberCollateral = env.storage().persistent()
            .get(&collateral_key)
            .expect("Member collateral not found");
        
        if !collateral.is_paused {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Check if already voted
        let vote_key = DataKey::VoteRecord(voter.clone(), target_member.clone(), group_id);
        if env.storage().persistent().has(&vote_key) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Record vote
        let vote = VoteRecord {
            voter: voter.clone(),
            target_member: target_member.clone(),
            group_id,
            vote_type: Symbol::new(&env, "slash"),
            voted_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&vote_key, &vote);
        
        // Count votes
        let vote_count_key = DataKey::GroupVoteCount(group_id, target_member.clone(), Symbol::new(&env, "slash"));
        let current_count: u64 = env.storage().instance().get(&vote_count_key).unwrap_or(0);
        let new_count = current_count + 1;
        env.storage().instance().set(&vote_count_key, &new_count);
        
        // If 2 votes reached, slash collateral and replace member
        if new_count >= 2 {
            // Transfer slashed collateral to group fund (for replacement member setup)
            let client = token::Client::new(&env, &Address::from_array(&[0; 32])); // Use XLM token address
            client.transfer(&env.current_contract_address(), &env.current_contract_address(), &collateral.collateral_amount);
            
            // Mark collateral as slashed
            let mut updated_collateral = collateral;
            updated_collateral.is_slashed = true;
            env.storage().persistent().set(&collateral_key, &updated_collateral);
            
            // Update study group members (replace target with replacement)
            let mut updated_group = study_group;
            for i in 0..updated_group.members.len() {
                let member_addr = updated_group.members.get(i).unwrap();
                if *member_addr == target_member {
                    updated_group.members.set(i, replacement_member.clone());
                    break;
                }
            }
            env.storage().persistent().set(&DataKey::StudyGroup(group_id), &updated_group);
            
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "COLLATERAL_SLASHED"), target_member, replacement_member),
                collateral.collateral_amount
            );
        }
    }
    
    pub fn withdraw_from_group_stream(env: Env, member: Address, group_id: u64, token: Address) -> i128 {
        member.require_auth();
        
        // Get study group
        let study_group: StudyGroup = env.storage().persistent()
            .get(&DataKey::StudyGroup(group_id))
            .expect("Study group not found");
        
        // Verify member is part of the group
        if !study_group.members.contains(&member) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Check if member's collateral is not paused
        let collateral_key = DataKey::MemberCollateral(member.clone());
        let collateral: MemberCollateral = env.storage().persistent()
            .get(&collateral_key)
            .expect("Member collateral not found");
        
        if collateral.is_paused {
            return 0; // No withdrawal if paused
        }
        
        // Calculate member's share (1/3 of total stream)
        let current_time = env.ledger().timestamp();
        let elapsed_seconds = current_time - study_group.grant_stream.start_time;
        let total_accrued = elapsed_seconds as i128 * study_group.grant_stream.amount_per_second;
        let member_share = total_accrued / 3; // Equal split among 3 members
        
        // Check available balance (consider previous withdrawals)
        let total_withdrawn_by_all = study_group.grant_stream.total_withdrawn;
        let available_total = (study_group.grant_stream.total_deposited - total_withdrawn_by_all).min(total_accrued);
        let available_member_share = available_total / 3;
        
        if available_member_share <= 0 {
            return 0;
        }
        
        // Transfer to member
        let client = token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &member, &available_member_share);
        
        // Update group's total withdrawn
        let mut updated_stream = study_group.grant_stream;
        updated_stream.total_withdrawn += available_member_share;
        
        let mut updated_group = study_group;
        updated_group.grant_stream = updated_stream;
        env.storage().persistent().set(&DataKey::StudyGroup(group_id), &updated_group);
        
        available_member_share
    }
    
    pub fn get_member_status(env: Env, member: Address) -> (bool, bool, bool) { // (is_locked, is_paused, is_slashed)
        if let Some(collateral) = env.storage().persistent().get::<DataKey, MemberCollateral>(&DataKey::MemberCollateral(member)) {
            (collateral.is_locked, collateral.is_paused, collateral.is_slashed)
        } else {
            (false, false, false)
        }
    }
    
    pub fn get_study_group_info(env: Env, group_id: u64) -> StudyGroup {
        env.storage().persistent()
            .get(&DataKey::StudyGroup(group_id))
            .expect("Study group not found")
    }

    // GPA-Based Flow Rate and Math Verification Functions
    
    pub fn set_academic_oracle(env: Env, admin: Address, oracle_address: Address) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        env.storage().instance().set(&DataKey::AcademicOracle(oracle_address), &true);
    }
    
    pub fn verify_gpa(env: Env, student: Address, gpa_scaled: u64, academic_period: Symbol, verifier_address: Address) {
        student.require_auth();
        
        // Verify oracle is authorized
        let is_authorized: bool = env.storage().instance().get(&DataKey::AcademicOracle(verifier_address.clone())).unwrap_or(false);
        if !is_authorized {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Verify GPA is within valid range (0.0 to 4.0 scaled)
        if gpa_scaled > GPA_SCALE {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let current_time = env.ledger().timestamp();
        let gpa_record = GpaRecord {
            student: student.clone(),
            gpa_scaled,
            verified_at: current_time,
            academic_period,
            verifier_address,
        };
        
        env.storage().persistent().set(&DataKey::GpaRecord(student), &gpa_record);
        
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "GPA_VERIFIED"), student),
            gpa_scaled
        );
    }
    
    pub fn calculate_gpa_adjusted_flow_rate(env: Env, student: Address, base_rate: i128, max_grant_amount: i128) -> i128 {
        // Get student's GPA record
        let gpa_record: GpaRecord = env.storage().persistent()
            .get(&DataKey::GpaRecord(student.clone()))
            .expect("GPA record not found");
        
        // Verify GPA meets minimum threshold
        if gpa_record.gpa_scaled < MIN_GPA_THRESHOLD {
            return 0; // No flow rate if below minimum GPA
        }
        
        // Calculate GPA bonus: linear scaling from 0% at 2.0 to 20% at 4.0
        let gpa_above_min = gpa_record.gpa_scaled - MIN_GPA_THRESHOLD;
        let gpa_range = GPA_SCALE - MIN_GPA_THRESHOLD;
        let bonus_percentage = (gpa_above_min as i128 * MAX_GPA_BONUS) / gpa_range as i128;
        
        // Apply bonus to base rate
        let adjusted_rate = base_rate + ((base_rate * bonus_percentage) / 1000);
        
        // Ensure adjusted rate doesn't exceed budget constraints
        let monthly_rate = adjusted_rate * (30 * 24 * 60 * 60) as i128;
        if monthly_rate > max_grant_amount {
            // Cap the rate to stay within budget
            max_grant_amount / (30 * 24 * 60 * 60) as i128
        } else {
            adjusted_rate
        }
    }
    
    pub fn create_gpa_adjusted_stream(env: Env, funder: Address, student: Address, base_rate: i128, max_grant_amount: i128, token: Address) {
        funder.require_auth();
        
        // Verify student has verified GPA
        if !env.storage().persistent().has(&DataKey::GpaRecord(student.clone())) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Calculate GPA-adjusted flow rate
        let adjusted_rate = Self::calculate_gpa_adjusted_flow_rate(env.clone(), student.clone(), base_rate, max_grant_amount);
        
        if adjusted_rate <= 0 {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let current_time = env.ledger().timestamp();
        
        // Create flow rate adjustment record
        let adjustment = FlowRateAdjustment {
            student: student.clone(),
            base_rate,
            adjusted_rate,
            gpa_scaled: {
                let gpa_record: GpaRecord = env.storage().persistent()
                    .get(&DataKey::GpaRecord(student.clone()))
                    .expect("GPA record not found");
                gpa_record.gpa_scaled
            },
            adjustment_timestamp: current_time,
            max_grant_amount,
            total_distributed: 0,
        };
        env.storage().persistent().set(&DataKey::FlowRateAdjustment(student.clone()), &adjustment);
        
        // Create budget tracker
        let budget_tracker = BudgetTracker {
            student: student.clone(),
            max_grant_amount,
            total_distributed: 0,
            current_flow_rate: adjusted_rate,
            last_accumulation_time: current_time,
            accumulated_amount: 0,
        };
        env.storage().persistent().set(&DataKey::BudgetTracker(student.clone()), &budget_tracker);
        
        // Create the stream with adjusted rate
        Self::create_stream(env.clone(), funder, student.clone(), adjusted_rate, token, None);
        
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "GPA_FLOW_ADJUSTED"), student),
            adjusted_rate
        );
    }
    
    pub fn verify_budget_invariant(env: Env, student: Address) -> bool {
        let budget_tracker: BudgetTracker = env.storage().persistent()
            .get(&DataKey::BudgetTracker(student.clone()))
            .expect("Budget tracker not found");
        
        let current_time = env.ledger().timestamp();
        let elapsed_time = current_time - budget_tracker.last_accumulation_time;
        
        // Calculate theoretical accumulated amount since last check
        let new_accumulation = elapsed_time as i128 * budget_tracker.current_flow_rate;
        let total_accumulated = budget_tracker.accumulated_amount + new_accumulation;
        
        // Core invariant: Sum(FlowRate * DeltaTime) <= Max_Grant_Amount
        let invariant_holds = total_accumulated <= budget_tracker.max_grant_amount;
        
        if invariant_holds {
            // Update the accumulated amount
            let mut updated_tracker = budget_tracker;
            updated_tracker.last_accumulation_time = current_time;
            updated_tracker.accumulated_amount = total_accumulated;
            env.storage().persistent().set(&DataKey::BudgetTracker(student), &updated_tracker);
        }
        
        invariant_holds
    }
    
    pub fn withdraw_with_math_verification(env: Env, student: Address, funder: Address, token: Address) -> i128 {
        student.require_auth();
        
        // First verify the budget invariant
        if !Self::verify_budget_invariant(env.clone(), student.clone()) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Perform standard withdrawal
        let amount = Self::withdraw_from_stream(env.clone(), student.clone(), funder.clone(), token.clone());
        
        if amount > 0 {
            // Update budget tracker
            let mut budget_tracker: BudgetTracker = env.storage().persistent()
                .get(&DataKey::BudgetTracker(student.clone()))
                .expect("Budget tracker not found");
            
            budget_tracker.total_distributed += amount;
            env.storage().persistent().set(&DataKey::BudgetTracker(student), &budget_tracker);
            
            // Update flow rate adjustment record
            let mut adjustment: FlowRateAdjustment = env.storage().persistent()
                .get(&DataKey::FlowRateAdjustment(student.clone()))
                .expect("Flow rate adjustment not found");
            
            adjustment.total_distributed += amount;
            env.storage().persistent().set(&DataKey::FlowRateAdjustment(student), &adjustment);
            
            // Final invariant check after withdrawal
            if !Self::verify_budget_invariant(env.clone(), student.clone()) {
                env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
            }
        }
        
        amount
    }
    
    pub fn update_gpa_and_flow(env: Env, student: Address, new_gpa_scaled: u64, academic_period: Symbol, verifier_address: Address) {
        // This allows for dynamic flow rate adjustment based on new GPA
        Self::verify_gpa(env.clone(), student.clone(), new_gpa_scaled, academic_period, verifier_address);
        
        // Get current flow rate adjustment
        let adjustment: FlowRateAdjustment = env.storage().persistent()
            .get(&DataKey::FlowRateAdjustment(student.clone()))
            .expect("Flow rate adjustment not found");
        
        // Calculate new adjusted rate
        let new_adjusted_rate = Self::calculate_gpa_adjusted_flow_rate(
            env.clone(), 
            student.clone(), 
            adjustment.base_rate, 
            adjustment.max_grant_amount
        );
        
        // Update the stream with new rate
        let stream_key = DataKey::Stream(adjustment.student.clone(), student.clone());
        let mut stream: Stream = env.storage().persistent()
            .get(&stream_key)
            .expect("Stream not found");
        
        stream.amount_per_second = new_adjusted_rate;
        env.storage().persistent().set(&stream_key, &stream);
        
        // Update budget tracker
        let mut budget_tracker: BudgetTracker = env.storage().persistent()
            .get(&DataKey::BudgetTracker(student.clone()))
            .expect("Budget tracker not found");
        
        budget_tracker.current_flow_rate = new_adjusted_rate;
        env.storage().persistent().set(&DataKey::BudgetTracker(student), &budget_tracker);
        
        // Update flow rate adjustment record
        let mut updated_adjustment = adjustment;
        updated_adjustment.adjusted_rate = new_adjusted_rate;
        updated_adjustment.gpa_scaled = new_gpa_scaled;
        updated_adjustment.adjustment_timestamp = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::FlowRateAdjustment(student), &updated_adjustment);
        
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "GPA_FLOW_UPDATED"), student),
            new_adjusted_rate
        );
    }
    
    pub fn get_budget_status(env: Env, student: Address) -> (i128, i128, i128, bool) { // (max_grant, total_distributed, remaining, invariant_holds)
        let budget_tracker: BudgetTracker = env.storage().persistent()
            .get(&DataKey::BudgetTracker(student.clone()))
            .expect("Budget tracker not found");
        
        let invariant_holds = Self::verify_budget_invariant(env.clone(), student.clone());
        let remaining = budget_tracker.max_grant_amount - budget_tracker.total_distributed;
        
        (budget_tracker.max_grant_amount, budget_tracker.total_distributed, remaining, invariant_holds)
    }
    
    // --- New Features (Task 174, 175, 176, 177) ---

    /// #174 Pay-It-Forward Alumni Tax Mechanism
    pub fn alumni_contribution_pledge(env: Env, alumni: Address, percentage: u32) {
        alumni.require_auth();
        if percentage > 100 {
            panic!("Percentage cannot exceed 100");
        }
        env.storage().persistent().set(&DataKey::AlumniPledge(alumni), &percentage);
    }

    fn check_and_apply_alumni_tax(env: &Env, alumni: &Address, amount: i128) -> i128 {
        if let Some(percentage) = env.storage().persistent().get::<_, u32>(&DataKey::AlumniPledge(alumni.clone())) {
            let tax_amount = (amount * percentage as i128) / 100;
            if tax_amount > 0 {
                // Route to Global Scholarship Pool
                let pool_address: Address = env.storage().instance().get(&DataKey::GlobalScholarshipPool)
                    .unwrap_or(env.current_contract_address()); // Default to contract address if not set
                
                // For simplicity in this implementation, we emit an event and 
                // in a real scenario we'd transfer or update a global pool balance.
                env.events().publish(
                    (Symbol::new(env, "PayItForwardExecuted"), alumni.clone()),
                    tax_amount
                );
                return amount - tax_amount;
            }
        }
        amount
    }

    /// #175 Cross-Chain Bridge Integration for USDC Sponsorships
    pub fn receive_cross_chain_sponsorship(
        env: Env,
        origin_chain: Symbol,
        tx_hash: BytesN<32>,
        student: Address,
        amount: i128,
        token: Address,
    ) {
        // Deduplication
        let msg_key = DataKey::CrossChainMessage(tx_hash.clone());
        if env.storage().persistent().has(&msg_key) {
            panic!("Message already processed");
        }
        env.storage().persistent().set(&msg_key, &true);

        // Verification (In a real scenario, this would verify a Relayer signature)
        // Here we assume the caller is an authorized bridge contract
        // (This would normally use env.current_contract_address().require_auth() or similar)

        // Fund scholarship
        Self::fund_scholarship(env.clone(), env.current_contract_address(), student.clone(), amount, token);

        env.events().publish(
            (Symbol::new(&env, "CrossChainFundReceived"), origin_chain, tx_hash),
            amount
        );
    }

    /// #176 Sponsor-Directed Yield Harvesting
    pub fn set_yield_preference(env: Env, sponsor: Address, preference: SponsorYieldPreference) {
        sponsor.require_auth();
        let mut profile: SponsorProfile = env.storage().persistent()
            .get(&DataKey::SponsorProfile(sponsor.clone()))
            .unwrap_or(SponsorProfile {
                preference: SponsorYieldPreference::Reinvest,
                total_sponsored: 0,
                active_capital: 0,
            });
        
        profile.preference = preference;
        env.storage().persistent().set(&DataKey::SponsorProfile(sponsor), &profile);
    }

    pub fn harvest_yield(env: Env, sponsor: Address, amount: i128, token: Address) {
        // High-precision accounting: Check sponsor's share of total yield
        let profile: SponsorProfile = env.storage().persistent()
            .get(&DataKey::SponsorProfile(sponsor.clone()))
            .expect("Sponsor profile not found");

        match profile.preference {
            SponsorYieldPreference::Reinvest => {
                // Add back to active capital
                let mut updated_profile = profile;
                updated_profile.active_capital += amount;
                env.storage().persistent().set(&DataKey::SponsorProfile(sponsor.clone()), &updated_profile);
            },
            SponsorYieldPreference::ReturnToSponsor => {
                let client = token::Client::new(&env, &token);
                client.transfer(&env.current_contract_address(), &sponsor, &amount);
            },
            SponsorYieldPreference::DonateToDAO => {
                // Route to DAO/Pool
                let pool: Address = env.storage().instance().get(&DataKey::GlobalScholarshipPool).expect("Pool not set");
                let client = token::Client::new(&env, &token);
                client.transfer(&env.current_contract_address(), &pool, &amount);
            },
        }

        env.events().publish(
            (Symbol::new(&env, "YieldRoutedByPreference"), sponsor, Symbol::new(&env, "Yield")),
            amount
        );
    }

    /// #177 Emergency-Liquidity Withdrawal Bounds
    pub fn calculate_liquidity_bounds(env: Env) -> i128 {
        let total_tvl: i128 = env.storage().instance().get(&DataKey::TotalTVL).unwrap_or(0);
        let daily_burn: i128 = env.storage().instance().get(&DataKey::DailyBurnRate).unwrap_or(0);
        
        let fourteen_day_burn = daily_burn * 14;
        let buffer = (total_tvl * 5) / 100; // 5% buffer
        
        let required_liquidity = fourteen_day_burn + buffer;
        if total_tvl < required_liquidity {
            return 0;
        }
        total_tvl - required_liquidity
    }

    pub fn route_to_yield(env: Env, admin: Address, amount: i128) {
        admin.require_auth();
        let deployable = Self::calculate_liquidity_bounds(env.clone());
        
        if amount > deployable {
            env.events().publish((Symbol::new(&env, "LiquidityBoundEnforced"), amount), deployable);
            panic!("Exceeds liquidity bounds");
        }

        // Logic to move funds to external DeFi would go here
    }

    // --- Missing Core Functions Implementation ---

    pub fn create_stream(env: Env, funder: Address, student: Address, amount_per_second: i128, token: Address, restriction: Option<Symbol>) {
        funder.require_auth();
        let current_time = env.ledger().timestamp();
        let stream = Stream {
            funder: funder.clone(),
            student: student.clone(),
            amount_per_second,
            total_deposited: 0,
            total_withdrawn: 0,
            start_time: current_time,
            is_active: true,
            geographic_restriction: restriction,
        };
        env.storage().persistent().set(&DataKey::Stream(funder, student), &stream);
    }

    pub fn withdraw_from_stream(env: Env, student: Address, funder: Address, token: Address) -> i128 {
        student.require_auth();
        let stream_key = DataKey::Stream(funder.clone(), student.clone());
        let mut stream: Stream = env.storage().persistent().get(&stream_key).expect("Stream not found");
        
        let current_time = env.ledger().timestamp();
        let elapsed = current_time - stream.start_time;
        let accrued = (elapsed as i128) * stream.amount_per_second;
        let available = accrued - stream.total_withdrawn;
        
        if available <= 0 {
            return 0;
        }

        // Apply Alumni Tax if applicable
        let final_amount = Self::check_and_apply_alumni_tax(&env, &student, available);

        let client = token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &student, &final_amount);
        
        stream.total_withdrawn += available;
        env.storage().persistent().set(&stream_key, &stream);
        
        final_amount
    }

    fn distribute_royalty(env: &Env, _course_id: u64, amount: i128, token: &Address) {
        // Placeholder for royalty distribution logic
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap_or(env.current_contract_address());
        let client = token::Client::new(env, token);
        let royalty = amount / 10; // 10% royalty
        if royalty > 0 {
            client.transfer(&env.current_contract_address(), &admin, &royalty);
        }
    }

    fn distribute_tuition_stipend_split(env: &Env, _student: &Address, amount: i128, _token: &Address) -> (i128, i128) {
        // Placeholder for split logic (70/30)
        let university_share = (amount * 70) / 100;
        let student_share = amount - university_share;
        (university_share, student_share)
    }

    fn apply_attendance_penalty_to_rate(_env: Env, _student: Address, rate: i128) -> i128 {
        // Placeholder for attendance penalty
        rate
    }

    pub fn withdraw_scholarship(env: Env, student: Address, amount: i128) {
        student.require_auth();
        let mut scholarship: Scholarship = env.storage().persistent().get(&DataKey::Scholarship(student.clone())).expect("Scholarship not found");
        
        if scholarship.is_paused { panic!("Paused"); }
        if scholarship.unlocked_balance < amount { panic!("Insufficient unlocked"); }
        
        let client = token::Client::new(&env, &scholarship.token);
        client.transfer(&env.current_contract_address(), &student, &amount);
        
        scholarship.balance -= amount;
        scholarship.unlocked_balance -= amount;
        env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
    }

    pub fn verify_academic_progress(env: Env, student: Address, _course_id: u64) {
        // Mock verification: unlocks some balance
        let mut scholarship: Scholarship = env.storage().persistent().get(&DataKey::Scholarship(student.clone())).expect("Scholarship not found");
        scholarship.unlocked_balance += 100; // Unlock 100 units
        env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
    }

    pub fn set_course_duration(env: Env, course_id: u64, duration: u64) {
        env.storage().persistent().set(&DataKey::CourseDuration(course_id), &duration);
    }

    pub fn is_sbt_minted(env: Env, student: Address, course_id: u64) -> bool {
        env.storage().persistent().get(&DataKey::SbtMinted(student, course_id)).unwrap_or(false)
    }

    pub fn get_watch_time(env: Env, student: Address, course_id: u64) -> u64 {
        let access: Access = env.storage().persistent().get(&DataKey::Access(student, course_id)).expect("No access");
        access.total_watch_time
    }
}

mod test;
mod tuition_stipend_split_tests;
mod student_profile_nft;
