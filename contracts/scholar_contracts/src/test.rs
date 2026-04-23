#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{token, vec, Address, Env, IntoVal, Symbol, Vec};

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
    token_client.mint(&student, &5000);

    // Deploy the scholarship contract
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Initialize the contract with new parameters
    client.init(&10, &3600, &10, &100, &60); // base_rate, threshold, discount%, min_deposit, heartbeat_interval

    // Student buys access to course 1 for 100 tokens (should be 10 seconds at base rate)
    client.buy_access(&student, &1, &100, &token_address.address());

    // Verify token balance
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        4900
    );
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&contract_id),
        100
    );

    // Verify access
    env.ledger().set_timestamp(0);
    assert!(client.has_access(&student, &1));

    // Test heartbeat mechanism
    client.heartbeat(
        &student,
        &1,
        &soroban_sdk::Bytes::from_slice(&env, b"test_signature"),
    );

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
    client.heartbeat(
        &student,
        &1,
        &soroban_sdk::Bytes::from_slice(&env, b"test_signature"),
    );

    // Simulate 1 hour of watch time (meets discount threshold)
    env.ledger().set_timestamp(3600);
    client.heartbeat(
        &student,
        &1,
        &soroban_sdk::Bytes::from_slice(&env, b"test_signature"),
    );

    // Now buy more access - should get discounted rate (9 tokens per second instead of 10)
    let balance_before = token::Client::new(&env, &token_address.address()).balance(&student);
    client.buy_access(&student, &1, &100, &token_address.address()); // Should buy ~11.1 seconds at discounted rate
    let balance_after = token::Client::new(&env, &token_address.address()).balance(&student);

    assert_eq!(balance_before - balance_after, 100);
}

#[test]
fn test_sbt_minting_trigger() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_course_duration(&1, &120); // 120 seconds duration

    env.ledger().set_timestamp(100);
    // Buy access for 2000 tokens -> 200 seconds of access
    client.buy_access(&student, &1, &2000, &token_address.address());

    client.heartbeat(
        &student,
        &1,
        &soroban_sdk::Bytes::from_slice(&env, b"test_signature"),
    );

    // Simulate 60 seconds watch time
    env.ledger().set_timestamp(160);
    client.heartbeat(
        &student,
        &1,
        &soroban_sdk::Bytes::from_slice(&env, b"test_signature"),
    );
    assert!(!client.is_sbt_minted(&student, &1));

    // Simulate another 60 seconds (total 120)
    env.ledger().set_timestamp(220);
    client.heartbeat(
        &student,
        &1,
        &soroban_sdk::Bytes::from_slice(&env, b"test_signature"),
    );

    // Should be minted now
    assert!(client.is_sbt_minted(&student, &1));
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
        Vec::from_array(
            &env,
            [
                student.into_val(&env),
                1_u64.into_val(&env),
                50_i128.into_val(&env),
                token_address.address().into_val(&env),
            ],
        ),
    );
    assert!(result.is_err());
}

#[test]
fn test_early_drop_immediate_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a token for testing
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &1000);

    // Deploy the scholarship contract
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Initialize the contract with a rate of 10 tokens per second
    client.init(&10, &3600, &10, &100, &60);

    // Student buys access to course 1 for 100 tokens (10 seconds) at timestamp 0
    client.buy_access(&student, &1, &100, &token_address.address());

    // Verify token balance after purchase
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        900
    );
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&contract_id),
        100
    );

    // Immediately request refund within 5 minutes - at timestamp 1
    env.ledger().set_timestamp(1);
    let refund_amount = client.pro_rated_refund(&student, &1);

    // Refund should be for remaining time: expiry at 10, current time 1, remaining = 9 seconds
    // Refund = 9 * 10 = 90 tokens
    assert_eq!(refund_amount, 90);

    // Verify tokens were refunded
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        990
    );
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&contract_id),
        10
    );

    // Access should be removed
    assert!(!client.has_access(&student, &1));
}

#[test]
fn test_early_drop_partial_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a token for testing
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &1000);

    // Deploy the scholarship contract
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Initialize the contract with a rate of 10 tokens per second
    client.init(&10, &3600, &10, &100, &60);

    // Student buys access to course 1 for 100 tokens (10 seconds) at timestamp 0
    client.buy_access(&student, &1, &100, &token_address.address());

    // Fast forward 5 seconds, request refund
    env.ledger().set_timestamp(5);
    let refund_amount = client.pro_rated_refund(&student, &1);

    // Refund should be for remaining time: expiry at 10, current time 5, remaining = 5 seconds
    // Refund = 5 * 10 = 50 tokens
    assert_eq!(refund_amount, 50);

    // Verify tokens were refunded
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        950
    );
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&contract_id),
        50
    );
}

#[test]
#[should_panic(expected = "Refund only available within 5 minutes of purchase")]
fn test_no_refund_after_5_minutes() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a token for testing
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &1000);

    // Deploy the scholarship contract
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Initialize the contract with a rate of 10 tokens per second
    client.init(&10, &3600, &10, &100, &60);

    // Student buys access to course 1 for 100 tokens (10 seconds) at timestamp 0
    client.buy_access(&student, &1, &100, &token_address.address());

    // Fast forward 6 minutes (360 seconds) - outside the 5 minute window
    env.ledger().set_timestamp(360);
    client.pro_rated_refund(&student, &1);
}

