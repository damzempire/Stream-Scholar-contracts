#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Address, Env, token, Vec, Bytes};

#[contracttype]
#[derive(Clone)]
pub struct BookStipendVoucher {
    pub voucher_id: u64,
    pub donor: Address,
    pub student: Address,
    pub amount: i128,
    pub book_token_address: Address,
    pub verified_bookstores: Vec<Address>,
    pub created_at: u64,
    pub expiry_time: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct ZKGPAProof {
    pub student: Address,
    pub proof_hash: Bytes,
    public_inputs: Vec<u64>,
    verification_level: u64, // 3.5 = 35, 4.0 = 40, etc.
    verified_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct SoulboundCredential {
    pub credential_id: u64,
    pub student: Address,
    pub total_hours_funded: u64,
    pub major: Bytes,
    pub donor_organization: Address,
    pub graduation_date: u64,
    pub metadata_url: Bytes,
}

#[contracttype]
#[derive(Clone)]
pub struct LearningVelocityScore {
    pub student: Address,
    pub score: u64,
    pub courses_completed: u64,
    pub avg_completion_time: u64,
    pub last_updated: u64,
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
    BookStipendVoucher(u64),
    ZKGPAProof(Address),
    SoulboundCredential(u64),
    LearningVelocityScore(Address),
    VoucherCounter,
    CredentialCounter,
    VerifiedBookstores,
    GrantStreamContract,
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

    // Issue #88: Multi-Token Book Stipend Voucher
    pub fn create_book_stipend_voucher(env: Env, donor: Address, student: Address, amount: i128, book_token_address: Address, duration_days: u64) {
        donor.require_auth();
        
        // Transfer book tokens to contract
        let book_token_client = token::Client::new(&env, &book_token_address);
        book_token_client.transfer(&donor, &env.current_contract_address(), &amount);
        
        let current_time = env.ledger().timestamp();
        let expiry_time = current_time + (duration_days * 24 * 60 * 60);
        
        let voucher_counter: u64 = env.storage().instance().get(&DataKey::VoucherCounter).unwrap_or(0);
        let voucher_id = voucher_counter + 1;
        
        let verified_bookstores: Vec<Address> = env.storage().instance().get(&DataKey::VerifiedBookstores)
            .unwrap_or(Vec::new(&env));
        
        let voucher = BookStipendVoucher {
            voucher_id,
            donor: donor.clone(),
            student: student.clone(),
            amount,
            book_token_address: book_token_address.clone(),
            verified_bookstores,
            created_at: current_time,
            expiry_time,
        };
        
        env.storage().instance().set(&DataKey::BookStipendVoucher(voucher_id), &voucher);
        env.storage().instance().set(&DataKey::VoucherCounter, &voucher_id);
    }
    
    pub fn redeem_book_stipend(env: Env, voucher_id: u64, bookstore_address: Address) {
        let voucher: BookStipendVoucher = env.storage().instance().get(&DataKey::BookStipendVoucher(voucher_id))
            .unwrap_or_else(|| env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction)));
        
        let current_time = env.ledger().timestamp();
        
        // Check if voucher is still valid
        if current_time >= voucher.expiry_time {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Verify bookstore is in the approved list
        if !voucher.verified_bookstores.contains(&bookstore_address) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // Transfer book tokens to verified bookstore
        let book_token_client = token::Client::new(&env, &voucher.book_token_address);
        book_token_client.transfer(&env.current_contract_address(), &bookstore_address, &voucher.amount);
        
        // Remove voucher after redemption
        env.storage().instance().remove(&DataKey::BookStipendVoucher(voucher_id));
    }
    
    pub fn add_verified_bookstore(env: Env, admin: Address, bookstore_address: Address) {
        // Simple admin check - in production, use proper access control
        admin.require_auth();
        
        let mut verified_bookstores: Vec<Address> = env.storage().instance().get(&DataKey::VerifiedBookstores)
            .unwrap_or(Vec::new(&env));
        
        if !verified_bookstores.contains(&bookstore_address) {
            verified_bookstores.push_back(bookstore_address);
            env.storage().instance().set(&DataKey::VerifiedBookstores, &verified_bookstores);
        }
    }

    // Issue #89: Zero-Knowledge GPA Verification Proof
    pub fn submit_gpa_proof(env: Env, student: Address, proof_hash: Bytes, public_inputs: Vec<u64>, verification_level: u64) {
        student.require_auth();
        
        let current_time = env.ledger().timestamp();
        
        let gpa_proof = ZKGPAProof {
            student: student.clone(),
            proof_hash: proof_hash.clone(),
            public_inputs: public_inputs.clone(),
            verification_level,
            verified_at: current_time,
        };
        
        env.storage().instance().set(&DataKey::ZKGPAProof(student.clone()), &gpa_proof);
    }
    
    pub fn verify_gpa_proof(env: Env, student: Address) -> bool {
        let proof: ZKGPAProof = env.storage().instance().get(&DataKey::ZKGPAProof(student.clone()))
            .unwrap_or_else(|| env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction)));
        
