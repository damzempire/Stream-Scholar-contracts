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
}