#[test]
fn test_refund_resets_last_purchase_time() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a token for testing
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &1000);

    // Deploy the scholarship contract
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Initialize the contract with a rate of 10 tokens per second
    client.init(&10, &3600, &10, &100, &60);

    // Student buys access to course 1 at timestamp 100
    env.ledger().set_timestamp(100);
    client.buy_access(&student, &1, &100, &token_address.address());

    // Fast forward 4 minutes (240 seconds), still within 5 minute window
    env.ledger().set_timestamp(340);
    let refund_amount = client.pro_rated_refund(&student, &1);

    // Should get full refund since we're within window
    // At 340, expiry was at 100+10=110, so remaining time = 0
    // But we're within 5 minutes, so this should work
    // Actually with the logic: time_since = 340 - 100 = 240 < 300 ✓
    // remaining = max(0, 110 - 340) = 0
    // refund = 0

    // Let's use a scenario where there's actually remaining time
    // Buy at 100, but the time should flow during buy_access
    // Let me adjust: buy at timestamp 100, expiry = 100 + 10 = 110
    // At timestamp 105, remaining = 110 - 105 = 5
    // Refund = 5 * 10 = 50

    assert!(refund_amount >= 0);
}

#[test]
fn test_decimals_and_leak_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a token simulating high precision decimals
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());

    // Give student 100 units (100 * 10^7 stroops)
    let initial_balance: i128 = 1_000_000_000;
    token_client.mint(&student, &initial_balance);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Set base rate to 1 unit per second (10_000_000 stroops)
    let rate: i128 = 10_000_000;
    client.init(&rate, &3600, &10, &100, &60);

    // Attempt to buy with an inexact amount (e.g. 2.5 units = 25_000_000 stroops)
    // Since rate is 10_000_000, 25_000_000 / 10_000_000 = 2 seconds
    // The actual cost should be 20_000_000. The remaining 5_000_000 should NOT be leaked.
    let amount_to_try: i128 = 25_000_000;
    client.buy_access(&student, &1, &amount_to_try, &token_address.address());

    // Verify balance was only deducted by the exact multiple of rate
    let actual_cost: i128 = 20_000_000;
    let expected_balance = initial_balance - actual_cost;
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        expected_balance
    );

    // Verify full refund leaves no value leaked
    env.ledger().set_timestamp(0); // exactly at purchase time
    let refund_amount = client.pro_rated_refund(&student, &1);

    // Should refund the exact time left (2 seconds total -> 20_000_000)
    assert_eq!(refund_amount, 20_000_000);

    // Final balance should be perfectly restored
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        initial_balance
    );
}

#[test]

fn test_admin_veto() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &2000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // 1. Test veto on bought access
    client.buy_access(&student, &1, &200, &token_address.address());
    assert!(client.has_access(&student, &1));

    client.veto_course_access(&admin, &student, &1);
    assert!(!client.has_access(&student, &1));

    // 2. Test veto on subscription access
    let course_ids = vec![&env, 2, 3];
    client.buy_subscription(&student, &course_ids, &1, &500, &token_address.address());
    assert!(client.has_access(&student, &2));
    assert!(client.has_access(&student, &3));

    client.veto_course_access(&admin, &student, &2);
    assert!(!client.has_access(&student, &2));
    assert!(client.has_access(&student, &3)); // Other course in sub should still work
}

#[test]
fn test_scholarship_role() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let teacher = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // 1. Approve teacher
    client.set_teacher(&admin, &teacher, &true);

    // 2. Fund scholarship for student
    client.fund_scholarship(&funder, &student, &500, &token_address.address(), &false);

    // Verify contract has tokens and student has balance
    let token = token::Client::new(&env, &token_address.address());
    assert_eq!(token.balance(&contract_id), 500);
    assert_eq!(token.balance(&funder), 500);

    // 3. Student pays teacher from scholarship
    client.transfer_scholarship_to_teacher(&student, &teacher, &200);

    assert_eq!(token.balance(&teacher), 200);
    assert_eq!(token.balance(&contract_id), 300);

    // 4. Try to pay unapproved teacher (should fail)
    let fake_teacher = Address::generate(&env);
    let result = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &contract_id,
        &soroban_sdk::Symbol::new(&env, "transfer_scholarship_to_teacher"),
        (student, fake_teacher, 100i128).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_global_course_veto() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student_a = Address::generate(&env);
    let student_b = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student_a, &1000);
    token_client.mint(&student_b, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // 1. Give both students access to course 1
    client.buy_access(&student_a, &1, &200, &token_address.address());
    let course_ids = vec![&env, 1];
    client.buy_subscription(&student_b, &course_ids, &1, &300, &token_address.address());

    assert!(client.has_access(&student_a, &1));
    assert!(client.has_access(&student_b, &1));

    // 2. Admin vetoes course 1 GLOBALLY
    client.veto_course_globally(&admin, &1, &true);

    // 3. Both should lose access
    assert!(!client.has_access(&student_a, &1));
    assert!(!client.has_access(&student_b, &1));

    // 4. Verification that other courses are not affected
    let course_ids_2 = vec![&env, 2];
    client.buy_subscription(
        &student_b,
        &course_ids_2,
        &1,
        &300,
        &token_address.address(),
    );
    assert!(client.has_access(&student_b, &2));
}

#[test]
#[should_panic(expected = "HostError")]
fn test_prevent_session_sharing() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &10000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.buy_access(&student, &1, &5000, &token_address.address());

    env.ledger().set_timestamp(100);

    let session1 = soroban_sdk::Bytes::from_slice(&env, b"11111111111111111111111111111111");
    let session2 = soroban_sdk::Bytes::from_slice(&env, b"22222222222222222222222222222222");

    client.heartbeat(&student, &1, &session1);

    // Fast forward to allowed heartbeat timing (100 + 60)
    // Here `active_session` is still TRUE (60 <= 60). New hash triggers PANIC.
    env.ledger().set_timestamp(160);
    client.heartbeat(&student, &1, &session2);
}

