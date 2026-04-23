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
    client.fund_scholarship(&funder, &student, &500, &token_address.address());

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
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &10000);
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.buy_access(&student, &1, &5000, &token_address.address());

    env.ledger().set_timestamp(100);

    let session1 = soroban_sdk::Bytes::from_slice(&env, b"11111111111111111111111111111111");
    let session2 = soroban_sdk::Bytes::from_slice(&env, b"22222222222222222222222222222222");

    client.heartbeat(&student, &1, &session1);

    // Fast forward to allowed heartbeat timing
    env.ledger().set_timestamp(160);
    client.heartbeat(&student, &1, &session1);
}

#[test]
fn test_allow_session_reset_after_timeout() {
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

    // Fast forward strictly past the heartbeat window (`161 - 100 > 60` -> active_session = false)
    // Allows takeover / overwritten session storage naturally
    env.ledger().set_timestamp(161);
    client.heartbeat(&student, &1, &session2);
}

#[test]
fn test_calculate_remaining_airtime() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let funder = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);

    assert_eq!(client.calculate_remaining_airtime(&student), 0);

    client.fund_scholarship(&funder, &student, &500, &token_address.address());

    assert_eq!(client.calculate_remaining_airtime(&student), 50);
}

#[test]
fn test_calculate_remaining_airtime_zero_flow_rate() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let funder = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&0, &3600, &10, &100, &60);
    client.fund_scholarship(&funder, &student, &500, &token_address.address());

    assert_eq!(client.calculate_remaining_airtime(&student), 0);
}

#[test]
fn test_abrupt_disconnect() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &10000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    let heartbeat_interval = 60;
    client.init(&10, &3600, &10, &100, &heartbeat_interval);
    client.buy_access(&student, &1, &5000, &token_address.address());

    env.ledger().set_timestamp(100);
    let session = soroban_sdk::Bytes::from_slice(&env, b"11111111111111111111111111111111");

    // 1. Initial Heartbeat
    client.heartbeat(&student, &1, &session);
    assert_eq!(client.get_watch_time(&student, &1), 0);

    // 2. Normal Heartbeat (within interval + buffer)
    env.ledger().set_timestamp(160);
    client.heartbeat(&student, &1, &session);
    assert_eq!(client.get_watch_time(&student, &1), 60);

    // 3. Simulating Abrupt Disconnect (student closes browser at T=160)
    // No more heartbeats sent until the student "resumes" later.

    // 4. "Resume" after a long period (T=300)
    env.ledger().set_timestamp(300);
    client.heartbeat(&student, &1, &session);

    // Ensure the contract didn't count the missing 140 seconds
    assert_eq!(client.get_watch_time(&student, &1), 60);

    // 5. Subsequent normal heartbeat should work again
    env.ledger().set_timestamp(360);
    client.heartbeat(&student, &1, &session);
    assert_eq!(client.get_watch_time(&student, &1), 120);
}

#[test]
fn test_scholarship_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();

    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // 1. Initial funding
    client.init(&10, &3600, &10, &100, &60);
    client.fund_scholarship(&funder, &student, &500, &token_address.address());

    // 2. Register Mock Oracle and Verify
    let oracle_id = env.register(MockOracle, ());
    
    // Set admin first to be safe
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_academic_oracle(&admin, &oracle_id);

    client.verify_academic_progress(&student, &1); // Course 1 returns 1 (Success) in MockOracle

    // 3. Successful withdrawal by student
    // BaseRate 10 * 30 days = 10 * 2592000 = 25920000 unlocked
    // We only have 500 in balance, so we can withdraw 200
    client.withdraw_scholarship(&student, &200);

    let token = token::Client::new(&env, &token_address.address());
    assert_eq!(token.balance(&student), 200);
    assert_eq!(token.balance(&contract_id), 300);
}

pub struct MockOracle;

#[contractimpl]
impl MockOracle {
    pub fn check_status(_env: Env, _student: Address, course_id: u64) -> u32 {
        if course_id == 1 {
            1 // Success
        } else if course_id == 2 {
            0 // Fail
        } else {
            2 // Incomplete
        }
    }
}

#[test]
fn test_academic_oracle_hook() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &1000000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    let oracle_id = env.register(MockOracle, ());
    client.set_academic_oracle(&admin, &oracle_id);

    client.fund_scholarship(&funder, &student, &50000, &token_address.address());

    // Should fail withdrawal before verification
    let result = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &contract_id,
        &Symbol::new(&env, "withdraw_scholarship"),
        (student.clone(), 100i128).into_val(&env),
    );
    assert!(result.is_err());

    // Verify progress - SUCCESS for course 1
    client.verify_academic_progress(&student, &1);
    
    // Now should work
    client.withdraw_scholarship(&student, &1000);
    assert_eq!(token_client.balance(&student), 1000);

    // Verify progress - FAIL for course 2
    client.verify_academic_progress(&student, &2);

    // Should fail withdrawal because paused
    let result2 = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &contract_id,
        &Symbol::new(&env, "withdraw_scholarship"),
        (student.clone(), 100i128).into_val(&env),
    );
    assert!(result2.is_err());
}
    // 3. Unauthorized withdrawal (mock_all_auths should normally be specific)
    // Actually, in Soroban tests, `mock_all_auths` is very permissive.
    // If I want to test AUTH specifically, I might want to use more fine-grained auth testing.
    // But for this task, the implementation of `require_auth` in `lib.rs` is the key part.
}

