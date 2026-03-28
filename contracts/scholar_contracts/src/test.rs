#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{token, Address, Env, Symbol, Vec, IntoVal, vec, Bytes};

#[test]
fn test_scholarship_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let _admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a token for testing
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &1000);

    // Deploy the scholarship contract
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Initialize the contract with new parameters
    client.init(&10, &3600, &10, &100, &60); // base_rate, threshold, discount%, min_deposit, heartbeat_interval

    // Student buys access to course 1 for 100 tokens (should be 10 seconds at base rate)
    client.buy_access(&student, &1, &100, &token_address.address());

    // Verify token balance
    assert_eq!(token::Client::new(&env, &token_address.address()).balance(&student), 900);
    assert_eq!(token::Client::new(&env, &token_address.address()).balance(&contract_id), 100);

    // Verify access
    env.ledger().set_timestamp(0);
    assert!(client.has_access(&student, &1));

    // Test heartbeat mechanism
    client.heartbeat(&student, &1, &soroban_sdk::Bytes::from_slice(&env, b"test_signature"));

    // Fast forward 5 seconds - should still have access
    env.ledger().set_timestamp(5);
    assert!(client.has_access(&student, &1));

    // Fast forward 11 seconds - should no longer have access
    env.ledger().set_timestamp(11);
    assert!(!client.has_access(&student, &1));
}

#[test]
fn test_subscription_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let subscriber = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&subscriber, &500);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Buy subscription for courses 1,2,3 for 1 month
    let course_ids = vec![&env, 1, 2, 3];
    client.buy_subscription(&subscriber, &course_ids, &1, &300, &token_address.address());

    // Should have access to subscribed courses without buying individual access
    assert!(client.has_access(&subscriber, &1));
    assert!(client.has_access(&subscriber, &2));
    assert!(client.has_access(&subscriber, &3));
    
    // Should not have access to non-subscribed course
    assert!(!client.has_access(&subscriber, &4));
}

#[test]
fn test_dynamic_pricing() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &100000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60); // 10% discount after 1 hour

    // Buy initial access and establish watch time
    client.buy_access(&student, &1, &72000, &token_address.address()); // 2 hours of access
    
    env.ledger().set_timestamp(0);
    client.heartbeat(&student, &1, &soroban_sdk::Bytes::from_slice(&env, b"test_signature"));
    
    // Simulate 1 hour of watch time (meets discount threshold)
    env.ledger().set_timestamp(3600);
    client.heartbeat(&student, &1, &soroban_sdk::Bytes::from_slice(&env, b"test_signature"));
    
    // Now buy more access - should get discounted rate (9 tokens per second instead of 10)
    let balance_before = token::Client::new(&env, &token_address.address()).balance(&student);
    client.buy_access(&student, &1, &100, &token_address.address()); // Should buy ~11.1 seconds at discounted rate
    let balance_after = token::Client::new(&env, &token_address.address()).balance(&student);
    
    assert_eq!(balance_before - balance_after, 100);
}

#[test]
fn test_minimum_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &50);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60); // 100 token minimum deposit

    // Should fail with amount below minimum
    let result = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &contract_id, 
        &Symbol::new(&env, "buy_access"),
        Vec::from_array(&env, [
            student.into_val(&env),
            1_u64.into_val(&env),
            50_i128.into_val(&env),
            token_address.address().into_val(&env)
        ])
    );
    assert!(result.is_err());
}

// Tests for Issue #88: Multi-Token Book Stipend Voucher
#[test]
fn test_book_stipend_voucher_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let donor = Address::generate(&env);
    let student = Address::generate(&env);
    let bookstore = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy book token
    let book_token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let book_token_client = token::StellarAssetClient::new(&env, &book_token_address.address());
    book_token_client.mint(&donor, &500);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Add verified bookstore
    client.add_verified_bookstore(&donor, &bookstore);

    // Create book stipend voucher
    client.create_book_stipend_voucher(&donor, &student, &200, &book_token_address.address(), &30);

    // Verify donor's book tokens were transferred
    assert_eq!(book_token_client.balance(&donor), 300);
    assert_eq!(book_token_client.balance(&contract_id), 200);

    // Student redeems voucher at verified bookstore
    client.redeem_book_stipend(&1, &bookstore);

    // Verify book tokens were transferred to bookstore
    assert_eq!(book_token_client.balance(&bookstore), 200);
    assert_eq!(book_token_client.balance(&contract_id), 0);
}