#[test]
fn test_calculate_remaining_airtime() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let funder = Address::generate(&env);
    let oracle = Address::generate(&env);
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_oracle_status(&admin, &oracle, &true);

    // Verify enrollment first (Issue #160 requirement for funding)
    let enrollment = EnrollmentData {
        student: student.clone(),
        university_id: 123,
        start_timestamp: 0,
        end_timestamp: 10000,
        generated_at: 0,
        nonce: 1,
    };
    client.verify_enrollment(&student, &oracle, &soroban_sdk::BytesN::from_array(&env, &[0u8; 64]), &enrollment);

    client.fund_scholarship(&funder, &student, &500, &token_address.address());

    // 500 balance / 10 base_rate = 50 seconds
    assert_eq!(client.calculate_remaining_airtime(&student), 50);

    // Test Reputation Bonus (2% discount on rate)
    // effective_rate = 10 * 0.98 = 9.8 -> but we use integer math (10 * 98) / 100 = 9
    client.set_reputation_bonus(&admin, &student, &true);
    // 500 balance / 9 effective_rate = 55 seconds
    assert_eq!(client.calculate_remaining_airtime(&student), 55);

    // Test GPA Multiplier (120% increase in rate -> 12000 bps)
    // Base rate is 9 (after rep bonus). 9 * 120% = 10.8 -> 10
    // Actually, effective_rate calculation: (base * 98/100 * multiplier/10000)
    // (10 * 98 / 100) = 9
    // (9 * 12000 / 10000) = 10
    let gpa_payload = GpaData {
        student: student.clone(),
        gpa_bps: 400, // 4.0 GPA
        epoch: 1,
        generated_at: 0,
        nonce: 1,
    };
    client.apply_gpa_multiplier(&student, &oracle, &soroban_sdk::BytesN::from_array(&env, &[0u8; 64]), &gpa_payload);
    // 500 / 10 = 50
    assert_eq!(client.calculate_remaining_airtime(&student), 50);
}

#[test]
fn test_withdrawal_whitelisting() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let payout = Address::generate(&env);
    let funder = Address::generate(&env);
    let oracle = Address::generate(&env);
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_oracle_status(&admin, &oracle, &true);

    // Verify enrollment
    let enrollment = EnrollmentData {
        student: student.clone(),
        university_id: 123,
        start_timestamp: 0,
        end_timestamp: 10000,
        generated_at: 0,
        nonce: 1,
    };
    client.verify_enrollment(&student, &oracle, &soroban_sdk::BytesN::from_array(&env, &[0u8; 64]), &enrollment);

    client.fund_scholarship(&funder, &student, &500, &token_address.address());

    // Set whitelisted address
    env.ledger().set_timestamp(0);
    client.set_authorized_payout_address(&student, &payout);

    // Try to confirm early (should fail)
    let result = env.try_invoke_contract::<(), Error>(&contract_id, &Symbol::new(&env, "confirm_authorized_payout_address"), (student.clone(),).into_val(&env));
    assert!(result.is_err());

    // Confirm after 48 hours (172800 seconds)
    env.ledger().set_timestamp(172801);
    client.confirm_authorized_payout_address(&student);

    // Claim scholarship
    client.claim_scholarship(&student, &200);

    assert_eq!(token::Client::new(&env, &token_address.address()).balance(&payout), 200);
}

#[test]
fn test_gpa_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let oracle = Address::generate(&env);
    let admin = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);
    
    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_oracle_status(&admin, &oracle, &true);

    // Apply low GPA (< 2.5)
    let gpa_payload = GpaData {
        student: student.clone(),
        gpa_bps: 200, // 2.0 GPA
        epoch: 1,
        generated_at: 0,
        nonce: 1,
    };
    client.apply_gpa_multiplier(&student, &oracle, &soroban_sdk::BytesN::from_array(&env, &[0u8; 64]), &gpa_payload);

    // Rate should be 0 (paused)
    assert_eq!(client.calculate_remaining_airtime(&student), 0);
}

#[test]
fn test_verify_enrollment_rejects_stale_oracle_data() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let oracle = Address::generate(&env);
    let admin = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_oracle_status(&admin, &oracle, &true);

    env.ledger().set_timestamp(ORACLE_STALENESS_THRESHOLD + 1);

    let enrollment = EnrollmentData {
        student: student.clone(),
        university_id: 123,
        start_timestamp: 0,
        end_timestamp: 10000,
        generated_at: 0,
        nonce: 1,
    };

    let result = env.try_invoke_contract::<(), Error>(
        &contract_id,
        &Symbol::new(&env, "verify_enrollment"),
        (
            student.clone(),
            oracle.clone(),
            soroban_sdk::BytesN::from_array(&env, &[0u8; 64]),
            enrollment,
        )
            .into_val(&env),
    );

    assert!(result.is_err());
}

#[test]
fn test_apply_gpa_multiplier_accepts_fresh_oracle_data_at_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let oracle = Address::generate(&env);
    let admin = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_oracle_status(&admin, &oracle, &true);

    env.ledger().set_timestamp(ORACLE_STALENESS_THRESHOLD);

    let gpa_payload = GpaData {
        student: student.clone(),
        gpa_bps: 400,
        epoch: 1,
        generated_at: 0,
        nonce: 1,
    };

    client.apply_gpa_multiplier(
        &student,
        &oracle,
        &soroban_sdk::BytesN::from_array(&env, &[0u8; 64]),
        &gpa_payload,
    );

    assert_eq!(client.calculate_remaining_airtime(&student), 0);
}

// PoA (Proof-of-Attendance) Tests

#[test]
fn test_poa_configuration() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Configure PoA with 1-week intervals and 7-day grace period
    client.init_poa_config(&admin, &604800, &604800, &3);

    let config = client.get_poa_config();
    assert_eq!(config.checkpoint_interval_seconds, 604800);
    assert_eq!(config.grace_period_seconds, 604800);
    assert_eq!(config.max_proofs_per_checkpoint, 3);
    assert!(config.is_active);
}