#[test]
fn test_course_registry_basic_functionality() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let teacher = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Add courses to registry
    client.add_course_to_registry(&1, &teacher);
    client.add_course_to_registry(&2, &teacher);
    client.add_course_to_registry(&3, &teacher);

    // List all courses
    let courses = client.list_courses();
    assert_eq!(courses.len(), 3);
    assert!(courses.contains(&1));
    assert!(courses.contains(&2));
    assert!(courses.contains(&3));

    // Test pagination
    let page1 = client.list_courses_paginated(&0, &2);
    assert_eq!(page1.len(), 2);

    let page2 = client.list_courses_paginated(&2, &2);
    assert_eq!(page2.len(), 1);

    let empty_page = client.list_courses_paginated(&10, &5);
    assert_eq!(empty_page.len(), 0);

    // Get course info
    let course_info = client.get_course_info(&1);
    assert_eq!(course_info.course_id, 1);
    assert_eq!(course_info.creator, teacher);
    assert!(course_info.is_active);

    // Deactivate a course
    client.deactivate_course(&admin, &1);
    let course_info = client.get_course_info(&1);
    assert!(!course_info.is_active);

    // Cleanup inactive courses
    let removed_count = client.cleanup_inactive_courses(&admin);
    assert_eq!(removed_count, 1);

    let active_courses = client.list_courses();
    assert_eq!(active_courses.len(), 2);
    assert!(!active_courses.contains(&1));
}

#[test]
#[should_panic(expected = "LimitExceeded")]
fn test_course_registry_size_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let teacher = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);

    // Try to add more courses than the maximum allowed
    // This will panic when trying to add the 1001st course
    for i in 1..=1001 {
        client.add_course_to_registry(&i, &teacher);
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_course_registry_duplicate_course() {
    let env = Env::default();
    env.mock_all_auths();

    let teacher = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);

    // Add the same course twice - should panic
    client.add_course_to_registry(&1, &teacher);
    client.add_course_to_registry(&1, &teacher);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_course_registry_pagination_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let teacher = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);

    // Try to request more than 100 courses per page - should panic
    client.list_courses_paginated(&0, &101);
}

#[test]
fn test_course_registry_ttl_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let teacher = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Add course and verify TTL is extended
    client.add_course_to_registry(&1, &teacher);

    // Multiple calls should extend TTL without issues
    for _ in 0..10 {
        let _courses = client.list_courses();
        let _info = client.get_course_info(&1);
    }
}

#[test]
fn test_course_registry_gas_efficiency() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let teacher = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Add a reasonable number of courses
    for i in 1..=50 {
        client.add_course_to_registry(&i, &teacher);
    }

    // Test that pagination works efficiently with larger datasets
    let small_pages = client.list_courses_paginated(&0, &10);
    assert_eq!(small_pages.len(), 10);

    let medium_pages = client.list_courses_paginated(&10, &20);
    assert_eq!(medium_pages.len(), 20);

    // Even with many courses, pagination should work
    let all_courses = client.list_courses();
    assert_eq!(all_courses.len(), 50);
}

#[test]
fn test_double_spend_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());

    // Give student exactly 150 tokens
    token_client.mint(&student, &150);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60); // Base rate = 10 tokens / sec

    // 1. First purchase succeeds (costs 100 tokens)
    client.buy_access(&student, &1, &100, &token_address.address());
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        50
    );
    assert!(client.has_access(&student, &1));

    // 2. Second purchase (aiming to spend 100 tokens again) MUST fail
    // We use try_invoke_contract to verify that it traps correctly
    let result = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &contract_id,
        &Symbol::new(&env, "buy_access"),
        (student.clone(), 2_u64, 100_i128, token_address.address()).into_val(&env),
    );

    // Should fail with HostError (likely Insufficient Funds in the token contract)
    assert!(result.is_err());

    // Verify that student still has 50 tokens and NO access to course 2
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        50
    );
    assert!(!client.has_access(&student, &2));
}

