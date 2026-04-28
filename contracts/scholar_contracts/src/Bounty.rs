#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec, String, Map, BytesN};
use crate::ScholarError;

// Student Profile NFT Contract for Soroban
// Implements dynamic NFTs that evolve with student achievements


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


#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudentProfileNFT {
    pub token_id: BytesN<32>,
    pub owner: Address,
    pub student_id: String,
    pub level: u32,
    pub xp: u64,
    pub achievements: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: Map<Symbol, String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub xp_reward: u64,
    pub unlocked_at: u64,
    pub rarity: String,
}

#[contracttype]
pub enum DataKey {
    NFT(BytesN<32>),
    StudentProfile(String),
    Achievement(String),
    NextTokenId,
    LevelThreshold(u32),
    NFTCounter,
}

// Level thresholds for progression
const LEVEL_THRESHOLDS: [(u32, u64); 8] = [
    (1, 0),      // Beginner
    (2, 100),    // Novice
    (3, 250),    // Apprentice
    (4, 500),    // Scholar
    (5, 1000),   // Expert
    (6, 2000),   // Master
    (7, 5000),   // Grandmaster
    (8, 10000),  // Legend
];

#[contract]
pub struct StudentProfileNFTContract;

#[contractimpl]
impl StudentProfileNFTContract {
    /// Initialize the NFT contract
    pub fn init(env: Env) {
        // Set next token ID to 1
        env.storage().instance().set(&DataKey::NextTokenId, &1u64);
        
        // Initialize level thresholds
        for (level, xp) in LEVEL_THRESHOLDS.iter() {
            env.storage().instance().set(&DataKey::LevelThreshold(*level), xp);
        }
        
        // Initialize NFT counter
        env.storage().instance().set(&DataKey::NFTCounter, &0u64);
    }

    /// Mint a new Student Profile NFT
    pub fn mint_nft(
        env: Env,
        owner: Address,
        student_id: String,
        initial_metadata: Map<Symbol, String>,
    ) -> BytesN<32> {
        owner.require_auth();

        // Generate unique token ID
        let next_id: u64 = env.storage().instance().get(&DataKey::NextTokenId).unwrap_or(1);
        let token_id = Self::generate_token_id(&env, next_id);
        
        // Update next token ID
        env.storage().instance().set(&DataKey::NextTokenId, &(next_id + 1));

        // Create student profile NFT
        let nft = StudentProfileNFT {
            token_id: token_id.clone(),
            owner: owner.clone(),
            student_id: student_id.clone(),
            level: 1,
            xp: 0,
            achievements: Vec::new(&env),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            metadata: initial_metadata,
        };

        // Store NFT data
        env.storage().persistent().set(&DataKey::NFT(token_id.clone()), &nft);
        
        // Store student profile reference
        env.storage().persistent().set(&DataKey::StudentProfile(student_id), &token_id);

        // Update NFT counter
        let mut counter: u64 = env.storage().instance().get(&DataKey::NFTCounter).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&DataKey::NFTCounter, &counter);

        // Emit mint event
        env.events().publish(
            (Symbol::new(&env, "NFT_Minted"), owner.clone(), token_id.clone()),
            (student_id, 1, 0)
        );