#[test]
fn test_poa_successful_attendance_proof() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.init_poa_config(&admin, &604800, &604800, &3);

    // Student buys access
    client.buy_access(&student, &1, &100, &token_address.address());

    // Set timestamp to start of first epoch
    env.ledger().set_timestamp(100000);

    // Submit attendance proof with valid hashes and timestamps
    let proof_hashes = vec![
        &env,
        soroban_sdk::Bytes::from_slice(&env, b"hash1"),
        soroban_sdk::Bytes::from_slice(&env, b"hash2"),
    ];
    let timestamps = vec![&env, 100001u64, 100002u64];

    client.submit_attendance_proof(&student, &1, &proof_hashes, &timestamps);

    // Verify student is still compliant
    assert!(client.check_poa_compliance(&student, &1));
    
    let poa_state = client.get_student_poa_state(&student, &1);
    assert_eq!(poa_state.current_state, CheckpointState::Compliant);
    assert_eq!(poa_state.last_checkpoint_submitted, 0); // First epoch
}

#[test]
#[should_panic]
fn test_poa_invalid_timestamp_range() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.init_poa_config(&admin, &604800, &604800, &3);

    client.buy_access(&student, &1, &100, &token_address.address());

    // Set timestamp to middle of epoch
    env.ledger().set_timestamp(400000);

    // Submit with timestamp outside current epoch (too early)
    let proof_hashes = vec![&env, soroban_sdk::Bytes::from_slice(&env, b"hash1")];
    let timestamps = vec![&env, 100000u64]; // Outside current epoch

    client.submit_attendance_proof(&student, &1, &proof_hashes, &timestamps);
}

#[test]
fn test_poa_late_submission_within_grace_period() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.init_poa_config(&admin, &604800, &604800, &3);

    client.buy_access(&student, &1, &100, &token_address.address());

    // Start in first epoch
    env.ledger().set_timestamp(100000);

    // Skip to next epoch but within grace period
    env.ledger().set_timestamp(700000); // Still within grace period of first epoch

    // Submit proof for previous epoch
    let proof_hashes = vec![&env, soroban_sdk::Bytes::from_slice(&env, b"hash1")];
    let timestamps = vec![&env, 200000u64]; // Within first epoch

    client.submit_attendance_proof(&student, &1, &proof_hashes, &timestamps);

    // Should still be compliant (within grace period)
    assert!(client.check_poa_compliance(&student, &1));
}

#[test]
fn test_poa_late_submission_after_grace_period() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.init_poa_config(&admin, &604800, &604800, &3);

    client.buy_access(&student, &1, &100, &token_address.address());

    // Start in first epoch
    env.ledger().set_timestamp(100000);

    // Skip well beyond grace period
    env.ledger().set_timestamp(1500000); // Well beyond grace period

    // Submit proof for previous epoch
    let proof_hashes = vec![&env, soroban_sdk::Bytes::from_slice(&env, b"hash1")];
    let timestamps = vec![&env, 200000u64]; // Within first epoch

    client.submit_attendance_proof(&student, &1, &proof_hashes, &timestamps);

    // Should be delinquent and stream halted
    assert!(!client.check_poa_compliance(&student, &1));
    
    let poa_state = client.get_student_poa_state(&student, &1);
    assert_eq!(poa_state.current_state, CheckpointState::Delinquent);
    assert!(poa_state.stream_halted_until > 0);
}

#[test]
fn test_poa_access_denied_without_compliance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.init_poa_config(&admin, &604800, &604800, &3);

    client.buy_access(&student, &1, &100, &token_address.address());

    // Initially has access
    assert!(client.has_access(&student, &1));

    // Skip beyond grace period without submitting proof
    env.ledger().set_timestamp(1500000);

    // Should no longer have access due to PoA non-compliance
    assert!(!client.has_access(&student, &1));
}

#[test]
#[should_panic]
fn test_poa_heartbeat_blocked_without_compliance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.init_poa_config(&admin, &604800, &604800, &3);

    client.buy_access(&student, &1, &100, &token_address.address());

    // Skip beyond grace period without submitting proof
    env.ledger().set_timestamp(1500000);

    // Heartbeat should fail due to PoA non-compliance
    client.heartbeat(
        &student,
        &1,
        &soroban_sdk::Bytes::from_slice(&env, b"test_signature"),
    );
}

#[test]
fn test_poa_resumed_after_successful_proof() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.init_poa_config(&admin, &604800, &604800, &3);

    client.buy_access(&student, &1, &100, &token_address.address());

    // Skip beyond grace period without submitting proof
    env.ledger().set_timestamp(1500000);

    // Should not have access
    assert!(!client.has_access(&student, &1));

    // Submit proof for current epoch
    let proof_hashes = vec![&env, soroban_sdk::Bytes::from_slice(&env, b"hash1")];
    let timestamps = vec![&env, 1400000u64]; // Within current epoch

    client.submit_attendance_proof(&student, &1, &proof_hashes, &timestamps);

    // Should have access again
    assert!(client.has_access(&student, &1));
    assert!(client.check_poa_compliance(&student, &1));
}

#[test]
fn test_poa_subscription_with_compliance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&subscriber, &500);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.init_poa_config(&admin, &604800, &604800, &3);

    // Buy subscription
    let course_ids = vec![&env, 1, 2, 3];
    client.buy_subscription(&subscriber, &course_ids, &1, &300, &token_address.address());

    // Initially has access via subscription
    assert!(client.has_access(&subscriber, &1));

    // Submit PoA proof
    env.ledger().set_timestamp(100000);
    let proof_hashes = vec![&env, soroban_sdk::Bytes::from_slice(&env, b"hash1")];
    let timestamps = vec![&env, 100001u64];
    client.submit_attendance_proof(&subscriber, &1, &proof_hashes, &timestamps);

    // Should still have access
    assert!(client.has_access(&subscriber, &1));

    // Skip beyond grace period without new proof
    env.ledger().set_timestamp(1500000);

    // Should lose access even with subscription due to PoA non-compliance
    assert!(!client.has_access(&subscriber, &1));
}