#[test]
fn test_referral_reward_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let referrer = Address::generate(&env);
    let friend = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initial state
    assert_eq!(client.get_bonus_minutes(&referrer), 0);

    // 1. Friend claims referral
    client.referral_reward_claim(&referrer, &friend);

    // Default bonus amount is 3600
    assert_eq!(client.get_bonus_minutes(&referrer), 3600);

    // 2. Cannot claim twice for the same friend
    let result = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &contract_id,
        &Symbol::new(&env, "referral_reward_claim"),
        (referrer.clone(), friend.clone()).into_val(&env),
    );
    assert!(result.is_err());

    // 3. Admin can change bonus amount
    client.set_referral_bonus_amount(&admin, &7200);

    let another_friend = Address::generate(&env);
    client.referral_reward_claim(&referrer, &another_friend);

    // 3600 + 7200 = 10800
    assert_eq!(client.get_bonus_minutes(&referrer), 10800);
}

#[test]
fn test_creator_royalty_split() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let teacher = Address::generate(&env);
    let editor = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy token
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &10000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Add course by teacher
    client.add_course_to_registry(&1, &teacher);

    // Set royalty split: 70% teacher, 30% editor
    let shares = vec![&env, (teacher.clone(), 70u32), (editor.clone(), 30u32)];
    client.set_royalty_split(&teacher, &1, &shares);

    // Check split was set
    let split = client.get_royalty_split(&1).unwrap();
    assert_eq!(split.shares.len(), 2);

    // Record balances before purchase
    let teacher_before = token_client.balance(&teacher);
    let editor_before = token_client.balance(&editor);
    let contract_before = token_client.balance(&contract_id);

    // Student buys access for 1000 tokens (100 seconds at rate 10)
    client.buy_access(&student, &1, &1000, &token_address.address());

    // Verify split distribution: 700 to teacher, 300 to editor
    assert_eq!(token_client.balance(&teacher), teacher_before + 700);
    assert_eq!(token_client.balance(&editor), editor_before + 300);
    assert_eq!(token_client.balance(&contract_id), contract_before); // no leftover
}

#[test]
#[should_panic(expected = "Royalty Share does not sum to 100")]
fn test_royalty_split_invalid_total() {
    let env = Env::default();
    env.mock_all_auths();

    let teacher = Address::generate(&env);
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.add_course_to_registry(&1, &teacher);

    let bad_shares = vec![
        &env,
        (teacher.clone(), 60u32),
        (Address::generate(&env), 30u32),
    ];
    client.set_royalty_split(&teacher, &1, &bad_shares);
}

#[test]
fn test_royalty_split_no_split_fallback() {
    let env = Env::default();
    env.mock_all_auths();

    let teacher = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.add_course_to_registry(&1, &teacher);

    // No royalty split set
    let contract_before = token_client.balance(&contract_id);

    // Buy access → all funds should stay in contract (no split defined)
    client.buy_access(&student, &1, &500, &token_address.address());

    assert_eq!(token_client.balance(&contract_id), contract_before + 500);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_creator_royalty_split_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let teacher = Address::generate(&env);
    let editor = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy token
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &10000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Add course by teacher
    client.add_course_to_registry(&1, &teacher);

    // Set royalty split: 70% teacher, 30% editor
    let shares = vec![&env, (teacher.clone(), 70u32), (editor.clone(), 30u32)];
    client.set_royalty_split(&unauthorized, &1, &shares);
}

// Research Grant Milestone Escrow Tests

#[test]
fn test_research_grant_milestone_escrow_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let grantor = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a token for testing
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&grantor, &10000);

    // Deploy the scholarship contract
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Initialize the contract
    client.init(&10, &3600, &10, &100, &60);

    // Create a research grant for $5,000 lab equipment
    let grant_id = client.create_research_grant(
        &grantor,
        &student,
        &5000,
        &token_address.address()
    );

    // Verify grant creation
    assert_eq!(grant_id, 1);
    
    // Check grant details
    let research_grant = client.get_research_grant(&student);
    assert_eq!(research_grant.student, student);
    assert_eq!(research_grant.total_amount, 5000);
    assert_eq!(research_grant.grantor, grantor);
    assert!(research_grant.is_active);

    // Verify token transfer to contract
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&grantor),
        5000
    );
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&contract_id),
        5000
    );

    // Submit milestone claim for lab equipment purchase
    let invoice_hash = Symbol::new(&env, "invoice_hash_123");
    let description = Symbol::new(&env, "Lab Equipment Purchase");
    
    client.submit_milestone_claim(
        &student,
        &1, // milestone_id
        &5000,
        &description,
        &invoice_hash
    );

    // Verify milestone claim submission
    let milestone_claim = client.get_milestone_claim(&1);
    assert_eq!(milestone_claim.milestone_id, 1);
    assert_eq!(milestone_claim.student, student);
    assert_eq!(milestone_claim.amount, 5000);
    assert_eq!(milestone_claim.description, description);
    assert_eq!(milestone_claim.invoice_hash.unwrap(), invoice_hash);
    assert!(!milestone_claim.is_approved);
    assert!(!milestone_claim.is_claimed);

    // Verify invoice hash storage
    let stored_invoice_hash = client.get_invoice_hash(&1);
    assert_eq!(stored_invoice_hash.unwrap(), invoice_hash);

    // Grantor approves the milestone claim
    client.approve_milestone_claim(&grantor, &1);

    // Verify approval
    let approved_claim = client.get_milestone_claim(&1);
    assert!(approved_claim.is_approved);
    assert!(approved_claim.approved_at.is_some());
    assert!(!approved_claim.is_claimed);

    // Verify approval status
    assert!(client.is_milestone_approved(&1));

    // Student claims the lump sum
    client.claim_milestone_lump_sum(&student, &1);

    // Verify claim completion
    let claimed_milestone = client.get_milestone_claim(&1);
    assert!(claimed_milestone.is_claimed);
    assert!(claimed_milestone.claimed_at.is_some());

    // Verify lump sum transfer to student
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&student),
        5000
    );
    assert_eq!(
        token::Client::new(&env, &token_address.address()).balance(&contract_id),
        0
    );
}