#[test]
fn test_book_stipend_voucher_unauthorized_bookstore() {
    let env = Env::default();
    env.mock_all_auths();

    let donor = Address::generate(&env);
    let student = Address::generate(&env);
    let unauthorized_bookstore = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let book_token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let book_token_client = token::StellarAssetClient::new(&env, &book_token_address.address());
    book_token_client.mint(&donor, &500);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Create voucher without adding verified bookstore
    client.create_book_stipend_voucher(&donor, &student, &200, &book_token_address.address(), &30);

    // Should fail when trying to redeem at unauthorized bookstore
    let result = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &contract_id,
        &Symbol::new(&env, "redeem_book_stipend"),
        Vec::from_array(&env, [
            1_u64.into_val(&env),
            unauthorized_bookstore.into_val(&env)
        ])
    );
    assert!(result.is_err());
}

// Tests for Issue #89: Zero-Knowledge GPA Verification Proof
#[test]
fn test_gpa_verification_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let donor = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&donor, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Student submits GPA proof (simulated)
    let proof_hash = Bytes::from_slice(&env, b"zk_proof_hash");
    let public_inputs = vec![&env, 35]; // 3.5 GPA
    client.submit_gpa_proof(&student, &proof_hash, &public_inputs, &35);

    // Verify GPA proof
    assert!(client.verify_gpa_proof(&student));

    // Donor can now drip with GPA verification
    client.drip_with_gpa_verification(&donor, &student, &100, &token_address.address());

    // Verify tokens were transferred
    assert_eq!(token_client.balance(&student), 100);
}

#[test]
fn test_gpa_verification_expired() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Student submits GPA proof
    let proof_hash = Bytes::from_slice(&env, b"zk_proof_hash");
    let public_inputs = vec![&env, 35];
    env.ledger().set_timestamp(0);
    client.submit_gpa_proof(&student, &proof_hash, &public_inputs, &35);

    // Fast forward 31 days (proof should be expired)
    env.ledger().set_timestamp(31 * 24 * 60 * 60);

    // Verification should fail
    assert!(!client.verify_gpa_proof(&student));
}

// Tests for Issue #90: Soulbound Scholarship Credential Minter
#[test]
fn test_soulbound_credential_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let donor_org = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Mint soulbound credential
    let major = Bytes::from_slice(&env, b"Computer Science");
    let metadata_url = Bytes::from_slice(&env, b"https://metadata.example.com/cred/1");
    client.mint_soulbound_credential(&student, &120, &major, &donor_org, &metadata_url);

    // Verify credential
    let credential = client.get_credential(&1);
    assert_eq!(credential.student, student);
    assert_eq!(credential.total_hours_funded, 120);
    assert_eq!(credential.major, major);
    assert_eq!(credential.donor_organization, donor_org);

    // Verify ownership
    assert!(client.verify_credential_ownership(&1, &student));
    assert!(!client.verify_credential_ownership(&1, &donor_org));
}

#[test]
fn test_soulbound_credential_transfer_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let other_user = Address::generate(&env);
    let donor_org = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Mint soulbound credential
    let major = Bytes::from_slice(&env, b"Computer Science");
    let metadata_url = Bytes::from_slice(&env, b"https://metadata.example.com/cred/1");
    client.mint_soulbound_credential(&student, &120, &major, &donor_org, &metadata_url);

    // Attempt to transfer should fail
    let result = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &contract_id,
        &Symbol::new(&env, "transfer_credential"),
        Vec::from_array(&env, [
            1_u64.into_val(&env),
            student.into_val(&env),
            other_user.into_val(&env)
        ])
    );
    assert!(result.is_err());
}

// Tests for Issue #91: Inter-Protocol Reputation Sync for Internships
#[test]
fn test_learning_velocity_score_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let admin = Address::generate(&env);
    let grant_stream_contract = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Set Grant Stream contract address
    client.set_grant_stream_contract(&admin, &grant_stream_contract);

    // Update learning velocity score
    client.update_learning_velocity_score(&student, &5, &100); // 5 courses, 100 avg completion time

    // Get learning velocity score
    let score = client.get_learning_velocity_score(&student);
    assert_eq!(score.student, student);
    assert_eq!(score.courses_completed, 5);
    assert_eq!(score.avg_completion_time, 100);
    assert_eq!(score.score, 50); // (5 * 1000) / 100 = 50

    // Verify reputation for grant
    assert!(client.verify_reputation_for_grant(&student, &40)); // Meets minimum
    assert!(!client.verify_reputation_for_grant(&student, &60)); // Doesn't meet higher minimum

    // Cross-contract reputation query
    let cross_contract_score = client.cross_contract_reputation_query(&student, &grant_stream_contract);
    assert_eq!(cross_contract_score.score, 50);
}

#[test]
fn test_cross_contract_reputation_query() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let requesting_contract = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);

    // Update learning velocity score first
    client.update_learning_velocity_score(&student, &10, &50); // Higher score

    // Cross-contract query should work
    let score = client.cross_contract_reputation_query(&student, &requesting_contract);
    assert_eq!(score.score, 200); // (10 * 1000) / 50 = 200
    assert_eq!(score.courses_completed, 10);
}