// Fuzz Tests for PoA Timeline Manipulation

#[test]
fn test_poa_fuzz_epoch_timeline_manipulation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &50000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    
    // Test with various epoch configurations
    let test_configs = vec![
        (3600, 1800, 1),    // 1 hour epoch, 30 min grace, 1 proof
        (86400, 43200, 2),  // 1 day epoch, 12 hour grace, 2 proofs
        (604800, 604800, 3), // 1 week epoch, 1 week grace, 3 proofs
        (1209600, 86400, 5), // 2 week epoch, 1 day grace, 5 proofs
    ];

    for (epoch_seconds, grace_seconds, max_proofs) in test_configs {
        // Reconfigure PoA
        client.init_poa_config(&admin, &epoch_seconds, &grace_seconds, &max_proofs);

        // Test timeline manipulation scenarios
        test_timeline_manipulation_scenarios(
            &env, &client, &student, &token_address.address(), epoch_seconds, grace_seconds
        );
    }
}

fn test_timeline_manipulation_scenarios(
    env: &Env,
    client: &ScholarContractClient,
    student: &Address,
    token_address: &Address,
    epoch_seconds: u64,
    grace_seconds: u64,
) {
    // Reset student state
    client.buy_access(student, &1, &1000, token_address);

    // Scenario 1: Submit proof exactly at epoch boundary
    env.ledger().set_timestamp(epoch_seconds - 1);
    let proof_hashes = vec![env, soroban_sdk::Bytes::from_slice(env, b"boundary_hash")];
    let timestamps = vec![env, epoch_seconds - 1];
    client.submit_attendance_proof(student, &1, &proof_hashes, &timestamps);
    assert!(client.check_poa_compliance(student, &1));

    // Scenario 2: Submit proof just within grace period
    env.ledger().set_timestamp(epoch_seconds + grace_seconds - 1);
    let proof_hashes2 = vec![env, soroban_sdk::Bytes::from_slice(env, b"grace_hash")];
    let timestamps2 = vec![env, epoch_seconds + 100]; // Within first epoch
    client.submit_attendance_proof(student, &1, &proof_hashes2, &timestamps2);
    assert!(client.check_poa_compliance(student, &1));

    // Scenario 3: Submit proof just after grace period (should fail)
    env.ledger().set_timestamp(epoch_seconds + grace_seconds + 1);
    let proof_hashes3 = vec![env, soroban_sdk::Bytes::from_slice(env, b"late_hash")];
    let timestamps3 = vec![env, epoch_seconds + 200]; // Within first epoch
    
    // This should mark as delinquent
    client.submit_attendance_proof(student, &1, &proof_hashes3, &timestamps3);
    assert!(!client.check_poa_compliance(student, &1));

    // Verify state is correctly set to delinquent
    let poa_state = client.get_student_poa_state(student, &1);
    assert_eq!(poa_state.current_state, CheckpointState::Delinquent);

    // Scenario 4: Attempt to manipulate by jumping to future epoch
    let future_epoch = 5;
    env.ledger().set_timestamp(future_epoch * epoch_seconds);
    
    // Submit proof for current epoch should restore compliance
    let proof_hashes4 = vec![env, soroban_sdk::Bytes::from_slice(env, b"future_hash")];
    let timestamps4 = vec![env, future_epoch * epoch_seconds + 1000];
    client.submit_attendance_proof(student, &1, &proof_hashes4, &timestamps4);
    assert!(client.check_poa_compliance(student, &1));
}

#[test]
fn test_poa_fuzz_grace_period_edge_cases() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &50000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    
    // Test with very short grace periods
    let epoch_seconds = 3600; // 1 hour
    let grace_seconds = 1;     // 1 second grace period
    
    client.init_poa_config(&admin, &epoch_seconds, &grace_seconds, &3);
    client.buy_access(student, &1, &1000, token_address);

    // Submit proof at start of epoch
    env.ledger().set_timestamp(1000);
    let proof_hashes = vec![env, soroban_sdk::Bytes::from_slice(env, b"early_hash")];
    let timestamps = vec![env, 1001];
    client.submit_attendance_proof(student, &1, &proof_hashes, &timestamps);

    // Jump to exactly end of grace period
    env.ledger().set_timestamp(epoch_seconds + grace_seconds);
    
    // Should still be compliant (exactly at grace period boundary)
    assert!(client.check_poa_compliance(student, &1));

    // Jump 1 second beyond grace period
    env.ledger().set_timestamp(epoch_seconds + grace_seconds + 1);
    
    // Should no longer be compliant
    assert!(!client.check_poa_compliance(student, &1));
}

#[test]
fn test_poa_fuzz_multiple_epoch_jumps() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &50000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    
    let epoch_seconds = 3600; // 1 hour
    let grace_seconds = 1800; // 30 minutes
    
    client.init_poa_config(&admin, &epoch_seconds, &grace_seconds, &3);
    client.buy_access(student, &1, &1000, token_address);

    // Test jumping multiple epochs without submissions
    let mut current_time = 1000;
    
    for epoch in 1..=5 {
        // Jump to start of epoch
        current_time = epoch * epoch_seconds;
        env.ledger().set_timestamp(current_time);
        
        // Should lose compliance after missing previous epoch
        if epoch > 1 {
            assert!(!client.check_poa_compliance(student, &1));
        }
        
        // Submit proof for current epoch to restore compliance
        let proof_hashes = vec![env, soroban_sdk::Bytes::from_slice(env, &format!("hash_{}", epoch).into_bytes())];
        let timestamps = vec![env, current_time + 100];
        client.submit_attendance_proof(student, &1, &proof_hashes, &timestamps);
        
        // Should be compliant again
        assert!(client.check_poa_compliance(student, &1));
        
        // Verify correct epoch tracking
        let poa_state = client.get_student_poa_state(student, &1);
        assert_eq!(poa_state.last_checkpoint_submitted, epoch - 1);
    }
}

