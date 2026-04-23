#![no_std]
use core::convert::TryFrom;
use expiry_math::{checked_access_expiry, checked_subscription_expiry};
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol, Vec, IntoVal};

// --- Constants for TTL Management ---
const LEDGER_BUMP_THRESHOLD: u32 = 123456; // Example threshold
const LEDGER_BUMP_EXTEND: u32 = 789012;    // Example extension
const MAX_COURSE_REGISTRY_SIZE: u64 = 1000;
const EARLY_DROP_WINDOW_SECONDS: u64 = 300; // 5 minutes
const MAX_COURSE_REGISTRY_SIZE: u64 = 1000; // Maximum number of courses to prevent gas limit issues

// GPA Bonus Constants
const GPA_BONUS_THRESHOLD: u64 = 35; // 3.5 GPA threshold (stored as 35 to avoid floating point)
const GPA_BONUS_PERCENTAGE_PER_POINT: u64 = 20; // 2% per 0.1 GPA (20% per 1.0 GPA)

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    SbtMint(Address, u64),
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
    CourseRegistry,
    CourseRegistrySize,
    CourseInfo(u64),
    BonusMinutes(Address),
    HasBeenReferred(Address),
    ReferralBonusAmount,
    RoyaltySplit(u64),
    AcademicOracle,
    // Research Grant Milestone Escrow keys
    ResearchGrant(Address), // student -> ResearchGrant struct
    MilestoneClaim(u64), // milestone_id -> MilestoneClaim struct
    InvoiceHash(u64), // milestone_id -> invoice hash
    GrantorApproval(u64), // milestone_id -> approval status
    StreakBonusAmount,
    ConsecutiveDays(Address, u64), // student, course_id -> StreakData
    GroupPool(u64), // pool_id -> GroupPool struct
    GroupPoolMember(u64, Address), // pool_id, member -> contribution amount
    GroupPoolAccess(u64, Address), // pool_id, member -> access granted
    ModuleLockConfig(u64, u64), // course_id, module_id -> requires_quiz
    ModuleQuizLock(Address, u64, u64), // student, course_id, module_id -> QuizProof
    TuitionStipendSplit(Address), // student -> TuitionStipendSplit struct
    StudentGPA(Address), // student -> StudentGPA struct
    // Multi-Sig Academic Board Review Keys
    DeansCouncil,
    BoardPauseRequest(Address), // student -> BoardPauseRequest struct
    BoardSignature(Address, Address), // student, council_member -> signature
    // Multi-Language Metadata Keys
    CourseMetadata(u64), // course_id -> CourseMetadata struct
    CourseLanguageMetadata(u64, Symbol), // course_id, language_code -> LanguageMetadata struct
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

// Multi-Language Metadata Structs
#[contracttype]
#[derive(Clone)]
pub struct LanguageMetadata {
    pub language_code: Symbol, // ISO 639-1 language code (e.g., "en", "es", "zh")
    pub ipfs_cid: Symbol, // IPFS CID for the language-specific metadata JSON
    pub title: Symbol, // Course title in this language
    pub description: Symbol, // Course description in this language
    pub thumbnail_cid: Option<Symbol>, // Optional language-specific thumbnail IPFS CID
    pub last_updated: u64, // Timestamp when this language version was last updated
}

