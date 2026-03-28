#![no_std]
use core::convert::TryFrom;
use expiry_math::{checked_access_expiry, checked_subscription_expiry};
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol, Vec, IntoVal};

// --- Constants for TTL Management ---
const LEDGER_BUMP_THRESHOLD: u32 = 123456; // Example threshold
const LEDGER_BUMP_EXTEND: u32 = 789012;    // Example extension
const MAX_COURSE_REGISTRY_SIZE: u64 = 1000;
const EARLY_DROP_WINDOW_SECONDS: u64 = 300; // 5 minutes

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
    StreakBonusAmount,
    ConsecutiveDays(Address, u64),
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
        // Configuration uses instance storage
        let storage = env.storage().instance();
        storage.set(&DataKey::BaseRate, &base_rate);
        storage.set(&DataKey::DiscountThreshold, &discount_threshold);
        storage.set(&DataKey::DiscountPercentage, &discount_percentage);
        storage.set(&DataKey::MinDeposit, &min_deposit);
        storage.set(&DataKey::HeartbeatInterval, &heartbeat_interval);
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

        // Access record uses persistent storage for user-specific data
        let access_key = DataKey::Access(student.clone(), course_id);
        let mut access = env.storage().persistent().get(&access_key).unwrap_or(Access {
            student: student.clone(),
            course_id,
            expiry_time: current_time,
            token: token.clone(),
            total_watch_time: 0,
            last_heartbeat: 0,
            last_purchase_time: 0,
        });

        // Use hardened expiry math
        access.expiry_time = checked_access_expiry(current_time, access.expiry_time, seconds_bought)
            .expect("Expiry calculation failed");
        
        access.last_purchase_time = current_time;

        env.storage().persistent().set(&access_key, &access);
        env.storage().persistent().extend_ttl(&access_key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);

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
            base_rate
        }
    }

    fn has_active_subscription(env: Env, student: Address, course_id: u64) -> bool {
        let sub_key = DataKey::Subscription(student);
        if let Some(sub) = env.storage().persistent().get::<_, SubscriptionTier>(&sub_key) {
            return env.ledger().timestamp() < sub.expiry_time && sub.course_ids.contains(&course_id);
        }
        false
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

    fn is_admin(env: &Env, caller: &Address) -> bool {
        let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        admin.map_or(false, |a| a == *caller)
    }
}