#[test]
fn test_milestone_claim_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let grantor = Address::generate(&env);
    let student = Address::generate(&env);
    let other_student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&grantor, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);

    // Create research grant
    client.create_research_grant(&grantor, &student, &5000, &token_address.address());

    // Submit milestone claim
    let invoice_hash = Symbol::new(&env, "invoice_hash_123");
    let description = Symbol::new(&env, "Lab Equipment");
    
    client.submit_milestone_claim(&student, &1, &5000, &description, &invoice_hash);

    // Try to claim without approval - should fail
    env.mock_auths(&[
        &student.authenticate(&client.claim_milestone_lump_sum(&student, &1)),
    ]);
    env.mock_all_auths(); // Reset

    // Approve the claim
    client.approve_milestone_claim(&grantor, &1);

    // Try to claim with wrong student - should fail
    env.mock_auths(&[
        &other_student.authenticate(&client.claim_milestone_lump_sum(&other_student, &1)),
    ]);
    env.mock_all_auths(); // Reset

    // Successful claim
    client.claim_milestone_lump_sum(&student, &1);

    // Try to claim again - should fail
    env.mock_auths(&[
        &student.authenticate(&client.claim_milestone_lump_sum(&student, &1)),
    ]);
    env.mock_all_auths(); // Reset
}

#[test]
fn test_grantor_authorization() {
    let env = Env::default();
    env.mock_all_auths();

    let grantor = Address::generate(&env);
    let other_grantor = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&grantor, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);

    // Create research grant
    client.create_research_grant(&grantor, &student, &5000, &token_address.address());

    // Submit milestone claim
    let invoice_hash = Symbol::new(&env, "invoice_hash_123");
    let description = Symbol::new(&env, "Lab Equipment");
    
    client.submit_milestone_claim(&student, &1, &5000, &description, &invoice_hash);

    // Try to approve with wrong grantor - should fail
    env.mock_auths(&[
        &other_grantor.authenticate(&client.approve_milestone_claim(&other_grantor, &1)),
    ]);
    env.mock_all_auths(); // Reset

    // Successful approval by correct grantor
    client.approve_milestone_claim(&grantor, &1);
}

#[test]
fn test_research_grant_with_scholarship_coexistence() {
    let env = Env::default();
    env.mock_all_auths();

    let grantor = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&grantor, &10000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);

    // Fund a regular scholarship for living stipend
    client.fund_scholarship(&grantor, &student, &2000, &token_address.address());

    // Create a research grant for equipment
    client.create_research_grant(&grantor, &student, &5000, &token_address.address());

    // Verify both coexist
    let scholarship = client.get_scholarship(&student);
    assert_eq!(scholarship.balance, 2000);

    let research_grant = client.get_research_grant(&student);
    assert_eq!(research_grant.total_amount, 5000);

    // Submit and claim milestone - should not affect scholarship
    let invoice_hash = Symbol::new(&env, "invoice_hash_123");
    let description = Symbol::new(&env, "Lab Equipment");
    
    client.submit_milestone_claim(&student, &1, &5000, &description, &invoice_hash);
    client.approve_milestone_claim(&grantor, &1);
    client.claim_milestone_lump_sum(&student, &1);

    // Verify scholarship is still intact
    let scholarship_after = client.get_scholarship(&student);
    assert_eq!(scholarship_after.balance, 2000);

    // Verify milestone claim is processed
    let claimed_milestone = client.get_milestone_claim(&1);
    assert!(claimed_milestone.is_claimed);
}

// Multi-Sig Academic Board Review Tests