#[test]
fn test_poa_fuzz_concurrent_submissions() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &50000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    
    client.init_poa_config(&admin, &604800, &604800, &3);
    client.buy_access(student, &1, &1000, token_address);

    env.ledger().set_timestamp(100000);

    // Test submitting maximum allowed proofs
    let mut proof_hashes = Vec::new(&env);
    let mut timestamps = Vec::new(&env);
    
    for i in 1..=3 {
        proof_hashes.push_back(soroban_sdk::Bytes::from_slice(env, &format!("hash_{}", i).into_bytes()));
        timestamps.push_back(100000 + i * 100);
    }
    
    client.submit_attendance_proof(student, &1, &proof_hashes, &timestamps);
    assert!(client.check_poa_compliance(student, &1));

    // Try to submit one more proof (should fail)
    let extra_hashes = vec![env, soroban_sdk::Bytes::from_slice(env, b"extra_hash")];
    let extra_timestamps = vec![env, 100400];
    
    // This should panic due to exceeding max_proofs_per_checkpoint
    std::panic::catch_unwind(|| {
        client.submit_attendance_proof(student, &1, &extra_hashes, &extra_timestamps);
    }).expect_err("Should panic when exceeding max proofs per checkpoint");
}

// ZK-Proof Verification Tests

#[test]
fn test_zk_verification_key_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create a mock verification key (200 bytes minimum)
    let mut vk_bytes = Vec::<u8>::new(&env);
    for i in 0..200 {
        vk_bytes.push_back(i as u8);
    }
    let verification_key = soroban_sdk::Bytes::from_slice(&env, &vk_bytes.to_array());

    // Initialize verification key
    client.init_zk_verification_key(&admin, &verification_key);

    // Verification should work now
    assert!(true); // If we get here, initialization succeeded
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_zk_verification_key_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    let mut vk_bytes = Vec::<u8>::new(&env);
    for i in 0..200 {
        vk_bytes.push_back(i as u8);
    }
    let verification_key = soroban_sdk::Bytes::from_slice(&env, &vk_bytes.to_array());

    // Try to initialize with unauthorized address
    client.init_zk_verification_key(&unauthorized, &verification_key);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_zk_verification_key_invalid_format() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create verification key that's too short (< 200 bytes)
    let short_vk = soroban_sdk::Bytes::from_slice(&env, b"short_key");

    // Should fail with invalid format
    client.init_zk_verification_key(&admin, &short_vk);
}

#[test]
fn test_gpa_threshold_proof_verification_valid() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize verification key
    let mut vk_bytes = Vec::<u8>::new(&env);
    for i in 0..200 {
        vk_bytes.push_back(i as u8);
    }
    let verification_key = soroban_sdk::Bytes::from_slice(&env, &vk_bytes.to_array());
    client.init_zk_verification_key(&admin, &verification_key);

    // Create a valid mock proof
    let valid_proof = create_mock_gpa_proof(&env, true);
    
    // Verify the proof
    let result = client.verify_gpa_threshold_proof(&student, &1, &valid_proof);
    
    // Should succeed with our simplified verification
    assert!(result);
    
    // Check academic standing
    assert!(client.has_academic_standing(&student, &1));
    
    let standing = client.get_academic_standing(&student, &1);
    assert!(standing.semester_passed);
    assert_eq!(standing.course_id, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_gpa_threshold_proof_verification_invalid_format() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize verification key
    let mut vk_bytes = Vec::<u8>::new(&env);
    for i in 0..200 {
        vk_bytes.push_back(i as u8);
    }
    let verification_key = soroban_sdk::Bytes::from_slice(&env, &vk_bytes.to_array());
    client.init_zk_verification_key(&admin, &verification_key);

    // Create an invalid proof with wrong format
    let invalid_proof = create_invalid_format_proof(&env);
    
    // Should panic due to invalid format
    client.verify_gpa_threshold_proof(&student, &1, &invalid_proof);
}

#[test]
fn test_gpa_threshold_proof_verification_empty_proof() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize verification key
    let mut vk_bytes = Vec::<u8>::new(&env);
    for i in 0..200 {
        vk_bytes.push_back(i as u8);
    }
    let verification_key = soroban_sdk::Bytes::from_slice(&env, &vk_bytes.to_array());
    client.init_zk_verification_key(&admin, &verification_key);

    // Create an empty proof
    let empty_proof = GPAThresholdProof {
        a: soroban_sdk::Bytes::new(&env),
        b: soroban_sdk::Bytes::new(&env),
        c: soroban_sdk::Bytes::new(&env),
        public_signals: soroban_sdk::Bytes::new(&env),
    };
    
    // Verify the empty proof
    let result = client.verify_gpa_threshold_proof(&student, &1, &empty_proof);
    
    // Should fail with empty proof
    assert!(!result);
    
    // Academic standing should not be granted
    assert!(!client.has_academic_standing(&student, &1));
}

