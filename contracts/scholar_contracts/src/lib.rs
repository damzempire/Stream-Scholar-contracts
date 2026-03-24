#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Address, Env, token, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Access {
    pub student: Address,
    pub course_id: u64,
    pub expiry_time: u64,
    pub token: Address,
    pub total_watch_time: u64,
    pub last_heartbeat: u64,
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
    Admin,
    VetoedCourse(Address, u64),
}

#[contracttype]
#[derive(Clone)]
pub struct SubscriptionTier {
    pub subscriber: Address,
    pub expiry_time: u64,
    pub course_ids: Vec<u64>,
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

        let client = token::Client::new(&env, &token);
        client.transfer(&student, &env.current_contract_address(), &amount);

        let rate = Self::calculate_dynamic_rate(env.clone(), student.clone(), course_id);
        let seconds_bought = (amount / rate) as u64;
        let current_time = env.ledger().timestamp();

        let mut access = env.storage().instance().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: current_time,
                token,
                total_watch_time: 0,
                last_heartbeat: 0,
            });

        if access.expiry_time > current_time {
            access.expiry_time += seconds_bought;
        } else {
            access.expiry_time = current_time + seconds_bought;
        }

        env.storage().instance().set(&DataKey::Access(student, course_id), &access);
    }

    pub fn heartbeat(env: Env, student: Address, course_id: u64, _signature: soroban_sdk::Bytes) {
        student.require_auth();
        
        let current_time = env.ledger().timestamp();
        let heartbeat_interval: u64 = env.storage().instance().get(&DataKey::HeartbeatInterval).unwrap_or(60);
        
        let mut access = env.storage().instance().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: 0,
                token: student.clone(),
                total_watch_time: 0,
                last_heartbeat: 0,
            });
        
        // Verify heartbeat timing
        if access.last_heartbeat > 0 && (current_time - access.last_heartbeat) < heartbeat_interval {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Update watch time and heartbeat
        if access.last_heartbeat > 0 {
            access.total_watch_time += current_time - access.last_heartbeat;
        }
        access.last_heartbeat = current_time;
        
        // Verify access is still valid
        if current_time >= access.expiry_time {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        env.storage().instance().set(&DataKey::Access(student, course_id), &access);
    }

    pub fn has_access(env: Env, student: Address, course_id: u64) -> bool {
        // Check if course is vetoed for this student
        let is_vetoed: bool = env.storage().instance().get(&DataKey::VetoedCourse(student.clone(), course_id)).unwrap_or(false);
        if is_vetoed {
            return false;
        }
        
        // Check subscription first
        if Self::has_active_subscription(env.clone(), student.clone(), course_id) {
            return true;
        }
        
        let access: Access = env.storage().instance().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: 0,
                token: student.clone(),
                total_watch_time: 0,
                last_heartbeat: 0,
            });
            
        env.ledger().timestamp() < access.expiry_time
    }

    fn has_active_subscription(env: Env, student: Address, course_id: u64) -> bool {
        let subscription: Option<SubscriptionTier> = env.storage().instance().get(&DataKey::Subscription(student.clone()));
        
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
        
        env.storage().instance().set(&DataKey::Subscription(subscriber.clone()), &subscription);
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

    pub fn veto_course_access(env: Env, admin: Address, student: Address, course_id: u64) {
        admin.require_auth();
        
        // Verify caller is admin
        let stored_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        if stored_admin.is_none() || stored_admin.unwrap() != admin {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Mark course as vetoed for this student
        env.storage().instance().set(&DataKey::VetoedCourse(student.clone(), course_id), &true);
        
        // Revoke existing access by setting expiry to 0
        let access_key = DataKey::Access(student.clone(), course_id);
        if let Some(mut access) = env.storage().instance().get::<DataKey, Access>(&access_key) {
            access.expiry_time = 0;
            env.storage().instance().set(&access_key, &access);
        }
        
        // Remove course from subscription if present
        let sub_key = DataKey::Subscription(student.clone());
        if let Some(mut subscription) = env.storage().instance().get::<DataKey, SubscriptionTier>(&sub_key) {
            // Filter out the vetoed course
            let mut new_course_ids = Vec::new(&env);
            for i in 0..subscription.course_ids.len() {
                let cid = subscription.course_ids.get(i).unwrap();
                if cid != course_id {
                    new_course_ids.push_back(cid);
                }
            }
            subscription.course_ids = new_course_ids;
            env.storage().instance().set(&sub_key, &subscription);
        }
    }
}

mod test;