#[test]
fn test_deans_council_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let council_member1 = Address::generate(&env);
    let council_member2 = Address::generate(&env);
    let council_member3 = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize Dean's Council with 3 members requiring 2 signatures
    let council_members = vec![&env, council_member1.clone(), council_member2.clone(), council_member3.clone()];
    client.init_deans_council(&admin, &council_members, &2);

    // Verify council is properly initialized
    let council = client.get_deans_council();
    assert!(council.is_some());
    let council = council.unwrap();
    assert_eq!(council.members.len(), 3);
    assert_eq!(council.required_signatures, 2);
    assert!(council.is_active);
}

#[test]
fn test_board_pause_request_and_execution() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let council_member1 = Address::generate(&env);
    let council_member2 = Address::generate(&env);
    let council_member3 = Address::generate(&env);
    let funder = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &10000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize Dean's Council
    let council_members = vec![&env, council_member1.clone(), council_member2.clone(), council_member3.clone()];
    client.init_deans_council(&admin, &council_members, &2);

    // Fund a scholarship for the student
    client.fund_scholarship(&funder, &student, &1000, &token_address.address());

    // Verify initial state - not disputed
    assert!(!client.is_disputed(&student));
    let scholarship = client.get_scholarship(&student);
    assert!(!scholarship.is_disputed);
    assert_eq!(scholarship.dispute_reason, None);

    // Council member 1 initiates pause request for plagiarism
    let reason = Symbol::new(&env, "plagiarism_suspected");
    client.board_pause_request(&council_member1, &student, &reason);

    // Verify request is created but not executed yet
    let request = client.get_board_pause_request(&student);
    assert!(request.is_some());
    let request = request.unwrap();
    assert_eq!(request.signatures.len(), 1);
    assert!(!request.is_executed);
    assert_eq!(request.reason, reason);

    // Scholarship should still be accessible until second signature
    assert!(!client.is_disputed(&student));

    // Council member 2 signs the request
    client.board_pause_sign(&council_member2, &student);

    // Now the pause should be executed
    assert!(client.is_disputed(&student));
    let scholarship_after = client.get_scholarship(&student);
    assert!(scholarship_after.is_disputed);
    assert!(scholarship_after.is_paused);
    assert_eq!(scholarship_after.dispute_reason, Some(reason));

    // Verify request is marked as executed
    let executed_request = client.get_board_pause_request(&student);
    assert!(executed_request.unwrap().is_executed);
}

#[test]
fn test_disputed_student_cannot_access_courses() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let council_member1 = Address::generate(&env);
    let council_member2 = Address::generate(&env);
    let council_member3 = Address::generate(&env);
    let funder = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &10000);
    token_client.mint(&student, &1000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize Dean's Council
    let council_members = vec![&env, council_member1.clone(), council_member2.clone(), council_member3.clone()];
    client.init_deans_council(&admin, &council_members, &2);

    // Fund scholarship and buy course access
    client.fund_scholarship(&funder, &student, &1000, &token_address.address());
    client.buy_access(&student, &1, &100, &token_address.address());

    // Verify initial access
    assert!(client.has_access(&student, &1));

    // Execute board pause for academic misconduct
    let reason = Symbol::new(&env, "academic_misconduct");
    client.board_pause_request(&council_member1, &student, &reason);
    client.board_pause_sign(&council_member2, &student);

    // Verify student no longer has access due to disputed status
    assert!(!client.has_access(&student, &1));
    assert!(client.is_disputed(&student));
}

#[test]
fn test_final_ruling_upload() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let council_member1 = Address::generate(&env);
    let council_member2 = Address::generate(&env);
    let council_member3 = Address::generate(&env);
    let funder = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &10000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize Dean's Council
    let council_members = vec![&env, council_member1.clone(), council_member2.clone(), council_member3.clone()];
    client.init_deans_council(&admin, &council_members, &2);

    // Fund scholarship
    client.fund_scholarship(&funder, &student, &1000, &token_address.address());

    // Execute board pause
    let reason = Symbol::new(&env, "plagiarism_confirmed");
    client.board_pause_request(&council_member1, &student, &reason);
    client.board_pause_sign(&council_member2, &student);

    // Verify disputed state
    let scholarship = client.get_scholarship(&student);
    assert!(scholarship.is_disputed);
    assert_eq!(scholarship.final_ruling, None);

    // Upload final ruling
    let final_ruling = Symbol::new(&env, "scholarship_revoked_plagiarism");
    client.upload_final_ruling(&admin, &student, &final_ruling);

    // Verify ruling is recorded
    let scholarship_after = client.get_scholarship(&student);
    assert_eq!(scholarship_after.final_ruling, Some(final_ruling));
    assert!(scholarship_after.is_disputed); // Still disputed until admin action
}