#[contracttype]
#[derive(Clone)]
pub struct CourseMetadata {
    pub course_id: u64,
    pub default_language: Symbol, // Default language code
    pub available_languages: Vec<Symbol>, // List of available language codes
    pub base_metadata_cid: Symbol, // IPFS CID for base course metadata
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
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
            }
        }

        env.storage().persistent().set(&access_key, &access);
        env.storage().persistent().extend_ttl(&access_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
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
            false
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
            panic!("Refund only available within 5 minutes of purchase");
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

    pub fn withdraw_scholarship(env: Env, student: Address, amount: i128) {
        student.require_auth();

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
            .set(&DataKey::Scholarship(student.clone()), &scholarship);

        // Transfer back to student
        let client = token::Client::new(&env, &scholarship.token);
        client.transfer(&env.current_contract_address(), &student, &amount);
    }

    pub fn set_academic_oracle(env: Env, admin: Address, oracle: Address) {
        admin.require_auth();
        if !Self::is_admin(&env, &admin) {
            panic!("Unauthorized");
        }
        env.storage().instance().set(&DataKey::AcademicOracle, &oracle);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Academic_Oracle_Updated"), admin),
            oracle,
        );
    }

    pub fn get_academic_oracle(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::AcademicOracle)
    }

    pub fn report_student_gpa(env: Env, oracle: Address, student: Address, gpa: u64) {
        oracle.require_auth();
        
        // Verify caller is the designated academic oracle
        let stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::AcademicOracle)
            .expect("Academic Oracle not set");
        if stored_oracle != oracle {
            panic!("Unauthorized: Not the academic oracle");
        }

        // Validate GPA range (0.0 to 4.4 stored as 0 to 44)
        if gpa > 44 {
            panic!("Invalid GPA: Maximum supported is 4.4");
        }

        let current_time = env.ledger().timestamp();
        let gpa_data = StudentGPA {
            student: student.clone(),
            gpa,
            last_updated: current_time,
            oracle_verified: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::StudentGPA(student.clone()), &gpa_data);
        env.storage().persistent().extend_ttl(
            &DataKey::StudentGPA(student),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        // Calculate and emit GPA bonus information
        let bonus_percentage = Self::calculate_gpa_bonus(env.clone(), student.clone());
        
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "GPA_Updated"), student, gpa),
            bonus_percentage,
        );

        // Trigger on-the-fly drip recalculation for meritocratic adjustment
        Self::recalculate_drip_rate_on_gpa_change(env.clone(), student.clone());
    }

    pub fn get_student_gpa(env: Env, student: Address) -> Option<StudentGPA> {
        let key = DataKey::StudentGPA(student.clone());
        let gpa_data = env.storage().persistent().get(&key);
        if gpa_data.is_some() {
            env.storage()
                .persistent()
                .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        }
        gpa_data
    }

    pub fn get_student_gpa_bonus(env: Env, student: Address) -> u64 {
        Self::calculate_gpa_bonus(env, student)
    }

    pub fn recalculate_drip_rate_on_gpa_change(env: Env, student: Address) {
        // This function handles on-the-fly drip recalculation when GPA changes
        // It preserves the stream start date and remaining balance while adjusting the rate
        
        let scholarship: Option<Scholarship> = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()));
        
        if let Some(mut sch) = scholarship {
            if !sch.is_paused {
                // Calculate new rate with updated GPA bonus
                let base_rate: i128 = env
                    .storage()
                    .instance()
                    .get(&DataKey::BaseRate)
                    .unwrap_or(1);
                
                // Apply GPA bonus to base rate
                let gpa_bonus_percentage = Self::calculate_gpa_bonus(env.clone(), student.clone());
                let new_rate = if gpa_bonus_percentage > 0 {
                    let bonus = (base_rate * gpa_bonus_percentage as i128) / 100;
                    base_rate + bonus
                } else {
                    base_rate
                };
                
                // Calculate remaining airtime with new rate
                let remaining_seconds = if new_rate > 0 {
                    (sch.unlocked_balance / new_rate) as u64
                } else {
                    0
                };
                
                // Emit event showing the recalculated rate and remaining time
                #[allow(deprecated)]
                env.events().publish(
                    (Symbol::new(&env, "Drip_Rate_Recalculated"), student),
                    (new_rate, remaining_seconds, gpa_bonus_percentage),
                );
            }
        }
    }

    pub fn pause_scholarship(env: Env, admin: Address, student: Address, status: bool) {
        admin.require_auth();
        if !Self::is_admin(&env, &admin) {
            panic!("Unauthorized");
        }

        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        scholarship.is_paused = status;
        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student.clone()), &scholarship);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Scholarship_Status_Updated"), student),
            status,
        );
    }

    pub fn verify_academic_progress(env: Env, student: Address, course_id: u64) {
        student.require_auth();

        let oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::AcademicOracle)
            .expect("Academic Oracle not set");

        // Call the external oracle
        // Assumption: Oracle has a function `check_status(student: Address, course_id: u64) -> u32`
        // 0: Fail, 1: Success, 2: Incomplete
        let status: u32 = env.invoke_contract(
            &oracle,
            &Symbol::new(&env, "check_status"),
            (student.clone(), course_id).into_val(&env),
        );

        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        if status == 1 {
            // Success - unlock next 30 days of drips
            let base_rate: i128 = env.storage().instance().get(&DataKey::BaseRate).unwrap_or(0);
            let thirty_days_seconds: i128 = 30 * 24 * 3600;
            let unlock_amount = thirty_days_seconds * base_rate;

            scholarship.unlocked_balance += unlock_amount;
            scholarship.last_verif = env.ledger().timestamp();
            scholarship.is_paused = false;

            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "Scholarship_Drip_Unlocked"), student.clone()),
                unlock_amount,
            );
        } else {
            // Fail or Incomplete - pause stream
            scholarship.is_paused = true;

            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "Scholarship_Paused_By_Oracle"), student.clone()),
                status,
            );
        }

        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student), &scholarship);
    }

    pub fn set_royalty_split(
        env: Env,
        caller: Address,
        course_id: u64,
        shares: Vec<(Address, u32)>,
    ) {
        caller.require_auth();

        // Verify caller is admin or course creator
        let course_info: Option<CourseInfo> = env
            .storage()
            .persistent()
            .get(&DataKey::CourseInfo(course_id));
        let is_admin = Self::is_admin(&env, &caller);
        let is_creator = course_info.map_or(false, |info| info.creator == caller);

        if !is_admin && !is_creator {
            panic!("Unauthorized")
        }

        // Validate total = 100%
        let total: u32 = shares.iter().map(|(_, p)| p).sum();
        if total != 100 {
            panic!("Royalty Share does not sum to 100")
        }

        let split = RoyaltySplit { shares };
        env.storage()
            .persistent()
            .set(&DataKey::RoyaltySplit(course_id), &split);
        env.storage().persistent().extend_ttl(
            &DataKey::RoyaltySplit(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );
    }

    pub fn get_royalty_split(env: Env, course_id: u64) -> Option<RoyaltySplit> {
        let key = DataKey::RoyaltySplit(course_id);
        let split = env.storage().persistent().get(&key);
        if split.is_some() {
            env.storage()
                .persistent()
                .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        }
        split
    }

    fn distribute_royalty(env: &Env, course_id: u64, total_amount: i128, token: &Address) {
        if let Some(split) = env.storage().persistent().get::<_, RoyaltySplit>(&DataKey::RoyaltySplit(course_id)) {
            let client = token::Client::new(env, token);
            for (recipient, percentage) in split.shares.iter() {
                let share = (total_amount * percentage as i128) / 100;
                if share > 0 {
                    client.transfer(&env.current_contract_address(), &recipient, &share);
                }
            }
        }
    }

    // Tuition-Stipend Split Functions
    
    pub fn set_tuition_stipend_split(
        env: Env,
        admin: Address,
        student: Address,
        university_address: Address,
        university_percentage: u32,
        student_percentage: u32,
    ) {
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
        
        // Validate percentages sum to 100
        if university_percentage + student_percentage != 100 {
            panic!("Percentages must sum to 100");
        }
        
        let split_config = TuitionStipendSplit {
            university_address,
            student_address: student.clone(),
            university_percentage,
            student_percentage,
        };
        
        env.storage()
            .persistent()
            .set(&DataKey::TuitionStipendSplit(student), &split_config);
        env.storage().persistent().extend_ttl(
            &DataKey::TuitionStipendSplit(student),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "TuitionStipendSplit_Configured"), admin, student),
            (university_address, university_percentage, student_percentage)
        );
    }
    
    pub fn get_tuition_stipend_split(env: Env, student: Address) -> Option<TuitionStipendSplit> {
        let key = DataKey::TuitionStipendSplit(student);
        let split = env.storage().persistent().get(&key);
        if split.is_some() {
            env.storage()
                .persistent()
                .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        }
        split
    }
    
    pub fn distribute_tuition_stipend_split(
        env: &Env,
        student: &Address,
        total_amount: i128,
        token: &Address,
    ) -> (i128, i128) {
        let split: Option<TuitionStipendSplit> = env
            .storage()
            .persistent()
            .get(&DataKey::TuitionStipendSplit(student));
        
        let client = token::Client::new(env, token);
        
        if let Some(config) = split {
            // Calculate university share (70% by default)
            let university_share = (total_amount * config.university_percentage as i128) / 100;
            let student_share = (total_amount * config.student_percentage as i128) / 100;
            
            // Transfer to university first (priority payment)
            if university_share > 0 {
                client.transfer(&env.current_contract_address(), &config.university_address, &university_share);
            }
            
            // Transfer remainder to student
            if student_share > 0 {
                client.transfer(&env.current_contract_address(), &config.student_address, &student_share);
            }
            
            // Emit split distribution event
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "TuitionStipendSplit_Distributed"), student, config.university_address),
                (university_share, student_share)
            );
            
            (university_share, student_share)
        } else {
            // No split configured, transfer full amount to student
            client.transfer(&env.current_contract_address(), student, &total_amount);
            (0, total_amount)
        }
    }

    // Course Registry Management Functions

    pub fn add_course_to_registry(env: Env, course_id: u64, creator: Address) {
        creator.require_auth();

        // Check if course already exists
        if let Some(_) = env
            .storage()
            .persistent()
            .get::<DataKey, CourseInfo>(&DataKey::CourseInfo(course_id))
        {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        // Check registry size limit to prevent gas limit issues
        let registry_size: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::CourseRegistrySize)
            .unwrap_or(0);
        if registry_size >= MAX_COURSE_REGISTRY_SIZE {
            panic!("LimitExceeded");
        }

        let current_time = env.ledger().timestamp();

        // Create course info
        let course_info = CourseInfo {
            course_id,
            created_at: current_time,
            is_active: true,
            creator: creator.clone(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::CourseInfo(course_id), &course_info);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseInfo(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        // Update registry
        let mut registry: CourseRegistry = env
            .storage()
            .persistent()
            .get(&DataKey::CourseRegistry)
            .unwrap_or(CourseRegistry {
                courses: Vec::new(&env),
                last_updated: current_time,
            });

        registry.courses.push_back(course_id);
        registry.last_updated = current_time;

        env.storage()
            .persistent()
            .set(&DataKey::CourseRegistry, &registry);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseRegistry,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        // Update size counter
        env.storage()
            .persistent()
            .set(&DataKey::CourseRegistrySize, &(registry_size + 1));
        env.storage().persistent().extend_ttl(
            &DataKey::CourseRegistrySize,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );
    }

    pub fn list_courses(env: Env) -> Vec<u64> {
        let registry: CourseRegistry = env
            .storage()
            .persistent()
            .get(&DataKey::CourseRegistry)
            .unwrap_or(CourseRegistry {
                courses: Vec::new(&env),
                last_updated: 0,
            });

        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(
            &DataKey::CourseRegistry,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        registry.courses
    }

    pub fn list_courses_paginated(env: Env, offset: u32, limit: u32) -> Vec<u64> {
        // Validate pagination parameters to prevent excessive gas consumption
        if limit > 100 {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        let registry: CourseRegistry = env
            .storage()
            .persistent()
            .get(&DataKey::CourseRegistry)
            .unwrap_or(CourseRegistry {
                courses: Vec::new(&env),
                last_updated: 0,
            });

        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(
            &DataKey::CourseRegistry,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        let total_courses = registry.courses.len();

        if offset >= total_courses {
            return Vec::new(&env);
        }

        let end_index = core::cmp::min(offset + limit, total_courses);
        let mut result = Vec::new(&env);

        for i in offset..end_index {
            result.push_back(registry.courses.get(i).unwrap());
        }

        result
    }

    pub fn get_course_info(env: Env, course_id: u64) -> CourseInfo {
        let course_info: CourseInfo = env
            .storage()
            .persistent()
            .get(&DataKey::CourseInfo(course_id))
            .unwrap_or_else(|| panic!("NotFound"));

        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(
            &DataKey::CourseInfo(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        course_info
    }

    pub fn deactivate_course(env: Env, admin: Address, course_id: u64) {
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

        let mut course_info: CourseInfo = env
            .storage()
            .persistent()
            .get(&DataKey::CourseInfo(course_id))
            .unwrap_or_else(|| panic!("NotFound"));

        course_info.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::CourseInfo(course_id), &course_info);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseInfo(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );
    }

    pub fn cleanup_inactive_courses(env: Env, admin: Address) -> u64 {
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

        let registry: CourseRegistry = env
            .storage()
            .persistent()
            .get(&DataKey::CourseRegistry)
            .unwrap_or(CourseRegistry {
                courses: Vec::new(&env),
                last_updated: 0,
            });

        let mut removed_count = 0;
        let mut active_courses = Vec::new(&env);
        let current_time = env.ledger().timestamp();

        // Filter out inactive courses
        for i in 0..registry.courses.len() {
            let course_id = registry.courses.get(i).unwrap();
            if let Some(course_info) = env
                .storage()
                .persistent()
                .get::<DataKey, CourseInfo>(&DataKey::CourseInfo(course_id))
            {
                if course_info.is_active {
                    active_courses.push_back(course_id);
                } else {
                    // Remove inactive course info
                    env.storage()
                        .persistent()
                        .remove(&DataKey::CourseInfo(course_id));
                    removed_count += 1;
                }
            }
        }

        // Update registry with only active courses
        let updated_registry = CourseRegistry {
            courses: active_courses,
            last_updated: current_time,
        };

        env.storage()
            .persistent()
            .set(&DataKey::CourseRegistry, &updated_registry);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseRegistry,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        // Update size counter
        let new_size = updated_registry.courses.len() as u64;
        env.storage()
            .persistent()
            .set(&DataKey::CourseRegistrySize, &new_size);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseRegistrySize,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        removed_count
    }

    // Multi-Language Metadata Functions

    pub fn create_course_metadata(
        env: Env,
        course_id: u64,
        default_language: Symbol,
        base_metadata_cid: Symbol,
        creator: Address,
    ) {
        creator.require_auth();

        // Verify caller is admin or course creator
        let course_info: Option<CourseInfo> = env
            .storage()
            .persistent()
            .get(&DataKey::CourseInfo(course_id));
        let is_admin = Self::is_admin(&env, &creator);
        let is_creator = course_info.map_or(false, |info| info.creator == creator);

        if !is_admin && !is_creator {
            panic!("Unauthorized: Only admin or course creator can set metadata");
        }

        // Check if metadata already exists
        if let Some(_) = env
            .storage()
            .persistent()
            .get::<_, CourseMetadata>(&DataKey::CourseMetadata(course_id))
        {
            panic!("Metadata already exists for this course");
        }

        let current_time = env.ledger().timestamp();

        // Create course metadata
        let metadata = CourseMetadata {
            course_id,
            default_language: default_language.clone(),
            available_languages: Vec::from_array(&env, [default_language.clone()]),
            base_metadata_cid,
            created_at: current_time,
            updated_at: current_time,
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::CourseMetadata(course_id), &metadata);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseMetadata(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Course_Metadata_Created"), course_id, creator),
            default_language,
        );
    }

    pub fn add_language_metadata(
        env: Env,
        course_id: u64,
        language_code: Symbol,
        ipfs_cid: Symbol,
        title: Symbol,
        description: Symbol,
        thumbnail_cid: Option<Symbol>,
        creator: Address,
    ) {
        creator.require_auth();

        // Verify caller is admin or course creator
        let course_info: Option<CourseInfo> = env
            .storage()
            .persistent()
            .get(&DataKey::CourseInfo(course_id));
        let is_admin = Self::is_admin(&env, &creator);
        let is_creator = course_info.map_or(false, |info| info.creator == creator);

        if !is_admin && !is_creator {
            panic!("Unauthorized: Only admin or course creator can add language metadata");
        }

        // Check if course metadata exists
        let mut metadata: CourseMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CourseMetadata(course_id))
            .expect("Course metadata not found");

        // Check if language already exists
        let lang_key = DataKey::CourseLanguageMetadata(course_id, language_code.clone());
        if let Some(_) = env.storage().persistent().get::<_, LanguageMetadata>(&lang_key) {
            panic!("Language metadata already exists for this language");
        }

        let current_time = env.ledger().timestamp();

        // Create language metadata
        let lang_metadata = LanguageMetadata {
            language_code: language_code.clone(),
            ipfs_cid,
            title,
            description,
            thumbnail_cid,
            last_updated: current_time,
        };

        // Store language metadata
        env.storage()
            .persistent()
            .set(&lang_key, &lang_metadata);
        env.storage().persistent().extend_ttl(
            &lang_key,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        // Update course metadata
        if !metadata.available_languages.contains(&language_code) {
            metadata.available_languages.push_back(language_code.clone());
        }
        metadata.updated_at = current_time;

        env.storage()
            .persistent()
            .set(&DataKey::CourseMetadata(course_id), &metadata);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseMetadata(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Language_Metadata_Added"), course_id, creator),
            language_code,
        );
    }

    pub fn update_language_metadata(
        env: Env,
        course_id: u64,
        language_code: Symbol,
        ipfs_cid: Symbol,
        title: Symbol,
        description: Symbol,
        thumbnail_cid: Option<Symbol>,
        creator: Address,
    ) {
        creator.require_auth();

        // Verify caller is admin or course creator
        let course_info: Option<CourseInfo> = env
            .storage()
            .persistent()
            .get(&DataKey::CourseInfo(course_id));
        let is_admin = Self::is_admin(&env, &creator);
        let is_creator = course_info.map_or(false, |info| info.creator == creator);

        if !is_admin && !is_creator {
            panic!("Unauthorized: Only admin or course creator can update language metadata");
        }

        let lang_key = DataKey::CourseLanguageMetadata(course_id, language_code.clone());
        let mut lang_metadata: LanguageMetadata = env
            .storage()
            .persistent()
            .get(&lang_key)
            .expect("Language metadata not found");

        let current_time = env.ledger().timestamp();

        // Update language metadata
        lang_metadata.ipfs_cid = ipfs_cid;
        lang_metadata.title = title;
        lang_metadata.description = description;
        lang_metadata.thumbnail_cid = thumbnail_cid;
        lang_metadata.last_updated = current_time;

        env.storage()
            .persistent()
            .set(&lang_key, &lang_metadata);
        env.storage().persistent().extend_ttl(
            &lang_key,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        // Update course metadata timestamp
        let mut metadata: CourseMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CourseMetadata(course_id))
            .expect("Course metadata not found");
        metadata.updated_at = current_time;

        env.storage()
            .persistent()
            .set(&DataKey::CourseMetadata(course_id), &metadata);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseMetadata(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Language_Metadata_Updated"), course_id, creator),
            language_code,
        );
    }

    pub fn get_course_metadata(env: Env, course_id: u64) -> CourseMetadata {
        let metadata: CourseMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CourseMetadata(course_id))
            .unwrap_or_else(|| panic!("Course metadata not found"));

        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(
            &DataKey::CourseMetadata(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        metadata
    }

    pub fn get_language_metadata(env: Env, course_id: u64, language_code: Symbol) -> LanguageMetadata {
        let lang_key = DataKey::CourseLanguageMetadata(course_id, language_code.clone());
        let metadata: LanguageMetadata = env
            .storage()
            .persistent()
            .get(&lang_key)
            .unwrap_or_else(|| panic!("Language metadata not found"));

        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(
            &lang_key,
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        metadata
    }

    pub fn get_available_languages(env: Env, course_id: u64) -> Vec<Symbol> {
        let metadata: CourseMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CourseMetadata(course_id))
            .unwrap_or_else(|| panic!("Course metadata not found"));

        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(
            &DataKey::CourseMetadata(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        metadata.available_languages
    }

    pub fn get_default_language(env: Env, course_id: u64) -> Symbol {
        let metadata: CourseMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CourseMetadata(course_id))
            .unwrap_or_else(|| panic!("Course metadata not found"));

        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(
            &DataKey::CourseMetadata(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        metadata.default_language
    }

    pub fn set_default_language(
        env: Env,
        course_id: u64,
        new_default_language: Symbol,
        creator: Address,
    ) {
        creator.require_auth();

        // Verify caller is admin or course creator
        let course_info: Option<CourseInfo> = env
            .storage()
            .persistent()
            .get(&DataKey::CourseInfo(course_id));
        let is_admin = Self::is_admin(&env, &creator);
        let is_creator = course_info.map_or(false, |info| info.creator == creator);

        if !is_admin && !is_creator {
            panic!("Unauthorized: Only admin or course creator can set default language");
        }

        let mut metadata: CourseMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CourseMetadata(course_id))
            .expect("Course metadata not found");

        // Verify the language exists
        if !metadata.available_languages.contains(&new_default_language) {
            panic!("Language not available for this course");
        }

        metadata.default_language = new_default_language.clone();
        metadata.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::CourseMetadata(course_id), &metadata);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseMetadata(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Default_Language_Updated"), course_id, creator),
            new_default_language,
        );
    }

    pub fn remove_language_metadata(
        env: Env,
        course_id: u64,
        language_code: Symbol,
        creator: Address,
    ) {
        creator.require_auth();

        // Verify caller is admin or course creator
        let course_info: Option<CourseInfo> = env
            .storage()
            .persistent()
            .get(&DataKey::CourseInfo(course_id));
        let is_admin = Self::is_admin(&env, &creator);
        let is_creator = course_info.map_or(false, |info| info.creator == creator);

        if !is_admin && !is_creator {
            panic!("Unauthorized: Only admin or course creator can remove language metadata");
        }

        let mut metadata: CourseMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::CourseMetadata(course_id))
            .expect("Course metadata not found");

        // Cannot remove the default language
        if metadata.default_language == language_code {
            panic!("Cannot remove default language");
        }

        // Remove language metadata
        let lang_key = DataKey::CourseLanguageMetadata(course_id, language_code.clone());
        env.storage().persistent().remove(&lang_key);

        // Remove from available languages
        let mut new_languages = Vec::new(&env);
        for i in 0..metadata.available_languages.len() {
            let lang = metadata.available_languages.get(i).unwrap();
            if lang != language_code {
                new_languages.push_back(lang);
            }
        }
        metadata.available_languages = new_languages;
        metadata.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::CourseMetadata(course_id), &metadata);
        env.storage().persistent().extend_ttl(
            &DataKey::CourseMetadata(course_id),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Language_Metadata_Removed"), course_id, creator),
            language_code,
        );
    }

    // Referral System

    pub fn set_referral_bonus_amount(env: Env, admin: Address, amount: u64) {
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
            .instance()
            .set(&DataKey::ReferralBonusAmount, &amount);
    }

    pub fn referral_reward_claim(env: Env, referrer: Address, friend: Address) {
        friend.require_auth();

        // Ensure the friend hasn't already been referred
        let has_been_referred: bool = env
            .storage()
            .persistent()
            .get(&DataKey::HasBeenReferred(friend.clone()))
            .unwrap_or(false);

        if has_been_referred {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }

        // Get configured bonus amount, default to 3600 seconds (60 minutes)
        let bonus_amount: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ReferralBonusAmount)
            .unwrap_or(3600);

        // Add to referrer's bonus minutes balance
        let mut current_bonus: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::BonusMinutes(referrer.clone()))
            .unwrap_or(0);

        current_bonus += bonus_amount;

        env.storage()
            .persistent()
            .set(&DataKey::BonusMinutes(referrer.clone()), &current_bonus);
        env.storage().persistent().extend_ttl(
            &DataKey::BonusMinutes(referrer.clone()),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        // Mark friend as referred
        env.storage()
            .persistent()
            .set(&DataKey::HasBeenReferred(friend.clone()), &true);
        env.storage().persistent().extend_ttl(
            &DataKey::HasBeenReferred(friend.clone()),
            LEDGER_BUMP_THRESHOLD,
            LEDGER_BUMP_EXTEND,
        );

        // Emit an event for the referral
        #[allow(deprecated)]
        env.events().publish(
            (
                Symbol::new(&env, "Referral_Claimed"),
                referrer,
                friend.clone(),
            ),
            bonus_amount,
        );
    }

    pub fn get_bonus_minutes(env: Env, student: Address) -> u64 {
        let key = DataKey::BonusMinutes(student);
        if env.storage().persistent().has(&key) {
            env.storage()
                .persistent()
                .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key).unwrap_or(0)
        } else {
            0
        }
    }

    // Research Grant Milestone Escrow Functions
    
    pub fn create_research_grant(
        env: Env,
        grantor: Address,
        student: Address,
        total_amount: i128,
        token: Address,
    ) -> u64 {
        grantor.require_auth();
        
        // Transfer funds to contract
        let client = token::Client::new(&env, &token);
        client.transfer(&grantor, &env.current_contract_address(), &total_amount);
        
        // Generate unique grant ID
        let grant_id: u64 = env.storage().instance()
            .get(&Symbol::new(&env, "NextGrantId"))
            .unwrap_or(1);
        
        let current_time = env.ledger().timestamp();
        
        let research_grant = ResearchGrant {
            student: student.clone(),
            total_amount,
            token: token.clone(),
            granted_at: current_time,
            is_active: true,
            grantor: grantor.clone(),
        };
        
        // Store the grant
        env.storage().persistent().set(&DataKey::ResearchGrant(student), &research_grant);
        env.storage().persistent().extend_ttl(&DataKey::ResearchGrant(student), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Increment next grant ID
        env.storage().instance().set(&Symbol::new(&env, "NextGrantId"), &(grant_id + 1));
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Research_Grant_Created"), grantor, student),
            (grant_id, total_amount)
        );
        
        grant_id
    }
    
    pub fn submit_milestone_claim(
        env: Env,
        student: Address,
        milestone_id: u64,
        amount: i128,
        description: Symbol,
        invoice_hash: Symbol,
    ) {
        student.require_auth();
        
        // Verify student has an active research grant
        let research_grant: ResearchGrant = env.storage().persistent()
            .get(&DataKey::ResearchGrant(student.clone()))
            .expect("No active research grant found");
        
        if !research_grant.is_active {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }
        
        let current_time = env.ledger().timestamp();
        
        let milestone_claim = MilestoneClaim {
            milestone_id,
            student: student.clone(),
            amount,
            description: description.clone(),
            invoice_hash: Some(invoice_hash.clone()),
            is_approved: false,
            is_claimed: false,
            submitted_at: current_time,
            approved_at: None,
            claimed_at: None,
        };
        
        // Store the milestone claim
        env.storage().persistent().set(&DataKey::MilestoneClaim(milestone_id), &milestone_claim);
        env.storage().persistent().set(&DataKey::InvoiceHash(milestone_id), &invoice_hash);
        env.storage().persistent().extend_ttl(&DataKey::MilestoneClaim(milestone_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        env.storage().persistent().extend_ttl(&DataKey::InvoiceHash(milestone_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Milestone_Claim_Submitted"), student, milestone_id),
            (amount, description, invoice_hash)
        );
    }
    
    pub fn approve_milestone_claim(
        env: Env,
        grantor: Address,
        milestone_id: u64,
    ) {
        grantor.require_auth();
        
        let mut milestone_claim: MilestoneClaim = env.storage().persistent()
            .get(&DataKey::MilestoneClaim(milestone_id))
            .expect("Milestone claim not found");
        
        // Verify grantor is the one who created the research grant
        let research_grant: ResearchGrant = env.storage().persistent()
            .get(&DataKey::ResearchGrant(milestone_claim.student.clone()))
            .expect("Research grant not found");
        
        if research_grant.grantor != grantor {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }
        
        if milestone_claim.is_approved {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }
        
        let current_time = env.ledger().timestamp();
        
        milestone_claim.is_approved = true;
        milestone_claim.approved_at = Some(current_time);
        
        // Store approval status
        env.storage().persistent().set(&DataKey::MilestoneClaim(milestone_id), &milestone_claim);
        env.storage().persistent().set(&DataKey::GrantorApproval(milestone_id), &true);
        env.storage().persistent().extend_ttl(&DataKey::MilestoneClaim(milestone_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        env.storage().persistent().extend_ttl(&DataKey::GrantorApproval(milestone_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Milestone_Claim_Approved"), grantor, milestone_claim.student.clone()),
            (milestone_id, milestone_claim.amount)
        );
    }
    
    pub fn claim_milestone_lump_sum(
        env: Env,
        student: Address,
        milestone_id: u64,
    ) {
        student.require_auth();
        
        let mut milestone_claim: MilestoneClaim = env.storage().persistent()
            .get(&DataKey::MilestoneClaim(milestone_id))
            .expect("Milestone claim not found");
        
        // Verify the claim belongs to this student
        if milestone_claim.student != student {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }
        
        // Verify claim is approved but not yet claimed
        if !milestone_claim.is_approved || milestone_claim.is_claimed {
            env.panic_with_error((
                soroban_sdk::xdr::ScErrorType::Contract,
                soroban_sdk::xdr::ScErrorCode::InvalidAction,
            ));
        }
        
        // Get the research grant to get token info
        let research_grant: ResearchGrant = env.storage().persistent()
            .get(&DataKey::ResearchGrant(student.clone()))
            .expect("Research grant not found");
        
        let current_time = env.ledger().timestamp();
        
        // Mark as claimed
        milestone_claim.is_claimed = true;
        milestone_claim.claimed_at = Some(current_time);
        
        // Update storage
        env.storage().persistent().set(&DataKey::MilestoneClaim(milestone_id), &milestone_claim);
        env.storage().persistent().extend_ttl(&DataKey::MilestoneClaim(milestone_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Transfer lump sum from treasury to student
        let client = token::Client::new(&env, &research_grant.token);
        client.transfer(&env.current_contract_address(), &student, &milestone_claim.amount);
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Milestone_Lump_Sum_Claimed"), student, milestone_id),
            (milestone_claim.amount, current_time)
        );
    }
    
    pub fn get_milestone_claim(env: Env, milestone_id: u64) -> MilestoneClaim {
        let key = DataKey::MilestoneClaim(milestone_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key).expect("Milestone claim not found")
        } else {
            panic!("Milestone claim not found");
        }
    }
    
    pub fn get_research_grant(env: Env, student: Address) -> ResearchGrant {
        let key = DataKey::ResearchGrant(student);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key).expect("Research grant not found")
        } else {
            panic!("Research grant not found");
        }
    }
    
    pub fn get_invoice_hash(env: Env, milestone_id: u64) -> Option<Symbol> {
        let key = DataKey::InvoiceHash(milestone_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key)
        } else {
            None
        }
    }
    
    pub fn is_milestone_approved(env: Env, milestone_id: u64) -> bool {
        let key = DataKey::GrantorApproval(milestone_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key).unwrap_or(false)
        } else {
            false
        }
    }

    // Gas Subsidy Feature - Reward students for consecutive learning days
    
    pub fn set_streak_bonus_amount(env: Env, admin: Address, amount: i128) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        env.storage().instance().set(&DataKey::StreakBonusAmount, &amount);
    }

    pub fn update_learning_streak(env: Env, student: Address, course_id: u64) {
        student.require_auth();
        
        let current_time = env.ledger().timestamp();
        let seconds_in_day = 86400; // 24 hours in seconds
        
        // Get existing streak data or create new
        let mut streak_data: StreakData = env.storage().persistent()
            .get(&DataKey::ConsecutiveDays(student.clone(), course_id))
            .unwrap_or(StreakData {
                current_streak: 0,
                last_watch_date: 0,
                total_reward_claimed: 0,
            });
        
        // Calculate days since last watch
        if streak_data.last_watch_date == 0 {
            // First time watching
            streak_data.current_streak = 1;
        } else {
            let days_since_last = (current_time - streak_data.last_watch_date) / seconds_in_day;
            
            if days_since_last == 0 {
                // Same day - don't increment, just update timestamp
                // This prevents multiple counts per day
            } else if days_since_last == 1 {
                // Consecutive day
                streak_data.current_streak += 1;
            } else {
                // Streak broken - reset to 1
                streak_data.current_streak = 1;
            }
        }
        
        streak_data.last_watch_date = current_time;
        
        // Check if student reached 5 consecutive days threshold
        if streak_data.current_streak == 5 {
            // Award gas subsidy
            let bonus_amount: i128 = env.storage().instance()
                .get(&DataKey::StreakBonusAmount)
                .unwrap_or(100_000_000); // Default 10 XLM (in stroops)
            
            streak_data.total_reward_claimed += bonus_amount;
            
            // Emit event for gas subsidy award
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "Gas_Subsidy_Awarded"), student.clone(), course_id),
                (streak_data.current_streak, bonus_amount)
            );
        }
        
        env.storage().persistent().set(&DataKey::ConsecutiveDays(student.clone(), course_id), &streak_data);
        env.storage().persistent().extend_ttl(&DataKey::ConsecutiveDays(student.clone(), course_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
    }

    pub fn get_learning_streak(env: Env, student: Address, course_id: u64) -> StreakData {
        let key = DataKey::ConsecutiveDays(student.clone(), course_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key).unwrap_or(StreakData {
                current_streak: 0,
                last_watch_date: 0,
                total_reward_claimed: 0,
            })
        } else {
            StreakData {
                current_streak: 0,
                last_watch_date: 0,
                total_reward_claimed: 0,
            }
        }
    }

    pub fn claim_gas_subsidy(env: Env, student: Address, course_id: u64) {
        student.require_auth();
        
        let streak_data: StreakData = env.storage().persistent()
            .get(&DataKey::ConsecutiveDays(student.clone(), course_id))
            .expect("No streak data found");
        
        // Must have at least 5 day streak
        if streak_data.current_streak < 5 {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Calculate reward amount (10 XLM default per 5-day streak)
        let bonus_per_streak: i128 = env.storage().instance()
            .get(&DataKey::StreakBonusAmount)
            .unwrap_or(100_000_000); // 10 XLM in stroops
        
        // Calculate how many complete 5-day streaks haven't been claimed yet
        let complete_streaks = streak_data.current_streak / 5;
        let claimed_streaks = streak_data.total_reward_claimed / bonus_per_streak;
        let unclaimed_streaks = complete_streaks - claimed_streaks;
        
        if unclaimed_streaks == 0 {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let reward_amount = unclaimed_streaks * bonus_per_streak;
        
        // Transfer reward from contract balance to student
        // Note: This assumes the contract has been funded with XLM
        // In production, you'd want a separate treasury management system
        
        // Update total claimed
        let mut updated_streak = streak_data;
        updated_streak.total_reward_claimed += reward_amount;
        env.storage().persistent().set(&DataKey::ConsecutiveDays(student.clone(), course_id), &updated_streak);
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Gas_Subsidy_Claimed"), student.clone(), course_id),
            (reward_amount, updated_streak.current_streak)
        );
    }

    pub fn reset_streak(env: Env, student: Address, course_id: u64) {
        student.require_auth();
        
        // Allow manual reset if needed
        let streak_data = StreakData {
            current_streak: 0,
            last_watch_date: 0,
            total_reward_claimed: 0,
        };
        
        env.storage().persistent().set(&DataKey::ConsecutiveDays(student.clone(), course_id), &streak_data);
    }

    // Group Pooling Feature - Students can pool funds to unlock masterclass
    
    pub fn create_group_pool(env: Env, creator: Address, course_id: u64, target_amount: i128, token: Address) -> u64 {
        creator.require_auth();
        
        // Generate unique pool ID
        let pool_id: u64 = env.storage().instance()
            .get(&Symbol::new(&env, "NextPoolId"))
            .unwrap_or(1);
        
        let current_time = env.ledger().timestamp();
        
        let group_pool = GroupPool {
            pool_id,
            course_id,
            target_amount,
            current_balance: 0,
            creator: creator.clone(),
            token: token.clone(),
            is_active: true,
            member_count: 0,
            created_at: current_time,
        };
        
        env.storage().persistent().set(&DataKey::GroupPool(pool_id), &group_pool);
        env.storage().persistent().extend_ttl(&DataKey::GroupPool(pool_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Increment next pool ID
        env.storage().instance().set(&Symbol::new(&env, "NextPoolId"), &(pool_id + 1));
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "GroupPool_Created"), creator, course_id),
            (pool_id, target_amount)
        );
        
        pool_id
    }

    pub fn contribute_to_pool(env: Env, contributor: Address, pool_id: u64, amount: i128) {
        contributor.require_auth();
        
        let mut pool: GroupPool = env.storage().persistent()
            .get(&DataKey::GroupPool(pool_id))
            .expect("Pool not found");
        
        if !pool.is_active {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Transfer tokens from contributor to contract
        let client = token::Client::new(&env, &pool.token);
        client.transfer(&contributor, &env.current_contract_address(), &amount);
        
        // Update pool balance
        pool.current_balance += amount;
        
        // Track member contribution
        let existing_contribution: i128 = env.storage().persistent()
            .get(&DataKey::GroupPoolMember(pool_id, contributor.clone()))
            .unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::GroupPoolMember(pool_id, contributor.clone()), &(existing_contribution + amount));
        
        // If first time contributing, increment member count
        if existing_contribution == 0 {
            pool.member_count += 1;
        }
        
        env.storage().persistent().set(&DataKey::GroupPool(pool_id), &pool);
        env.storage().persistent().extend_ttl(&DataKey::GroupPool(pool_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Check if target reached
        if pool.current_balance >= pool.target_amount {
            pool.is_active = false; // Close the pool
            env.storage().persistent().set(&DataKey::GroupPool(pool_id), &pool);
            
            // Grant access to all contributors
            Self::grant_pool_access_to_all_members(env.clone(), pool_id, pool.course_id);
            
            // Emit success event
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "GroupPool_TargetReached"), pool.creator, pool.course_id),
                (pool_id, pool.current_balance)
            );
        }
    }

    fn grant_pool_access_to_all_members(env: Env, pool_id: u64, course_id: u64) {
        let pool: GroupPool = env.storage().persistent()
            .get(&DataKey::GroupPool(pool_id))
            .expect("Pool not found");
        
        // Note: In a real implementation, you'd need to iterate through all members
        // For now, we'll mark members as having access when they check
        
        // Grant access to creator
        let current_time = env.ledger().timestamp();
        let one_year_seconds = 31536000; // 365 days
        
        // Create access record for the pool (special marker)
        // Individual members will check against this pool access
        env.storage().persistent().set(
            &DataKey::GroupPoolAccess(pool_id, pool.creator.clone()),
            &true
        );
    }

    pub fn get_pool_access(env: Env, member: Address, pool_id: u64, course_id: u64) -> bool {
        // Check if member has access via this pool
        let has_access: Option<bool> = env.storage().persistent()
            .get(&DataKey::GroupPoolAccess(pool_id, member.clone()));
        
        if has_access.unwrap_or(false) {
            return true;
        }
        
        // Check if pool reached target (even if individual access not set yet)
        if let Some(pool) = env.storage().persistent().get::<DataKey, GroupPool>(&DataKey::GroupPool(pool_id)) {
            if pool.course_id == course_id && pool.current_balance >= pool.target_amount {
                // Grant access on-the-fly
                env.storage().persistent().set(&DataKey::GroupPoolAccess(pool_id, member.clone()), &true);
                return true;
            }
        }
        
        false
    }

    pub fn join_pool_with_access(env: Env, member: Address, pool_id: u64, course_id: u64) {
        member.require_auth();
        
        // Check if pool exists and target is met
        let pool: GroupPool = env.storage().persistent()
            .get(&DataKey::GroupPool(pool_id))
            .expect("Pool not found");
        
        if pool.course_id != course_id {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        if pool.current_balance >= pool.target_amount {
            // Grant access since target is met
            env.storage().persistent().set(&DataKey::GroupPoolAccess(pool_id, member.clone()), &true);
        } else {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
    }

    pub fn get_pool_info(env: Env, pool_id: u64) -> GroupPool {
        let pool: GroupPool = env.storage().persistent()
            .get(&DataKey::GroupPool(pool_id))
            .unwrap_or_else(|| panic!("Pool not found"));
        
        env.storage().persistent().extend_ttl(&DataKey::GroupPool(pool_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        pool
    }

    pub fn get_member_contribution(env: Env, member: Address, pool_id: u64) -> i128 {
        let key = DataKey::GroupPoolMember(pool_id, member);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn close_pool(env: Env, admin: Address, pool_id: u64) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let mut pool: GroupPool = env.storage().persistent()
            .get(&DataKey::GroupPool(pool_id))
            .expect("Pool not found");
        
        pool.is_active = false;
        env.storage().persistent().set(&DataKey::GroupPool(pool_id), &pool);
        
        // Refund contributors
        for i in 0..pool.member_count {
            // In production, you'd need to track member list properly
            // This is a simplified version
        }
    }

    // Quiz Lock Feature - Lock modules until quiz is passed
    
    pub fn configure_module_quiz(env: Env, admin: Address, course_id: u64, module_id: u64, requires_quiz: bool) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        env.storage().persistent().set(&DataKey::ModuleLockConfig(course_id, module_id), &requires_quiz);
        env.storage().persistent().extend_ttl(&DataKey::ModuleLockConfig(course_id, module_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Module_Quiz_Configured"), course_id, module_id),
            requires_quiz
        );
    }

    pub fn submit_quiz_proof(env: Env, student: Address, course_id: u64, module_id: u64, quiz_hash: Symbol, score: u64) {
        student.require_auth();
        
        let current_time = env.ledger().timestamp();
        
        // Create quiz proof record
        let quiz_proof = QuizProof {
            student: student.clone(),
            course_id,
            module_id,
            quiz_hash: quiz_hash.clone(),
            score,
            passed_at: current_time,
            is_verified: true, // In production, this would require verification logic
        };
        
        // Store the quiz proof
        env.storage().persistent().set(&DataKey::ModuleQuizLock(student.clone(), course_id, module_id), &quiz_proof);
        env.storage().persistent().extend_ttl(&DataKey::ModuleQuizLock(student.clone(), course_id, module_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Quiz_Proof_Submitted"), student, course_id),
            (module_id, quiz_hash, score)
        );
    }

    pub fn verify_module_unlocked(env: Env, student: Address, course_id: u64, module_id: u64) -> bool {
        // Check if this module requires a quiz
        let requires_quiz: bool = env.storage().persistent()
            .get(&DataKey::ModuleLockConfig(course_id, module_id))
            .unwrap_or(false);
        
        if !requires_quiz {
            return true; // No quiz required, module is unlocked
        }
        
        // For module 1, always allow access (no prerequisite)
        if module_id == 1 {
            return true;
        }
        
        // Check if previous module's quiz is completed
        let previous_module = module_id - 1;
        
        // Check if student has passed quiz for previous module
        let quiz_proof: Option<QuizProof> = env.storage().persistent()
            .get(&DataKey::ModuleQuizLock(student.clone(), course_id, previous_module));
        
        if let Some(proof) = quiz_proof {
            // Require minimum passing score (e.g., 70%)
            if proof.is_verified && proof.score >= 70 {
                return true;
            }
        }
        
        false // Module is locked
    }

    pub fn get_quiz_proof(env: Env, student: Address, course_id: u64, module_id: u64) -> QuizProof {
        let key = DataKey::ModuleQuizLock(student.clone(), course_id, module_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key).expect("Quiz proof not found")
        } else {
            panic!("Quiz proof not found");
        }
    }

    pub fn get_module_progress(env: Env, student: Address, course_id: u64, total_modules: u64) -> Vec<u64> {
        let mut unlocked_modules = Vec::new(&env);
        
        for module_id in 1..=total_modules {
            if Self::verify_module_unlocked(env.clone(), student.clone(), course_id, module_id) {
                unlocked_modules.push_back(module_id);
            }
        }
        
        unlocked_modules
    }

    pub fn lock_module(env: Env, admin: Address, student: Address, course_id: u64, module_id: u64) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Remove quiz proof if exists (force re-lock)
        let key = DataKey::ModuleQuizLock(student.clone(), course_id, module_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().remove(&key);
            
            // Emit event
            #[allow(deprecated)]
            env.events().publish(
                (Symbol::new(&env, "Module_Locked"), student, course_id),
                module_id
            );
        }
    }

    pub fn batch_submit_quiz_proofs(env: Env, student: Address, course_id: u64, module_ids: Vec<u64>, quiz_hashes: Vec<Symbol>, scores: Vec<u64>) {
        student.require_auth();
        
        if module_ids.len() != quiz_hashes.len() || module_ids.len() != scores.len() {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let current_time = env.ledger().timestamp();
        
        for i in 0..module_ids.len() {
            let module_id = module_ids.get(i).unwrap();
            let quiz_hash = quiz_hashes.get(i).unwrap();
            let score = scores.get(i).unwrap();
            
            let quiz_proof = QuizProof {
                student: student.clone(),
                course_id,
                module_id,
                quiz_hash: quiz_hash.clone(),
                score,
                passed_at: current_time,
                is_verified: true,
            };
            
            env.storage().persistent().set(&DataKey::ModuleQuizLock(student.clone(), course_id, module_id), &quiz_proof);
            env.storage().persistent().extend_ttl(&DataKey::ModuleQuizLock(student.clone(), course_id, module_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        }
        
        // Emit event
        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Batch_Quiz_Proofs_Submitted"), student, course_id),
            module_ids.len()
        );
    }

    // Multi-Sig Academic Board Review Functions

    pub fn init_deans_council(env: Env, admin: Address, members: Vec<Address>, required_signatures: u32) {
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

        // Validate members count (should be 3 for 2-of-3)
        if members.len() != 3 {
            panic!("Deans Council must have exactly 3 members");
        }

        // Validate required signatures (should be 2 for 2-of-3)
        if required_signatures != 2 {
            panic!("Required signatures must be 2 for 2-of-3 multisig");
        }

        let council = DeansCouncil {
            members: members.clone(),
            required_signatures,
            is_active: true,
        };

        env.storage()
            .instance()
            .set(&DataKey::DeansCouncil, &council);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Deans_Council_Initialized"), admin),
            (members, required_signatures)
        );
    }

    pub fn get_deans_council(env: Env) -> Option<DeansCouncil> {
        env.storage().instance().get(&DataKey::DeansCouncil)
    }

    fn is_council_member(env: &Env, member: &Address) -> bool {
        if let Some(council) = env.storage().instance().get(&DataKey::DeansCouncil) {
            council.members.contains(member) && council.is_active
        } else {
            false
        }
    }

    pub fn board_pause_request(env: Env, council_member: Address, student: Address, reason: Symbol) {
        council_member.require_auth();

        // Verify caller is authorized council member
        if !Self::is_council_member(&env, &council_member) {
            panic!("Unauthorized: Not a council member");
        }

        // Check if there's already a pending request for this student
        if let Some(existing_request) = env.storage().persistent().get(&DataKey::BoardPauseRequest(student.clone())) {
            if !existing_request.is_executed {
                panic!("Pause request already pending for this student");
            }
        }

        let current_time = env.ledger().timestamp();
        
        let mut request = BoardPauseRequest {
            student: student.clone(),
            reason: reason.clone(),
            requested_at: current_time,
            signatures: Vec::new(&env),
            is_executed: false,
            executed_at: None,
        };

        // Add the first signature
        request.signatures.push_back(council_member.clone());

        env.storage()
            .persistent()
            .set(&DataKey::BoardPauseRequest(student.clone()), &request);

        // Record the signature
        env.storage()
            .persistent()
            .set(&DataKey::BoardSignature(student.clone(), council_member), &true);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Board_Pause_Requested"), council_member, student),
            reason
        );
    }

    pub fn board_pause_sign(env: Env, council_member: Address, student: Address) {
        council_member.require_auth();

        // Verify caller is authorized council member
        if !Self::is_council_member(&env, &council_member) {
            panic!("Unauthorized: Not a council member");
        }

        // Check if this member already signed
        if let Some(_) = env.storage().persistent().get(&DataKey::BoardSignature(student.clone(), council_member.clone())) {
            panic!("Council member has already signed this request");
        }

        // Get the existing request
        let mut request: BoardPauseRequest = env
            .storage()
            .persistent()
            .get(&DataKey::BoardPauseRequest(student.clone()))
            .unwrap_or_else(|| panic!("No pause request found for this student"));

        if request.is_executed {
            panic!("Request has already been executed");
        }

        // Add the signature
        request.signatures.push_back(council_member.clone());

        // Record the signature
        env.storage()
            .persistent()
            .set(&DataKey::BoardSignature(student.clone(), council_member), &true);

        // Check if we have enough signatures to execute
        let council: DeansCouncil = env
            .storage()
            .instance()
            .get(&DataKey::DeansCouncil)
            .expect("Deans Council not initialized");

        if request.signatures.len() >= council.required_signatures as usize {
            // Execute the pause
            Self::execute_board_pause(env.clone(), student.clone(), request.clone());
            request.is_executed = true;
            request.executed_at = Some(env.ledger().timestamp());
        }

        env.storage()
            .persistent()
            .set(&DataKey::BoardPauseRequest(student), &request);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Board_Pause_Signed"), council_member, student),
            request.signatures.len()
        );
    }

    fn execute_board_pause(env: Env, student: Address, request: BoardPauseRequest) {
        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        scholarship.is_paused = true;
        scholarship.is_disputed = true;
        scholarship.dispute_reason = Some(request.reason);

        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student.clone()), &scholarship);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Board_Pause_Executed"), student),
            (request.signatures.len(), request.reason)
        );
    }

    pub fn upload_final_ruling(env: Env, admin: Address, student: Address, ruling: Symbol) {
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

        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");

        if !scholarship.is_disputed {
            panic!("Scholarship is not in disputed state");
        }

        scholarship.final_ruling = Some(ruling.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student.clone()), &scholarship);

        #[allow(deprecated)]
        env.events().publish(
            (Symbol::new(&env, "Final_Ruling_Uploaded"), admin, student),
            ruling
        );
    }

    pub fn get_board_pause_request(env: Env, student: Address) -> Option<BoardPauseRequest> {
        let key = DataKey::BoardPauseRequest(student);
        let request = env.storage().persistent().get(&key);
        if request.is_some() {
            env.storage()
                .persistent()
                .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        }
        request
    }

    pub fn is_disputed(env: Env, student: Address) -> bool {
        let scholarship: Option<Scholarship> = env.storage().persistent().get(&DataKey::Scholarship(student));
        scholarship.map_or(false, |s| s.is_disputed)
    }
}

mod test;
mod tuition_stipend_split_tests;
mod student_profile_nft;