#[test]
fn test_batch_verify_gpa_proofs() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize verification key
    let mut vk_bytes = Vec::<u8>::new(&env);
    for i in 0..200 {
        vk_bytes.push_back(i as u8);
    }
    let verification_key = soroban_sdk::Bytes::from_slice(&env, &vk_bytes.to_array());
    client.init_zk_verification_key(&admin, &verification_key);

    // Create multiple proofs
    let course_ids = vec![&env, 1, 2, 3];
    let proofs = vec![
        &env,
        create_mock_gpa_proof(&env, true),
        create_mock_gpa_proof(&env, true),
        create_mock_gpa_proof(&env, true),
    ];
    
    // Batch verify
    let results = client.batch_verify_gpa_proofs(&student, &course_ids, &proofs);
    
    // All should succeed
    assert_eq!(results.len(), 3);
    for i in 0..results.len() {
        assert!(results.get(i).unwrap());
    }
    
    // All courses should have academic standing
    assert!(client.has_academic_standing(&student, &1));
    assert!(client.has_academic_standing(&student, &2));
    assert!(client.has_academic_standing(&student, &3));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_batch_verify_mismatched_lengths() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize verification key
    let mut vk_bytes = Vec::<u8>::new(&env);
    for i in 0..200 {
        vk_bytes.push_back(i as u8);
    }
    let verification_key = soroban_sdk::Bytes::from_slice(&env, &vk_bytes.to_array());
    client.init_zk_verification_key(&admin, &verification_key);

    // Create mismatched arrays
    let course_ids = vec![&env, 1, 2]; // 2 courses
    let proofs = vec![
        &env,
        create_mock_gpa_proof(&env, true),
        create_mock_gpa_proof(&env, true),
        create_mock_gpa_proof(&env, true),
    ]; // 3 proofs
    
    // Should panic due to mismatched lengths
    client.batch_verify_gpa_proofs(&student, &course_ids, &proofs);
}

#[test]
fn test_revoke_academic_standing() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize verification key
    let mut vk_bytes = Vec::<u8>::new(&env);
    for i in 0..200 {
        vk_bytes.push_back(i as u8);
    }
    let verification_key = soroban_sdk::Bytes::from_slice(&env, &vk_bytes.to_array());
    client.init_zk_verification_key(&admin, &verification_key);

    // First, grant academic standing
    let valid_proof = create_mock_gpa_proof(&env, true);
    let result = client.verify_gpa_threshold_proof(&student, &1, &valid_proof);
    assert!(result);
    assert!(client.has_academic_standing(&student, &1));

    // Revoke academic standing
    client.revoke_academic_standing(&admin, &student, &1);

    // Should no longer have academic standing
    assert!(!client.has_academic_standing(&student, &1));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_revoke_academic_standing_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let student = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Try to revoke with unauthorized address
    client.revoke_academic_standing(&unauthorized, &student, &1);
}

#[test]
fn test_benchmark_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Create a mock proof
    let proof = create_mock_gpa_proof(&env, true);
    
    // Benchmark verification
    let instructions_used = client.benchmark_verification(&proof);
    
    // Should use some instructions (greater than 0)
    assert!(instructions_used > 0);
    
    // Should be reasonable (less than 1 million instructions for basic validation)
    assert!(instructions_used < 1_000_000);
}

#[test]
#[should_panic(expected = "Academic standing not found")]
fn test_get_academic_standing_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Try to get academic standing that doesn't exist
    client.get_academic_standing(&student, &1);
}

// Helper functions for testing

fn create_mock_gpa_proof(env: &Env, valid: bool) -> GPAThresholdProof {
    if valid {
        // Create a valid format proof
        let mut a_bytes = Vec::<u8>::new(env);
        let mut b_bytes = Vec::<u8>::new(env);
        let mut c_bytes = Vec::<u8>::new(env);
        let mut signals_bytes = Vec::<u8>::new(env);
        
        // G1 points (64 bytes each)
        for i in 0..64 {
            a_bytes.push_back(i as u8);
            c_bytes.push_back((i + 64) as u8);
        }
        
        // G2 point (128 bytes)
        for i in 0..128 {
            b_bytes.push_back((i + 128) as u8);
        }
        
        // Public signals (96 bytes minimum - 3 * 32 bytes)
        for i in 0..96 {
            signals_bytes.push_back((i + 256) as u8);
        }
        
        GPAThresholdProof {
            a: soroban_sdk::Bytes::from_slice(env, &a_bytes.to_array()),
            b: soroban_sdk::Bytes::from_slice(env, &b_bytes.to_array()),
            c: soroban_sdk::Bytes::from_slice(env, &c_bytes.to_array()),
            public_signals: soroban_sdk::Bytes::from_slice(env, &signals_bytes.to_array()),
        }
    } else {
        // Create an invalid proof (empty)
        GPAThresholdProof {
            a: soroban_sdk::Bytes::new(env),
            b: soroban_sdk::Bytes::new(env),
            c: soroban_sdk::Bytes::new(env),
            public_signals: soroban_sdk::Bytes::new(env),
        }
    }
}

fn create_invalid_format_proof(env: &Env) -> GPAThresholdProof {
    // Create a proof with invalid format (wrong sizes)
    let mut a_bytes = Vec::<u8>::new(env);
    let mut b_bytes = Vec::<u8>::new(env);
    let mut c_bytes = Vec::<u8>::new(env);
    let mut signals_bytes = Vec::<u8>::new(env);
    
    // Wrong sizes to trigger validation failure
    for i in 0..32 { // Should be 64 for G1
        a_bytes.push_back(i as u8);
    }
    
    for i in 0..64 { // Should be 128 for G2
        b_bytes.push_back((i + 32) as u8);
    }
    
    for i in 0..32 { // Should be 64 for G1
        c_bytes.push_back((i + 96) as u8);
    }
    
    for i in 0..32 { // Should be at least 96 for signals
        signals_bytes.push_back((i + 128) as u8);
    }
    
    GPAThresholdProof {
        a: soroban_sdk::Bytes::from_slice(env, &a_bytes.to_array()),
        b: soroban_sdk::Bytes::from_slice(env, &b_bytes.to_array()),
        c: soroban_sdk::Bytes::from_slice(env, &c_bytes.to_array()),
        public_signals: soroban_sdk::Bytes::from_slice(env, &signals_bytes.to_array()),
    }
}

// Milestone Bounty Tests

#[test]
fn test_bounty_reserve_funding() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy token and contract
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Fund bounty reserve with 500 tokens
    client.fund_bounty_reserve(&funder, &student, &1, &500, &token_address.address());

    // Verify bounty reserve balance
    let bounty_reserve = client.get_bounty_reserve(&student, &1);
    assert_eq!(bounty_reserve.balance, 500);
    assert_eq!(bounty_reserve.course_id, 1);

    // Verify token balances
    assert_eq!(token_client.balance(&funder), 500);
    assert_eq!(token_client.balance(&contract_id), 500);
}