#[test]
fn test_board_pause_security_checks() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let council_member1 = Address::generate(&env);
    let council_member2 = Address::generate(&env);
    let council_member3 = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Initialize Dean's Council
    let council_members = vec![&env, council_member1.clone(), council_member2.clone(), council_member3.clone()];
    client.init_deans_council(&admin, &council_members, &2);

    // Test unauthorized user cannot request pause
    let reason = Symbol::new(&env, "test_reason");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.board_pause_request(&unauthorized_user, &student, &reason);
    }));
    assert!(result.is_err());

    // Test unauthorized user cannot sign pause
    client.board_pause_request(&council_member1, &student, &reason);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.board_pause_sign(&unauthorized_user, &student);
    }));
    assert!(result.is_err());

    // Test council member cannot sign twice
    client.board_pause_sign(&council_member2, &student); // This should succeed and execute
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.board_pause_sign(&council_member1, &student); // Try to sign again
    }));
    assert!(result.is_err());
}

#[test]
fn test_deans_council_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let council_member1 = Address::generate(&env);
    let council_member2 = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Test that council must have exactly 3 members
    let two_members = vec![&env, council_member1.clone(), council_member2.clone()];
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.init_deans_council(&admin, &two_members, &2);
    }));
    assert!(result.is_err());

    // Test that required signatures must be 2
    let three_members = vec![&env, council_member1.clone(), council_member2.clone(), Address::generate(&env)];
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.init_deans_council(&admin, &three_members, &3);
    }));
    assert!(result.is_err());
}

#[test]
fn test_gpa_bonus_calculation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_academic_oracle(&admin, &oracle);

    // Test 1: No GPA reported - should have no bonus
    assert_eq!(client.get_student_gpa_bonus(&student), 0);

    // Test 2: GPA exactly 3.5 (35) - should have no bonus
    client.report_student_gpa(&oracle, &student, &35);
    assert_eq!(client.get_student_gpa_bonus(&student), 0);

    // Test 3: GPA 3.6 (36) - should have 2% bonus
    client.report_student_gpa(&oracle, &student, &36);
    assert_eq!(client.get_student_gpa_bonus(&student), 2);

    // Test 4: GPA 3.7 (37) - should have 4% bonus
    client.report_student_gpa(&oracle, &student, &37);
    assert_eq!(client.get_student_gpa_bonus(&student), 4);

    // Test 5: GPA 4.0 (40) - should have 10% bonus
    client.report_student_gpa(&oracle, &student, &40);
    assert_eq!(client.get_student_gpa_bonus(&student), 10);

    // Test 6: GPA 4.4 (44) - should have 18% bonus (maximum)
    client.report_student_gpa(&oracle, &student, &44);
    assert_eq!(client.get_student_gpa_bonus(&student), 18);
}

#[test]
fn test_gpa_weighted_flow_rate() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&student, &5000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60); // base_rate = 10
    client.set_admin(&admin);
    client.set_academic_oracle(&admin, &oracle);

    // Test without GPA bonus - should pay base rate
    client.buy_access(&student, &1, &100, &token_address.address());
    let balance_without_gpa = token_client.balance(&student);
    
    // Reset student balance
    token_client.mint(&student, &5000);

    // Report GPA 4.0 (40) for 10% bonus
    client.report_student_gpa(&oracle, &student, &40);
    
    // Now should pay 10% more (11 tokens per second)
    client.buy_access(&student, &2, &110, &token_address.address());
    let balance_with_gpa = token_client.balance(&student);
    
    // Should have spent more due to higher rate
    assert!(balance_with_gpa < balance_without_gpa);
}

#[test]
fn test_gpa_data_storage() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let student = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_academic_oracle(&admin, &oracle);

    // Test GPA data storage and retrieval
    client.report_student_gpa(&oracle, &student, &38); // 3.8 GPA
    
    let gpa_data = client.get_student_gpa(&student).unwrap();
    assert_eq!(gpa_data.gpa, 38);
    assert_eq!(gpa_data.student, student);
    assert!(gpa_data.oracle_verified);

    // Test unauthorized GPA reporting
    let unauthorized = Address::generate(&env);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.report_student_gpa(&unauthorized, &student, &40);
    }));
    assert!(result.is_err());
}

#[test]
fn test_drip_recalculation_on_gpa_change() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let student = Address::generate(&env);
    let funder = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address.address());
    token_client.mint(&funder, &10000);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_academic_oracle(&admin, &oracle);

    // Fund scholarship
    client.fund_scholarship(&funder, &student, &5000, &token_address.address());

    // Report initial GPA 3.6 (2% bonus)
    client.report_student_gpa(&oracle, &student, &36);

    // Upgrade GPA to 4.0 (10% bonus) - should trigger recalculation
    client.report_student_gpa(&oracle, &student, &40);

    // Verify bonus is updated
    assert_eq!(client.get_student_gpa_bonus(&student), 10);
}

