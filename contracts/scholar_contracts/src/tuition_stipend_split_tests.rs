use soroban_sdk::{Address, Env, Symbol, token};

use crate::{ScholarContract, TuitionStipendSplit, DataKey, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_EXTEND};

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::contractimpl;
    
    #[test]
    fn test_tuition_stipend_split_configuration() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScholarContract);
        let client = ScholarContractClient::new(&env, &contract_id);
        
        // Setup admin
        let admin = Address::generate(&env);
        client.init(&10, &3600, &10, &100, &60);
        client.set_admin(&admin);
        
        // Create student and university addresses
        let student = Address::generate(&env);
        let university = Address::generate(&env);
        
        // Configure tuition-stipend split (70% university, 30% student)
        client.set_tuition_stipend_split(
            &admin,
            &student,
            &university,
            &70, // university_percentage
            &30  // student_percentage
        );
        
        // Verify the configuration
        let split_config = client.get_tuition_stipend_split(&student);
        assert!(split_config.is_some());
        
        let config = split_config.unwrap();
        assert_eq!(config.university_address, university);
        assert_eq!(config.student_address, student);
        assert_eq!(config.university_percentage, 70);
        assert_eq!(config.student_percentage, 30);
    }
    
    #[test]
    fn test_tuition_stipend_split_distribution() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScholarContract);
        let client = ScholarContractClient::new(&env, &contract_id);
        
        // Setup admin and token contract
        let admin = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token_contract_id = env.register_stellar_asset_contract(token_admin.clone());
        let token_client = token::StellarAssetClient::new(&env, &token_contract_id);
        
        client.init(&10, &3600, &10, &100, &60);
        client.set_admin(&admin);
        
        // Create addresses
        let student = Address::generate(&env);
        let university = Address::generate(&env);
        let funder = Address::generate(&env);
        
        // Mint tokens to funder
        token_client.mint(&funder, &1000);
        
        // Configure split
        client.set_tuition_stipend_split(&admin, &student, &university, &70, &30);
        
        // Fund scholarship (this should trigger the split)
        client.fund_scholarship(&funder, &student, &1000, &token_contract_id, &Symbol::new(&env, "default_roadmap"));
        
        // Check balances - university should have 700, student scholarship should have 300
        let university_balance = token_client.balance(&university);
        let student_scholarship = client.get_scholarship(&student);
        
        assert_eq!(university_balance, 700);
        assert_eq!(student_scholarship.balance, 300);
    }
    
    #[test]
    #[should_panic(expected = "Percentages must sum to 100")]
    fn test_invalid_split_percentages() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScholarContract);
        let client = ScholarContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let student = Address::generate(&env);
        let university = Address::generate(&env);
        
        client.init(&10, &3600, &10, &100, &60);
        client.set_admin(&admin);
        
        // Try to set invalid percentages (should panic)
        client.set_tuition_stipend_split(&admin, &student, &university, &80, &30); // 80 + 30 = 110
    }
    
    #[test]
    fn test_no_split_configuration() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScholarContract);
        let client = ScholarContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token_contract_id = env.register_stellar_asset_contract(token_admin.clone());
        let token_client = token::StellarAssetClient::new(&env, &token_contract_id);
        
        client.init(&10, &3600, &10, &100, &60);
        client.set_admin(&admin);
        
        let student = Address::generate(&env);
        let funder = Address::generate(&env);
        
        token_client.mint(&funder, &1000);
        
        // Fund scholarship without configuring split
        client.fund_scholarship(&funder, &student, &1000, &token_contract_id, &Symbol::new(&env, "default_roadmap"));
        
        // Student should receive full amount
        let student_scholarship = client.get_scholarship(&student);
        assert_eq!(student_scholarship.balance, 1000);
    }
}