        // In a real implementation, this would perform actual ZK-proof verification
        // For now, we'll simulate verification by checking the verification_level
        // and ensuring the proof was submitted recently (within 30 days)
        let current_time = env.ledger().timestamp();
        let thirty_days_in_seconds = 30 * 24 * 60 * 60;
        
        (current_time - proof.verified_at) < thirty_days_in_seconds && proof.verification_level >= 35 // 3.5 GPA threshold
    }
    
    pub fn drip_with_gpa_verification(env: Env, donor: Address, student: Address, amount: i128, token: Address) {
        donor.require_auth();
        
        // Verify GPA proof first
        if !Self::verify_gpa_proof(env.clone(), student.clone()) {
            env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
        }
        
        // If GPA proof is valid, proceed with drip
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&donor, &student, &amount);
    }

    // Issue #90: Soulbound Scholarship Credential Minter
    pub fn mint_soulbound_credential(env: Env, student: Address, total_hours_funded: u64, major: Bytes, donor_organization: Address, metadata_url: Bytes) {
        student.require_auth();
        
        let current_time = env.ledger().timestamp();
        
        let credential_counter: u64 = env.storage().instance().get(&DataKey::CredentialCounter).unwrap_or(0);
        let credential_id = credential_counter + 1;
        
        let credential = SoulboundCredential {
            credential_id,
            student: student.clone(),
            total_hours_funded,
            major: major.clone(),
            donor_organization: donor_organization.clone(),
            graduation_date: current_time,
            metadata_url: metadata_url.clone(),
        };
        
        env.storage().instance().set(&DataKey::SoulboundCredential(credential_id), &credential);
        env.storage().instance().set(&DataKey::CredentialCounter, &credential_id);
    }
    
    pub fn get_credential(env: Env, credential_id: u64) -> SoulboundCredential {
        env.storage().instance().get(&DataKey::SoulboundCredential(credential_id))
            .unwrap_or_else(|| env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction)))
    }
    
    pub fn verify_credential_ownership(env: Env, credential_id: u64, claimed_student: Address) -> bool {
        let credential: SoulboundCredential = env.storage().instance().get(&DataKey::SoulboundCredential(credential_id))
            .unwrap_or_else(|| env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction)));
        
        credential.student == claimed_student
    }
    
    // Soulbound tokens cannot be transferred - this function will always fail
    pub fn transfer_credential(env: Env, _credential_id: u64, _from: Address, _to: Address) {
        env.panic_with_error((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction));
    }

    // Issue #91: Inter-Protocol Reputation Sync for Internships
    pub fn update_learning_velocity_score(env: Env, student: Address, courses_completed: u64, avg_completion_time: u64) {
        // This would typically be called by an authorized oracle or admin
        let current_time = env.ledger().timestamp();
        
        // Calculate learning velocity score (simplified formula)
        let score = if avg_completion_time > 0 {
            (courses_completed * 1000) / avg_completion_time
        } else {
            0
        };
        
        let velocity_score = LearningVelocityScore {
            student: student.clone(),
            score,
            courses_completed,
            avg_completion_time,
            last_updated: current_time,
        };
        
        env.storage().instance().set(&DataKey::LearningVelocityScore(student.clone()), &velocity_score);
    }
    
    pub fn get_learning_velocity_score(env: Env, student: Address) -> LearningVelocityScore {
        env.storage().instance().get(&DataKey::LearningVelocityScore(student.clone()))
            .unwrap_or(LearningVelocityScore {
                student: student.clone(),
                score: 0,
                courses_completed: 0,
                avg_completion_time: 0,
                last_updated: 0,
            })
    }
    
    pub fn set_grant_stream_contract(env: Env, admin: Address, grant_stream_address: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::GrantStreamContract, &grant_stream_address);
    }
    
    pub fn verify_reputation_for_grant(env: Env, student: Address, min_score: u64) -> bool {
        let velocity_score = Self::get_learning_velocity_score(env.clone(), student.clone());
        
        // Check if student meets the minimum score requirement
        velocity_score.score >= min_score
    }
    
    pub fn cross_contract_reputation_query(env: Env, student: Address, _requesting_contract: Address) -> LearningVelocityScore {
        // In a real implementation, this would verify the requesting contract is authorized
        // For now, we'll allow any contract to query reputation scores
        
        Self::get_learning_velocity_score(env, student)
    }
}

mod test;