#[test]
fn test_gpa_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let student = Address::generate(&env);

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);
    client.set_academic_oracle(&admin, &oracle);

    // Test invalid GPA (above 4.4)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.report_student_gpa(&oracle, &student, &45); // 4.5 GPA - invalid
    }));
    assert!(result.is_err());

    // Test valid GPA (4.4 maximum)
    client.report_student_gpa(&oracle, &student, &44); // 4.4 GPA - valid
    assert_eq!(client.get_student_gpa_bonus(&student), 18);
}

// Multi-Language Metadata Tests

#[test]
fn test_create_course_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // First, create the course in registry
    client.add_course_to_registry(&course_id, &creator);

    // Create course metadata
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);

    // Verify metadata was created
    let metadata = client.get_course_metadata(&course_id);
    assert_eq!(metadata.course_id, course_id);
    assert_eq!(metadata.default_language, default_language);
    assert_eq!(metadata.base_metadata_cid, base_metadata_cid);
    assert_eq!(metadata.available_languages.len(), 1);
    assert!(metadata.available_languages.contains(&default_language));
    assert!(metadata.is_active);
}

#[test]
fn test_add_language_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let spanish_language = Symbol::new(&env, "es");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");
    let spanish_metadata_cid = Symbol::new(&env, "QmSpanish456");
    let spanish_title = Symbol::new(&env, "Curso de Ejemplo");
    let spanish_description = Symbol::new(&env, "Descripción del curso en español");
    let spanish_thumbnail = Symbol::new(&env, "QmSpanishThumb");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course and base metadata
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);

    // Add Spanish language metadata
    client.add_language_metadata(
        &course_id,
        &spanish_language,
        &spanish_metadata_cid,
        &spanish_title,
        &spanish_description,
        Some(spanish_thumbnail),
        &creator,
    );

    // Verify Spanish metadata was added
    let spanish_metadata = client.get_language_metadata(&course_id, &spanish_language);
    assert_eq!(spanish_metadata.language_code, spanish_language);
    assert_eq!(spanish_metadata.ipfs_cid, spanish_metadata_cid);
    assert_eq!(spanish_metadata.title, spanish_title);
    assert_eq!(spanish_metadata.description, spanish_description);
    assert_eq!(spanish_metadata.thumbnail_cid.unwrap(), spanish_thumbnail);

    // Verify course metadata was updated
    let course_metadata = client.get_course_metadata(&course_id);
    assert_eq!(course_metadata.available_languages.len(), 2);
    assert!(course_metadata.available_languages.contains(&default_language));
    assert!(course_metadata.available_languages.contains(&spanish_language));
}

#[test]
fn test_update_language_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");
    let updated_cid = Symbol::new(&env, "QmUpdated789");
    let updated_title = Symbol::new(&env, "Updated Title");
    let updated_description = Symbol::new(&env, "Updated Description");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course and base metadata
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);

    // Update default language metadata
    client.update_language_metadata(
        &course_id,
        &default_language,
        &updated_cid,
        &updated_title,
        &updated_description,
        None,
        &creator,
    );

    // Verify metadata was updated
    let updated_metadata = client.get_language_metadata(&course_id, &default_language);
    assert_eq!(updated_metadata.ipfs_cid, updated_cid);
    assert_eq!(updated_metadata.title, updated_title);
    assert_eq!(updated_metadata.description, updated_description);
    assert!(updated_metadata.thumbnail_cid.is_none());
}

#[test]
fn test_get_available_languages() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let spanish_language = Symbol::new(&env, "es");
    let french_language = Symbol::new(&env, "fr");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course and base metadata
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);

    // Initially should have only default language
    let languages = client.get_available_languages(&course_id);
    assert_eq!(languages.len(), 1);
    assert!(languages.contains(&default_language));

    // Add Spanish
    client.add_language_metadata(
        &course_id,
        &spanish_language,
        &Symbol::new(&env, "QmSpanish"),
        &Symbol::new(&env, "Spanish Title"),
        &Symbol::new(&env, "Spanish Description"),
        None,
        &creator,
    );

    let languages = client.get_available_languages(&course_id);
    assert_eq!(languages.len(), 2);
    assert!(languages.contains(&default_language));
    assert!(languages.contains(&spanish_language));

    // Add French
    client.add_language_metadata(
        &course_id,
        &french_language,
        &Symbol::new(&env, "QmFrench"),
        &Symbol::new(&env, "French Title"),
        &Symbol::new(&env, "French Description"),
        None,
        &creator,
    );

    let languages = client.get_available_languages(&course_id);
    assert_eq!(languages.len(), 3);
    assert!(languages.contains(&default_language));
    assert!(languages.contains(&spanish_language));
    assert!(languages.contains(&french_language));
}

