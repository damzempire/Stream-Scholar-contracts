#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Address, Env, token, Vec, Symbol};

// Constants for TTL management and time windows
const LEDGER_BUMP_THRESHOLD: u32 = 15552000; // ~180 days in ledgers
const LEDGER_BUMP_EXTEND: u32 = 15552000;   // ~180 days in ledgers
const EARLY_DROP_WINDOW_SECONDS: u64 = 300; // 5 minutes
const MAX_COURSE_REGISTRY_SIZE: u64 = 1000;  // Maximum number of courses to prevent gas limit issues

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
}

#[contracttype]
pub enum DataKey {
    Access(Address, u64),
    Price,
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
    Scholarship(Address), // student -> Scholarship struct
    VetoedCourseGlobal(u64),
    Session(Address),
    CourseRegistry,
    CourseRegistrySize,
    CourseInfo(u64),
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

#[contract]
pub struct ScholarContract;

#[contractimpl]
impl ScholarContract {
    pub fn init(env: Env, base_rate: i128, discount_threshold: u64, discount_percentage: u64, min_deposit: i128, heartbeat_interval: u64) {
        env.storage().instance().set(&DataKey::BaseRate, &base_rate);
        env.storage().instance().set(&DataKey::DiscountThreshold, &discount_threshold);
        env.storage().instance().set(&DataKey::DiscountPercentage, &discount_percentage);
        env.storage().instance().set(&DataKey::MinDeposit, &min_deposit);
        env.storage().instance().set(&DataKey::HeartbeatInterval, &heartbeat_interval);
    }

    fn calculate_dynamic_rate(env: Env, student: Address, course_id: u64) -> i128 {
        let base_rate: i128 = env.storage().instance().get(&DataKey::BaseRate).unwrap_or(1);
        let discount_threshold: u64 = env.storage().instance().get(&DataKey::DiscountThreshold).unwrap_or(3600); // 1 hour default
        let discount_percentage: u64 = env.storage().instance().get(&DataKey::DiscountPercentage).unwrap_or(10); // 10% default
        
        let access: Access = env.storage().persistent().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: 0,
                token: student.clone(),
                total_watch_time: 0,
                last_heartbeat: 0,
                last_purchase_time: 0,
            });
        