        token_id
    }

    /// Update student XP and level
    pub fn update_xp(env: Env, student_id: String, xp_amount: u64, caller: Address) {
        caller.require_auth();

        // Get token ID from student profile
        let token_id: BytesN<32> = env.storage().persistent()
            .get(&DataKey::StudentProfile(student_id.clone()))
            .expect("Student profile not found");

        // Get current NFT data
        let mut nft: StudentProfileNFT = env.storage().persistent()
            .get(&DataKey::NFT(token_id.clone()))
            .expect("NFT not found");

        // Verify caller is owner
        if nft.owner != caller {
            env.panic_with_error(ScholarError::OnlyOwnerCanUpdateXP);
        }

        let old_level = nft.level;
        nft.xp += xp_amount;
        nft.level = Self::calculate_level(&env, nft.xp);
        nft.updated_at = env.ledger().timestamp();

        // Store updated NFT
        env.storage().persistent().set(&DataKey::NFT(token_id.clone()), &nft);

        // Check for level up
        if nft.level > old_level {
            // Add level up achievement
            let achievement_title = format!("Level {}: {}", nft.level, Self::get_level_name(nft.level));
            nft.achievements.push_back(String::from_str(&env, &achievement_title));
            
            // Emit level up event
            env.events().publish(
                (Symbol::new(&env, "Level_Up"), caller, token_id.clone()),
                (old_level, nft.level, nft.xp)
            );
        }

        // Emit XP update event
        env.events().publish(
            (Symbol::new(&env, "XP_Updated"), caller, token_id),
            (xp_amount, nft.xp, nft.level)
        );
    }

    /// Add achievement to student profile
    pub fn add_achievement(
        env: Env,
        student_id: String,
        achievement: Achievement,
        caller: Address,
    ) {
        caller.require_auth();

        // Get token ID from student profile
        let token_id: BytesN<32> = env.storage().persistent()
            .get(&DataKey::StudentProfile(student_id.clone()))
            .expect("Student profile not found");

        // Get current NFT data
        let mut nft: StudentProfileNFT = env.storage().persistent()
            .get(&DataKey::NFT(token_id.clone()))
            .expect("NFT not found");

        // Verify caller is owner
        if nft.owner != caller {
            env.panic_with_error(ScholarError::OnlyOwnerCanAddAchievements);
        }

        // Store achievement
        env.storage().persistent().set(
            &DataKey::Achievement(achievement.id.clone()),
            &achievement
        );

        // Add to NFT achievements list
        nft.achievements.push_back(achievement.title.clone());
        nft.updated_at = env.ledger().timestamp();

        // Store updated NFT
        env.storage().persistent().set(&DataKey::NFT(token_id), &nft);

        // Award XP if achievement has reward
        if achievement.xp_reward > 0 {
            Self::update_xp(env, student_id, achievement.xp_reward, caller);
        }

        // Emit achievement event
        env.events().publish(
            (Symbol::new(&env, "Achievement_Added"), caller, student_id),
            (achievement.title, achievement.xp_reward, achievement.rarity)
        );
    }

    /// Transfer NFT to new owner
    pub fn transfer_nft(env: Env, token_id: BytesN<32>, from: Address, to: Address) {
        from.require_auth();

        let mut nft: StudentProfileNFT = env.storage().persistent()
            .get(&DataKey::NFT(token_id.clone()))
            .expect("NFT not found");

        // Verify from address is current owner
        if nft.owner != from {
            env.panic_with_error(ScholarError::TransferNotAuthorized);
        }

        // Update ownership
        nft.owner = to.clone();
        nft.updated_at = env.ledger().timestamp();

        // Store updated NFT
        env.storage().persistent().set(&DataKey::NFT(token_id), &nft);

        // Emit transfer event
        env.events().publish(
            (Symbol::new(&env, "NFT_Transferred"), from, to),
            token_id
        );
    }

    /// Get NFT data by token ID
    pub fn get_nft(env: Env, token_id: BytesN<32>) -> StudentProfileNFT {
        env.storage().persistent()
            .get(&DataKey::NFT(token_id))
            .expect("NFT not found")
    }

    /// Get NFT by student ID
    pub fn get_nft_by_student(env: Env, student_id: String) -> StudentProfileNFT {
        let token_id: BytesN<32> = env.storage().persistent()
            .get(&DataKey::StudentProfile(student_id))
            .expect("Student profile not found");

        Self::get_nft(env, token_id)
    }

    /// Get achievement by ID
    pub fn get_achievement(env: Env, achievement_id: String) -> Achievement {
        env.storage().persistent()
            .get(&DataKey::Achievement(achievement_id))
            .expect("Achievement not found")
    }

    /// Get total number of NFTs minted
    pub fn get_total_nfts(env: Env) -> u64 {
        env.storage().instance()
            .get(&DataKey::NFTCounter)
            .unwrap_or(0)
    }

    /// Get level threshold XP
    pub fn get_level_threshold(env: Env, level: u32) -> u64 {
        env.storage().instance()
            .get(&DataKey::LevelThreshold(level))
            .unwrap_or(0)
    }

    /// Calculate level based on XP
    fn calculate_level(env: &Env, xp: u64) -> u32 {
        for (level, threshold) in LEVEL_THRESHOLDS.iter().rev() {
            if xp >= *threshold {
                return *level;
            }
        }
        1
    }

    /// Get level name
    fn get_level_name(level: u32) -> &'static str {
        match level {
            1 => "Beginner",
            2 => "Novice",
            3 => "Apprentice",
            4 => "Scholar",
            5 => "Expert",
            6 => "Master",
            7 => "Grandmaster",
            8 => "Legend",
            _ => "Unknown",
        }
    }

    /// Generate unique token ID
    fn generate_token_id(env: &Env, id: u64) -> BytesN<32> {
        let mut bytes = [0u8; 32];
        let id_bytes = id.to_be_bytes();
        let timestamp = env.ledger().timestamp().to_be_bytes();
        
        // Combine ID and timestamp for uniqueness
        bytes[0..8].copy_from_slice(&id_bytes);
        bytes[8..16].copy_from_slice(&timestamp[0..8]);
        
        // Fill remaining bytes with pseudo-random data
        for i in 16..32 {
            bytes[i] = (id + i as u64).to_be_bytes()[7];
        }
        
        BytesN::from_array(env, &bytes)
    }

    /// Check if student exists
    pub fn student_exists(env: Env, student_id: String) -> bool {
        env.storage().persistent()
            .get::<DataKey, BytesN<32>>(&DataKey::StudentProfile(student_id))
            .is_some()
    }

    /// Get student's current level and XP
    pub fn get_student_level(env: Env, student_id: String) -> (u32, u64) {
        let nft = Self::get_nft_by_student(env, student_id);
        (nft.level, nft.xp)
    }

    /// Get student's achievements
    pub fn get_student_achievements(env: Env, student_id: String) -> Vec<String> {
        let nft = Self::get_nft_by_student(env, student_id);
        nft.achievements
    }

    /// Get progress to next level
    pub fn get_level_progress(env: Env, student_id: String) -> (u64, u64, f64) {
        let nft = Self::get_nft_by_student(env, student_id);
        
        if nft.level >= 8 {
            return (nft.xp, 0, 1.0); // Max level
        }

        let current_threshold = Self::get_level_threshold(env, nft.level);
        let next_threshold = Self::get_level_threshold(env, nft.level + 1);
        let progress = if next_threshold > current_threshold {
            (nft.xp - current_threshold) as f64 / (next_threshold - current_threshold) as f64
        } else {
            0.0
        };

        (nft.xp, next_threshold, progress)
    }
}

