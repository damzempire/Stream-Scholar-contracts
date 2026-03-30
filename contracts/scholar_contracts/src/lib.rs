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

    }
}