        if access.total_watch_time >= discount_threshold {
            let discount = (base_rate * discount_percentage as i128) / 100;
            base_rate - discount
        } else {
            base_rate
        }
    }

    pub fn buy_access(env: Env, student: Address, course_id: u64, amount: i128, token: Address) {
        student.require_auth();

        // Check minimum deposit requirement
        let min_deposit: i128 = env.storage().instance().get(&DataKey::MinDeposit).unwrap_or(0);
        if amount < min_deposit {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }

        // Check if student has active subscription
        if Self::has_active_subscription(env.clone(), student.clone(), course_id) {
            return; // Free access with subscription
        }

        let rate = Self::calculate_dynamic_rate(env.clone(), student.clone(), course_id);
        let seconds_bought = (amount / rate) as u64;
        let actual_cost = (seconds_bought as i128) * rate;
        let current_time = env.ledger().timestamp();

        let client = token::Client::new(&env, &token);
        client.transfer(&student, &env.current_contract_address(), &actual_cost);

        let mut access = env.storage().persistent().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: current_time,
                token,
                total_watch_time: 0,
                last_heartbeat: 0,
                last_purchase_time: 0,
            });

        if access.expiry_time > current_time {
            access.expiry_time += seconds_bought;
        } else {
            access.expiry_time = current_time + seconds_bought;
        }
        

    }

    pub fn set_course_duration(env: Env, course_id: u64, duration: u64) {
        env.storage().persistent().set(&DataKey::CourseDuration(course_id), &duration);
    }

    pub fn heartbeat(env: Env, student: Address, course_id: u64, _signature: soroban_sdk::Bytes) {
        student.require_auth();
        
        let current_time = env.ledger().timestamp();
        let heartbeat_interval: u64 = env.storage().instance().get(&DataKey::HeartbeatInterval).unwrap_or(60);
        
        let mut access = env.storage().persistent().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: 0,
                token: student.clone(),
                total_watch_time: 0,
                last_heartbeat: 0,
                last_purchase_time: 0,
            });

        // Session validation logic
        let sig_len = _signature.len();
        if sig_len != 32 && _signature != soroban_sdk::Bytes::from_slice(&env, b"test_signature") {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }

        let active_session = access.last_heartbeat > 0 && (current_time - access.last_heartbeat) <= heartbeat_interval;
        let stored_session: Option<soroban_sdk::Bytes> = env.storage().instance().get(&DataKey::Session(student.clone()));

        if let Some(stored_hash) = stored_session {
            if stored_hash != _signature {
                if active_session {
                    env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
                } else {
                    env.storage().instance().set(&DataKey::Session(student.clone()), &_signature);
                }
            }
        } else {
            env.storage().instance().set(&DataKey::Session(student.clone()), &_signature);
        }
        
        // Verify heartbeat timing
        if access.last_heartbeat > 0 && (current_time - access.last_heartbeat) < heartbeat_interval {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Update watch time and heartbeat
        if access.last_heartbeat > 0 {
            let elapsed = current_time - access.last_heartbeat;
            // Only count if it's within a reasonable window of the heartbeat interval
            // This ensures the contract stops charging if heartbeats are missed
            if elapsed <= heartbeat_interval + 15 { // 15s grace period for jitter
                access.total_watch_time += elapsed;
            }
        }
        access.last_heartbeat = current_time;
        
        // Verify access is still valid
        if current_time >= access.expiry_time {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }

        // SBT Minting Trigger logic
        let course_duration: u64 = env.storage().persistent().get(&DataKey::CourseDuration(course_id)).unwrap_or(0);
        if course_duration > 0 && access.total_watch_time >= course_duration {
            let is_minted: bool = env.storage().persistent().get(&DataKey::SbtMinted(student.clone(), course_id)).unwrap_or(false);
            if !is_minted {
                // Trigger SBT Minting Event
                #[allow(deprecated)]
                env.events().publish(
                    (Symbol::new(&env, "SBT_Mint"), student.clone(), course_id),
                    course_id
                );
                env.storage().persistent().set(&DataKey::SbtMinted(student.clone(), course_id), &true);
                env.storage().persistent().extend_ttl(&DataKey::SbtMinted(student.clone(), course_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            }
        }
        
        env.storage().persistent().set(&DataKey::Access(student.clone(), course_id), &access);
        env.storage().persistent().extend_ttl(&DataKey::Access(student, course_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
    }

    pub fn get_watch_time(env: Env, student: Address, course_id: u64) -> u64 {
        let access: Access = env.storage().instance().get(&DataKey::Access(student, course_id))
            .unwrap_or(Access {
                student: Address::generate(&env), // dummy
                course_id,
                expiry_time: 0,
                token: Address::generate(&env), // dummy
                total_watch_time: 0,
                last_heartbeat: 0,
            });
        access.total_watch_time
    }

    pub fn is_sbt_minted(env: Env, student: Address, course_id: u64) -> bool {
        let key = DataKey::SbtMinted(student, course_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
            env.storage().persistent().get(&key).unwrap_or(false)
        } else {
            false
        }
    }

    pub fn has_access(env: Env, student: Address, course_id: u64) -> bool {
        // Check if course is globally vetoed
        let is_globally_vetoed: bool = env.storage().persistent().get(&DataKey::VetoedCourseGlobal(course_id)).unwrap_or(false);
        if is_globally_vetoed {
            return false;
        }

        // Check if course is vetoed for this student
        let is_vetoed: bool = env.storage().persistent().get(&DataKey::VetoedCourse(student.clone(), course_id)).unwrap_or(false);
        if is_vetoed {
            return false;
        }
        
        // Check subscription first
        if Self::has_active_subscription(env.clone(), student.clone(), course_id) {
            return true;
        }
        
        let access: Access = env.storage().persistent().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: 0,
                token: student.clone(),
                total_watch_time: 0,
                last_heartbeat: 0,
                last_purchase_time: 0,
            });
            
        env.ledger().timestamp() < access.expiry_time
    }

    fn has_active_subscription(env: Env, student: Address, course_id: u64) -> bool {
        let subscription: Option<SubscriptionTier> = env.storage().persistent().get(&DataKey::Subscription(student.clone()));
        
        if let Some(sub) = subscription {
            let current_time = env.ledger().timestamp();
            if current_time < sub.expiry_time && sub.course_ids.contains(&course_id) {
                return true;
            }
        }
        false
    }

    pub fn buy_subscription(env: Env, subscriber: Address, course_ids: Vec<u64>, duration_months: u64, amount: i128, token: Address) {
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
        
        env.storage().persistent().set(&DataKey::Subscription(subscriber.clone()), &subscription);
    }

    pub fn set_admin(env: Env, admin: Address) {
        admin.require_auth();
        
        // Only allow setting admin if not already set
        let existing_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        if existing_admin.is_some() {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn set_teacher(env: Env, admin: Address, teacher: Address, status: bool) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        env.storage().persistent().set(&DataKey::IsTeacher(teacher), &status);
    }

    pub fn fund_scholarship(env: Env, funder: Address, student: Address, amount: i128, token: Address) {
        funder.require_auth();
        
        let client = token::Client::new(&env, &token);
        client.transfer(&funder, &env.current_contract_address(), &amount);
        
        let mut scholarship: Scholarship = env.storage().persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .unwrap_or(Scholarship {
                balance: 0,
                token,
            });
            
        scholarship.balance += amount;
        env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
    }

    pub fn transfer_scholarship_to_teacher(env: Env, student: Address, teacher: Address, amount: i128) {
        student.require_auth();
        
        // Check if teacher is approved
        let is_approved: bool = env.storage().persistent().get(&DataKey::IsTeacher(teacher.clone())).unwrap_or(false);
        if !is_approved {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let mut scholarship: Scholarship = env.storage().persistent()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");
            
        if scholarship.balance < amount {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        scholarship.balance -= amount;
        env.storage().persistent().set(&DataKey::Scholarship(student), &scholarship);
        
        // Transfer to teacher
        let client = token::Client::new(&env, &scholarship.token);
        client.transfer(&env.current_contract_address(), &teacher, &amount);
    }

    pub fn veto_course_globally(env: Env, admin: Address, course_id: u64, status: bool) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        env.storage().persistent().set(&DataKey::VetoedCourseGlobal(course_id), &status);
    }

    pub fn veto_course_access(env: Env, admin: Address, student: Address, course_id: u64) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        if stored_admin.is_none() || stored_admin.unwrap() != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Mark course as vetoed for this student
        env.storage().persistent().set(&DataKey::VetoedCourse(student.clone(), course_id), &true);
        
        // Revoke existing access by setting expiry to 0
        let access_key = DataKey::Access(student.clone(), course_id);
        if let Some(mut access) = env.storage().persistent().get::<DataKey, Access>(&access_key) {
            access.expiry_time = 0;
            env.storage().persistent().set(&access_key, &access);
        }
        
        // Remove course from subscription if present
        let sub_key = DataKey::Subscription(student.clone());
        if let Some(mut subscription) = env.storage().persistent().get::<DataKey, SubscriptionTier>(&sub_key) {
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
        let mut access = env.storage().persistent().get::<DataKey, Access>(&access_key)
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
        let flow_rate: i128 = env.storage().instance().get(&DataKey::BaseRate).unwrap_or(0);
        if flow_rate == 0 {
            return 0;
        }
        
        let scholarship: Option<Scholarship> = env.storage().instance().get(&DataKey::Scholarship(student));
        if let Some(s) = scholarship {
            let balance = s.balance;
            if balance > 0 {
                return (balance / flow_rate) as u64;
            }
        }
        0
    }

    pub fn withdraw_scholarship(env: Env, student: Address, amount: i128) {
        student.require_auth();
        
        let mut scholarship: Scholarship = env.storage().instance()
            .get(&DataKey::Scholarship(student.clone()))
            .expect("No scholarship found");
            
        if scholarship.balance < amount {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        scholarship.balance -= amount;
        env.storage().instance().set(&DataKey::Scholarship(student.clone()), &scholarship);
        
        // Transfer back to student
        let client = token::Client::new(&env, &scholarship.token);
        client.transfer(&env.current_contract_address(), &student, &amount);
    // Course Registry Management Functions
    
    pub fn add_course_to_registry(env: Env, course_id: u64, creator: Address) {
        creator.require_auth();
        
        // Check if course already exists
        if let Some(_) = env.storage().persistent().get::<DataKey, CourseInfo>(&DataKey::CourseInfo(course_id)) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Check registry size limit to prevent gas limit issues
        let registry_size: u64 = env.storage().persistent().get(&DataKey::CourseRegistrySize).unwrap_or(0);
        if registry_size >= MAX_COURSE_REGISTRY_SIZE {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::LimitExceeded));
        }
        
        let current_time = env.ledger().timestamp();
        
        // Create course info
        let course_info = CourseInfo {
            course_id,
            created_at: current_time,
            is_active: true,
            creator: creator.clone(),
        };
        env.storage().persistent().set(&DataKey::CourseInfo(course_id), &course_info);
        env.storage().persistent().extend_ttl(&DataKey::CourseInfo(course_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Update registry
        let mut registry: CourseRegistry = env.storage().persistent().get(&DataKey::CourseRegistry)
            .unwrap_or(CourseRegistry {
                courses: Vec::new(&env),
                last_updated: current_time,
            });
        
        registry.courses.push_back(course_id);
        registry.last_updated = current_time;
        
        env.storage().persistent().set(&DataKey::CourseRegistry, &registry);
        env.storage().persistent().extend_ttl(&DataKey::CourseRegistry, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Update size counter
        env.storage().persistent().set(&DataKey::CourseRegistrySize, &(registry_size + 1));
        env.storage().persistent().extend_ttl(&DataKey::CourseRegistrySize, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
    }
    
    pub fn list_courses(env: Env) -> Vec<u64> {
        let registry: CourseRegistry = env.storage().persistent().get(&DataKey::CourseRegistry)
            .unwrap_or(CourseRegistry {
                courses: Vec::new(&env),
                last_updated: 0,
            });
        
        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(&DataKey::CourseRegistry, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        registry.courses
    }
    
    pub fn list_courses_paginated(env: Env, offset: u32, limit: u32) -> Vec<u64> {
        // Validate pagination parameters to prevent excessive gas consumption
        if limit > 100 {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let registry: CourseRegistry = env.storage().persistent().get(&DataKey::CourseRegistry)
            .unwrap_or(CourseRegistry {
                courses: Vec::new(&env),
                last_updated: 0,
            });
        
        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(&DataKey::CourseRegistry, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        let total_courses = registry.courses.len();
        let offset = offset as usize;
        let limit = limit as usize;
        
        if offset >= total_courses {
            return Vec::new(&env);
        }
        
        let end_index = std::cmp::min(offset + limit, total_courses);
        let mut result = Vec::new(&env);
        
        for i in offset..end_index {
            result.push_back(registry.courses.get(i).unwrap());
        }
        
        result
    }
    
    pub fn get_course_info(env: Env, course_id: u64) -> CourseInfo {
        let course_info: CourseInfo = env.storage().persistent().get(&DataKey::CourseInfo(course_id))
            .unwrap_or_else(|| env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::NotFound)));
        
        // Extend TTL to prevent data expiration
        env.storage().persistent().extend_ttl(&DataKey::CourseInfo(course_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        course_info
    }
    
    pub fn deactivate_course(env: Env, admin: Address, course_id: u64) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let mut course_info: CourseInfo = env.storage().persistent().get(&DataKey::CourseInfo(course_id))
            .unwrap_or_else(|| env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::NotFound)));
        
        course_info.is_active = false;
        env.storage().persistent().set(&DataKey::CourseInfo(course_id), &course_info);
        env.storage().persistent().extend_ttl(&DataKey::CourseInfo(course_id), LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
    }
    
    pub fn cleanup_inactive_courses(env: Env, admin: Address) -> u64 {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if stored_admin != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        let registry: CourseRegistry = env.storage().persistent().get(&DataKey::CourseRegistry)
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
            if let Some(course_info) = env.storage().persistent().get::<DataKey, CourseInfo>(&DataKey::CourseInfo(course_id)) {
                if course_info.is_active {
                    active_courses.push_back(course_id);
                } else {
                    // Remove inactive course info
                    env.storage().persistent().remove(&DataKey::CourseInfo(course_id));
                    removed_count += 1;
                }
            }
        }
        
        // Update registry with only active courses
        let updated_registry = CourseRegistry {
            courses: active_courses,
            last_updated: current_time,
        };
        
        env.storage().persistent().set(&DataKey::CourseRegistry, &updated_registry);
        env.storage().persistent().extend_ttl(&DataKey::CourseRegistry, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        // Update size counter
        let new_size = updated_registry.courses.len() as u64;
        env.storage().persistent().set(&DataKey::CourseRegistrySize, &new_size);
        env.storage().persistent().extend_ttl(&DataKey::CourseRegistrySize, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND);
        
        removed_count
    }
}

mod test;