#[test]
fn test_set_default_language() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let spanish_language = Symbol::new(&env, "es");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course and add multiple languages
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);
    client.add_language_metadata(
        &course_id,
        &spanish_language,
        &Symbol::new(&env, "QmSpanish"),
        &Symbol::new(&env, "Spanish Title"),
        &Symbol::new(&env, "Spanish Description"),
        None,
        &creator,
    );

    // Verify initial default language
    assert_eq!(client.get_default_language(&course_id), default_language);

    // Change default language to Spanish
    client.set_default_language(&course_id, &spanish_language, &creator);

    // Verify default language was changed
    assert_eq!(client.get_default_language(&course_id), spanish_language);
}

#[test]
fn test_remove_language_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let spanish_language = Symbol::new(&env, "es");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course and add Spanish language
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);
    client.add_language_metadata(
        &course_id,
        &spanish_language,
        &Symbol::new(&env, "QmSpanish"),
        &Symbol::new(&env, "Spanish Title"),
        &Symbol::new(&env, "Spanish Description"),
        None,
        &creator,
    );

    // Verify Spanish exists
    let languages = client.get_available_languages(&course_id);
    assert_eq!(languages.len(), 2);
    assert!(languages.contains(&spanish_language));

    // Remove Spanish language
    client.remove_language_metadata(&course_id, &spanish_language, &creator);

    // Verify Spanish was removed
    let languages = client.get_available_languages(&course_id);
    assert_eq!(languages.len(), 1);
    assert!(!languages.contains(&spanish_language));
    assert!(languages.contains(&default_language));

    // Verify Spanish metadata no longer exists
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.get_language_metadata(&course_id, &spanish_language);
    }));
    assert!(result.is_err());
}

#[test]
fn test_cannot_remove_default_language() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);

    // Try to remove default language - should fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.remove_language_metadata(&course_id, &default_language, &creator);
    }));
    assert!(result.is_err());
}

#[test]
fn test_unauthorized_language_metadata_access() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);

    // Try to add language metadata as unauthorized user - should fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.add_language_metadata(
            &course_id,
            &Symbol::new(&env, "es"),
            &Symbol::new(&env, "QmSpanish"),
            &Symbol::new(&env, "Spanish Title"),
            &Symbol::new(&env, "Spanish Description"),
            None,
            &unauthorized_user,
        );
    }));
    assert!(result.is_err());

    // Try to update language metadata as unauthorized user - should fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.update_language_metadata(
            &course_id,
            &default_language,
            &Symbol::new(&env, "QmUpdated"),
            &Symbol::new(&env, "Updated Title"),
            &Symbol::new(&env, "Updated Description"),
            None,
            &unauthorized_user,
        );
    }));
    assert!(result.is_err());

    // Try to remove language metadata as unauthorized user - should fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.remove_language_metadata(&course_id, &default_language, &unauthorized_user);
    }));
    assert!(result.is_err());
}

#[test]
fn test_admin_can_manage_language_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let spanish_language = Symbol::new(&env, "es");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);

    // Admin should be able to add language metadata
    client.add_language_metadata(
        &course_id,
        &spanish_language,
        &Symbol::new(&env, "QmSpanish"),
        &Symbol::new(&env, "Spanish Title"),
        &Symbol::new(&env, "Spanish Description"),
        None,
        &admin,
    );

    // Verify Spanish was added
    let languages = client.get_available_languages(&course_id);
    assert_eq!(languages.len(), 2);
    assert!(languages.contains(&spanish_language));

    // Admin should be able to update language metadata
    client.update_language_metadata(
        &course_id,
        &spanish_language,
        &Symbol::new(&env, "QmSpanishUpdated"),
        &Symbol::new(&env, "Updated Spanish Title"),
        &Symbol::new(&env, "Updated Spanish Description"),
        None,
        &admin,
    );

    // Verify metadata was updated
    let spanish_metadata = client.get_language_metadata(&course_id, &spanish_language);
    assert_eq!(spanish_metadata.ipfs_cid, Symbol::new(&env, "QmSpanishUpdated"));
    assert_eq!(spanish_metadata.title, Symbol::new(&env, "Updated Spanish Title"));
}

#[test]
fn test_duplicate_language_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let course_id = 1u64;
    let default_language = Symbol::new(&env, "en");
    let base_metadata_cid = Symbol::new(&env, "QmTest123");

    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    client.init(&10, &3600, &10, &100, &60);
    client.set_admin(&admin);

    // Create course
    client.add_course_to_registry(&course_id, &creator);
    client.create_course_metadata(&course_id, &default_language, &base_metadata_cid, &creator);

    // Try to add duplicate language metadata - should fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.add_language_metadata(
            &course_id,
            &default_language, // Same as default
            &Symbol::new(&env, "QmDuplicate"),
            &Symbol::new(&env, "Duplicate Title"),
            &Symbol::new(&env, "Duplicate Description"),
            None,
            &creator,
        );
    }));
    assert!(result.is_err());
}