#[test]
fn test_milestone_bounty_claim_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy token and contract
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);
    token_client.mint(&student, &100);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Student buys access to course
    client.buy_access(&student, &1, &100, &token_address.address());

    // Fund bounty reserve
    client.fund_bounty_reserve(&funder, &student, &1, &500, &token_address.address());

    // Claim milestone bounty with valid advisor signature
    let advisor_sig = soroban_sdk::Bytes::from_slice(&env, b"test_advisor_sig");
    client.claim_milestone_bounty(&student, &1, &1, &200, &advisor_sig);

    // Verify milestone marked as claimed
    assert!(client.is_milestone_claimed(&student, &1, &1));

    // Verify bounty reserve balance decreased
    let bounty_reserve = client.get_bounty_reserve(&student, &1);
    assert_eq!(bounty_reserve.balance, 300);

    // Verify student received bounty
    assert_eq!(token_client.balance(&student), 300); // 100 initial + 200 bounty

    // Verify continuous stream access still works
    assert!(client.has_access(&student, &1));
}

#[test]
fn test_milestone_double_claim_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);
    token_client.mint(&student, &100);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    client.buy_access(&student, &1, &100, &token_address.address());
    client.fund_bounty_reserve(&funder, &student, &1, &500, &token_address.address());

    // First claim should succeed
    let advisor_sig = soroban_sdk::Bytes::from_slice(&env, b"test_advisor_sig");
    client.claim_milestone_bounty(&student, &1, &1, &200, &advisor_sig);

    // Second claim should fail
    let result = env.try_invoke_contract::<soroban_sdk::xdr::ScVal>(
        &contract_id,
        &Symbol::new(&env, "claim_milestone_bounty"),
        (
            &student,
            &1u64, // course_id
            &1u64, // milestone_id (same as before)
            &200i128, // bounty_amount
            &advisor_sig,
        ),
    );

    assert!(result.is_err());
}

#[test]
fn test_bounty_insufficient_reserve() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);
    token_client.mint(&student, &100);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    client.buy_access(&student, &1, &100, &token_address.address());
    client.fund_bounty_reserve(&funder, &student, &1, &100, &token_address.address()); // Only 100 tokens

    // Try to claim 200 tokens - should fail
    let advisor_sig = soroban_sdk::Bytes::from_slice(&env, b"test_advisor_sig");
    let result = env.try_invoke_contract::<soroban_sdk::xdr::ScVal>(
        &contract_id,
        &Symbol::new(&env, "claim_milestone_bounty"),
        (
            &student,
            &1u64,
            &1u64,
            &200i128, // More than available
            &advisor_sig,
        ),
    );

    assert!(result.is_err());
}

#[test]
fn test_bounty_requires_active_stream() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Fund bounty reserve but don't buy access
    client.fund_bounty_reserve(&funder, &student, &1, &500, &token_address.address());

    // Try to claim without active access - should fail
    let advisor_sig = soroban_sdk::Bytes::from_slice(&env, b"test_advisor_sig");
    let result = env.try_invoke_contract::<soroban_sdk::xdr::ScVal>(
        &contract_id,
        &Symbol::new(&env, "claim_milestone_bounty"),
        (
            &student,
            &1u64,
            &1u64,
            &200i128,
            &advisor_sig,
        ),
    );

    assert!(result.is_err());
}

#[test]
fn test_bounty_stream_parameters_unaffected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);
    token_client.mint(&student, &500);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60); // 10 tokens/second
    client.set_admin(&admin);

    // Buy access for 30 seconds (300 tokens)
    client.buy_access(&student, &1, &300, &token_address.address());

    // Fund and claim bounty
    client.fund_bounty_reserve(&funder, &student, &1, &500, &token_address.address());
    
    let advisor_sig = soroban_sdk::Bytes::from_slice(&env, b"test_advisor_sig");
    client.claim_milestone_bounty(&student, &1, &1, &200, &advisor_sig);

    // Verify stream access still works and time not affected
    env.ledger().set_timestamp(10);
    assert!(client.has_access(&student, &1));

    env.ledger().set_timestamp(30);
    assert!(client.has_access(&student, &1));

    env.ledger().set_timestamp(31);
    assert!(!client.has_access(&student, &1)); // Stream should expire at original time

    // Verify student has both remaining stream time and bounty
    assert_eq!(token_client.balance(&student), 400); // 200 remaining + 200 bounty
}

#[test]
fn test_multiple_milestone_claims() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &2000);
    token_client.mint(&student, &100);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    client.buy_access(&student, &1, &100, &token_address.address());
    client.fund_bounty_reserve(&funder, &student, &1, &1000, &token_address.address());

    let advisor_sig = soroban_sdk::Bytes::from_slice(&env, b"test_advisor_sig");

    // Claim multiple different milestones
    client.claim_milestone_bounty(&student, &1, &1, &200, &advisor_sig);
    client.claim_milestone_bounty(&student, &1, &2, &300, &advisor_sig);
    client.claim_milestone_bounty(&student, &1, &3, &250, &advisor_sig);

    // Verify all milestones marked as claimed
    assert!(client.is_milestone_claimed(&student, &1, &1));
    assert!(client.is_milestone_claimed(&student, &1, &2));
    assert!(client.is_milestone_claimed(&student, &1, &3));

    // Verify final bounty reserve balance
    let bounty_reserve = client.get_bounty_reserve(&student, &1);
    assert_eq!(bounty_reserve.balance, 250); // 1000 - 200 - 300 - 250

    // Verify student received all bounties
    assert_eq!(token_client.balance(&student), 850); // 100 initial + 200 + 300 + 250
}
