use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, token,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    StudentProfile(Address), // Optimized: Groups metrics into a single persistent storage key
    ResearchGrant(u64),      // Maps grant_id to ResearchGrant details
    MarketplaceAuth(u64),    // Temporary authorization for NFT marketplace listing
    SkillBadge(Address, Symbol), // student, badge_name -> earned status
    Milestone(Address, Symbol),  // student, milestone_type -> verification status
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudentProfile {
    pub academic_points: u64,
    pub courses_completed: u64,
    pub current_streak: u64,
    pub last_activity: u64,
    pub book_voucher_claimed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchGrant {
    pub student_researcher: Address,
    pub current_beneficiary: Address, // Beneficiary can be sold to an investor
    pub total_amount: i128,
    pub token: Address,
    pub is_locked: bool,
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
        
        // Temporary authorization for the marketplace to initiate the lock
        env.storage().temporary().set(&DataKey::MarketplaceAuth(grant_id), &marketplace);
    }

    /// #117: Marketplace utility to lock the grant during the auction process.
    pub fn lock_grant_for_auction(env: Env, marketplace: Address, grant_id: u64) {
        marketplace.require_auth();
        let authorized_market: Address = env.storage().temporary().get(&DataKey::MarketplaceAuth(grant_id))
            .expect("Marketplace not authorized for this grant");
        assert_eq!(authorized_market, marketplace, "Unauthorized marketplace access");

        let mut grant: ResearchGrant = env.storage().persistent().get(&DataKey::ResearchGrant(grant_id)).unwrap();
        grant.is_locked = true;
        env.storage().persistent().set(&DataKey::ResearchGrant(grant_id), &grant);
    }

    /// #117: Updates the beneficiary to an investor after a successful marketplace auction.
    pub fn transfer_grant_beneficiary(env: Env, marketplace: Address, grant_id: u64, new_investor: Address) {
        marketplace.require_auth();
        let authorized_market: Address = env.storage().temporary().get(&DataKey::MarketplaceAuth(grant_id)).unwrap();
        assert_eq!(authorized_market, marketplace);

        let mut grant: ResearchGrant = env.storage().persistent().get(&DataKey::ResearchGrant(grant_id)).unwrap();
        assert!(grant.is_locked, "Grant must be locked by marketplace for transfer");

        grant.current_beneficiary = new_investor;
        grant.is_locked = false; // Grant unlocked, future drips flow to investor
        env.storage().persistent().set(&DataKey::ResearchGrant(grant_id), &grant);
    }

    /// #120: Skill-Badge Vesting Trigger
    /// Releases 1,000 tokens when an oracle verifies a specific skill (e.g., "AWS Certified").
    /// Idempotent logic ensures the reward is only paid once.
    pub fn verify_skill_badge(env: Env, oracle: Address, student: Address, badge_name: Symbol, token_addr: Address) {
        oracle.require_auth();
        
        let badge_key = DataKey::SkillBadge(student.clone(), badge_name.clone());
        if !env.storage().persistent().has(&badge_key) {
            env.storage().persistent().set(&badge_key, &true);
            
            // Skill-based reward: 1,000 tokens (assuming 7 decimal precision)
            let reward_amount: i128 = 1000_0000000;
            let client = token::Client::new(&env, &token_addr);
            client.transfer(&env.current_contract_address(), &student, &reward_amount);

            env.events().publish((symbol_short!("skill"), student), badge_name);
        }
    }

    /// #113: Milestone-Gated Book Voucher Support
    /// Allows admins to verify documentation like "Class Schedule".
    pub fn verify_milestone_document(env: Env, admin: Address, student: Address, milestone_type: Symbol) {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        stored_admin.require_auth();
        
        env.storage().persistent().set(&DataKey::Milestone(student, milestone_type), &true);
    }

    /// #113: Conditional Release of the $200 Book Voucher.
    /// Only releases if the schedule milestone is verified.
    pub fn claim_book_voucher(env: Env, student: Address, token_addr: Address) {
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

    pub fn create_grant_for_test(env: Env, grant_id: u64, student: Address, amount: i128, token_addr: Address) {
        let grant = ResearchGrant { student_researcher: student.clone(), current_beneficiary: student, total_amount: amount, token: token_addr, is_locked: false };
        env.storage().persistent().set(&DataKey::ResearchGrant(grant_id), &grant);
    }
}