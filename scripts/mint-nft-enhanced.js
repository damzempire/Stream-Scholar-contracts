#!/usr/bin/env node

/**
 * Mint Student Profile NFT script
 * Mints new NFTs for students and manages their profiles
 */

const { Server, Networks, TransactionBuilder, Keypair, Contract } = require('@stellar/stellar-sdk');
const StudentProfileNFTIntegration = require('../src/StudentProfileNFTIntegration');
const fs = require('fs');
const path = require('path');

class NFTMinter {
    constructor(network = 'testnet') {
        this.network = network === 'testnet' ? Networks.TESTNET : Networks.STANDALONE;
        this.horizonUrl = network === 'testnet' 
            ? 'https://horizon-testnet.stellar.org' 
            : 'http://localhost:8000';
        this.server = new Server(this.horizonUrl);
        this.integration = new StudentProfileNFTIntegration(this.network, this.horizonUrl);
        
        // Load contract configuration
        this.loadContractConfig();
    }

    loadContractConfig() {
        try {
            const configPath = path.join(__dirname, '../config/deployment.json');
            if (fs.existsSync(configPath)) {
                const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
                this.contractId = config.contractId;
                console.log(`✅ Loaded contract configuration: ${this.contractId}`);
            } else {
                throw new Error('Deployment configuration not found. Please run deploy-nft.js first.');
            }
        } catch (error) {
            console.error('❌ Failed to load contract configuration:', error.message);
            throw error;
        }
    }

    /**
     * Mint a new Student Profile NFT
     */
    async mintStudentProfile(studentData, issuerKeypair) {
        try {
            console.log(`🎓 Creating Student Profile NFT for: ${studentData.studentId}`);
            
            const result = await this.integration.createStudentProfile(
                studentData.studentId,
                studentData.personalInfo,
                issuerKeypair
            );

            console.log('✅ Student Profile NFT created successfully!');
            console.log(`🔑 Token ID: ${result.nft.tokenId}`);
            console.log(`📊 Initial Level: ${result.profile.learning.level}`);
            console.log(`💰 Initial XP: ${result.profile.learning.totalXP}`);

            return result;
        } catch (error) {
            console.error('❌ Failed to mint Student Profile NFT:', error.message);
            throw error;
        }
    }

    /**
     * Add course completion to student profile
     */
    async completeCourse(studentId, courseData, signerKeypair) {
        try {
            console.log(`📚 Completing course: ${courseData.title} for student: ${studentId}`);
            
            const result = await this.integration.completeCourse(studentId, courseData, signerKeypair);
            
            console.log('✅ Course completed successfully!');
            console.log(`🎯 Course Progress: ${result.progress}%`);
            console.log(`✅ Completed: ${result.completed}`);
            
            return result;
        } catch (error) {
            console.error('❌ Failed to complete course:', error.message);
            throw error;
        }
    }

    /**
     * Add achievement to student profile
     */
    async addAchievement(studentId, achievementData, signerKeypair) {
        try {
            console.log(`🏆 Adding achievement: ${achievementData.title} for student: ${studentId}`);
            
            const result = await this.integration.addAchievement(studentId, achievementData, signerKeypair);
            
            if (result) {
                console.log('✅ Achievement added successfully!');
                console.log(`🎖️  Achievement: ${result.title}`);
                console.log(`💎 Rarity: ${result.rarity}`);
                console.log(`💰 XP Reward: ${result.xpReward}`);
            } else {
                console.log('ℹ️  Achievement already exists or was not added');
            }
            
            return result;
        } catch (error) {
            console.error('❌ Failed to add achievement:', error.message);
            throw error;
        }
    }

    /**
     * Get complete student profile
     */
    async getStudentProfile(studentId) {
        try {
            console.log(`📋 Retrieving profile for student: ${studentId}`);
            
            const profile = await this.integration.getCompleteProfile(studentId);
            
            console.log('✅ Profile retrieved successfully!');
            console.log(`📊 Level: ${profile.profile.learning.level} (${profile.stats.totalXP} XP)`);
            console.log(`🎓 Courses Completed: ${profile.stats.coursesCompleted}`);
            console.log(`🏆 Achievements: ${profile.stats.achievementsUnlocked}`);
            console.log(`🔥 Study Streak: ${profile.stats.studyStreak} days`);
            
            return profile;
        } catch (error) {
            console.error('❌ Failed to retrieve student profile:', error.message);
            throw error;
        }
    }

    /**
     * Create demo student profiles
     */
    async createDemoProfiles() {
        console.log('🎭 Creating demo student profiles...');
        
        const demoStudents = [
            {
                studentId: 'alice_wonderland',
                personalInfo: {
                    name: 'Alice Wonderland',
                    email: 'alice@stream-scholar.com',
                    bio: 'Computer Science student passionate about blockchain',
                    avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=alice'
                }
            },
            {
                studentId: 'bob_builder',
                personalInfo: {
                    name: 'Bob Builder',
                    email: 'bob@stream-scholar.com',
                    bio: 'Engineering student focused on sustainable tech',
                    avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=bob'
                }
            },
            {
                studentId: 'charlie_chocolate',
                personalInfo: {
                    name: 'Charlie Chocolate',
                    email: 'charlie@stream-scholar.com',
                    bio: 'Business student exploring Web3 entrepreneurship',
                    avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=charlie'
                }
            }
        ];

        const results = [];
        
        for (const student of demoStudents) {
            try {
                // Generate keypair for demo student
                const studentKeypair = Keypair.random();
                
                const result = await this.mintStudentProfile(student, studentKeypair);
                
                // Add some initial achievements
                await this.addAchievement(student.studentId, {
                    id: 'welcome_achievement',
                    title: 'Welcome to Stream-Scholar!',
                    description: 'Joined the Stream-Scholar platform',
                    icon: '👋',
                    category: 'onboarding',
                    xpReward: 10,
                    rarity: 'common'
                }, studentKeypair);

                // Add some demo courses
                await this.completeCourse(student.studentId, {
                    id: 'intro_to_blockchain',
                    title: 'Introduction to Blockchain',
                    description: 'Learn the basics of blockchain technology',
                    difficulty: 'beginner',
                    duration: 1800, // 30 minutes
                    category: 'blockchain'
                }, studentKeypair);

                results.push({
                    ...result,
                    studentKeypair: studentKeypair.secret()
                });
                
            } catch (error) {
                console.error(`❌ Failed to create demo profile for ${student.studentId}:`, error.message);
            }
        }
        
        console.log(`🎉 Created ${results.length} demo profiles successfully!`);
        
        // Save demo accounts
        this.saveDemoAccounts(results);
        
        return results;
    }

    /**
     * Save demo account information
     */
    saveDemoAccounts(demoAccounts) {
        const demoData = {
            created: new Date().toISOString(),
            accounts: demoAccounts.map(account => ({
                studentId: account.profile.studentId,
                publicKey: account.profile.nftContractAddress,
                secretKey: account.studentKeypair,
                tokenId: account.nft.tokenId,
                level: account.profile.learning.level,
                xp: account.profile.learning.totalXP
            }))
        };
        
        const demoPath = path.join(__dirname, '../config/demo-accounts.json');
        fs.writeFileSync(demoPath, JSON.stringify(demoData, null, 2));
        console.log(`💾 Demo accounts saved to: ${demoPath}`);
    }

    /**
     * Transfer NFT to new owner
     */
    async transferNFT(studentId, newOwnerPublicKey, signerKeypair) {
        try {
            console.log(`🔄 Transferring NFT for student: ${studentId} to: ${newOwnerPublicKey}`);
            
            const result = await this.integration.transferNFT(studentId, newOwnerPublicKey, signerKeypair);
            
            console.log('✅ NFT transferred successfully!');
            console.log(`🔗 Transaction Hash: ${result.hash}`);
            
            return result;
        } catch (error) {
            console.error('❌ Failed to transfer NFT:', error.message);
            throw error;
        }
    }

    /**
     * Sync profile with blockchain
     */
    async syncProfile(studentId, signerKeypair) {
        try {
            console.log(`🔄 Syncing profile for student: ${studentId}`);
            
            const result = await this.integration.syncWithBlockchain(studentId, signerKeypair);
            
            console.log('✅ Profile synced successfully!');
            console.log(`📊 Profile XP: ${result.profile.xp}`);
            console.log(`⛓️  Blockchain XP: ${result.nft.xp}`);
            
            return result;
        } catch (error) {
            console.error('❌ Failed to sync profile:', error.message);
            throw error;
        }
    }
}

// CLI interface
async function main() {
    const args = process.argv.slice(2);
    const command = args[0];
    const network = args.includes('--standalone') ? 'standalone' : 'testnet';
    
    const minter = new NFTMinter(network);
    
    try {
        switch (command) {
            case 'mint': {
                const studentId = args[1];
                const secretKey = args[2] || process.env.STUDENT_SECRET;
                
                if (!studentId || !secretKey) {
                    console.error('Usage: node mint-nft.js mint <studentId> <secretKey>');
                    process.exit(1);
                }
                
                const keypair = Keypair.fromSecret(secretKey);
                
                await minter.mintStudentProfile({
                    studentId,
                    personalInfo: {
                        name: studentId,
                        email: `${studentId}@example.com`,
                        bio: 'Stream-Scholar student'
                    }
                }, keypair);
                break;
            }
            
            case 'demo': {
                await minter.createDemoProfiles();
                break;
            }
            
            case 'profile': {
                const studentId = args[1];
                if (!studentId) {
                    console.error('Usage: node mint-nft.js profile <studentId>');
                    process.exit(1);
                }
                
                await minter.getStudentProfile(studentId);
                break;
            }
            
            case 'course': {
                const studentId = args[1];
                const secretKey = args[2] || process.env.STUDENT_SECRET;
                
                if (!studentId || !secretKey) {
                    console.error('Usage: node mint-nft.js course <studentId> <secretKey>');
                    process.exit(1);
                }
                
                const keypair = Keypair.fromSecret(secretKey);
                
                await minter.completeCourse(studentId, {
                    id: 'demo_course_001',
                    title: 'Demo Course',
                    description: 'A demonstration course',
                    difficulty: 'beginner',
                    duration: 1800,
                    category: 'demo'
                }, keypair);
                break;
            }
            
            case 'achievement': {
                const studentId = args[1];
                const secretKey = args[2] || process.env.STUDENT_SECRET;
                
                if (!studentId || !secretKey) {
                    console.error('Usage: node mint-nft.js achievement <studentId> <secretKey>');
                    process.exit(1);
                }
                
                const keypair = Keypair.fromSecret(secretKey);
                
                await minter.addAchievement(studentId, {
                    id: 'demo_achievement',
                    title: 'Demo Achievement',
                    description: 'A demonstration achievement',
                    icon: '🏆',
                    category: 'demo',
                    xpReward: 50,
                    rarity: 'common'
                }, keypair);
                break;
            }
            
            default:
                console.log(`
🎓 Stream-Scholar NFT Minter

Usage:
  node mint-nft.js <command> [options]

Commands:
  mint <studentId> <secretKey>     Mint a new student profile NFT
  demo                             Create demo student profiles
  profile <studentId>              Get student profile information
  course <studentId> <secretKey>   Complete a demo course
  achievement <studentId> <secretKey> Add a demo achievement

Options:
  --standalone                     Use standalone network instead of testnet

Examples:
  node mint-nft.js demo
  node mint-nft.js mint alice123 SABC...
  node mint-nft.js profile alice123
                `);
        }
    } catch (error) {
        console.error('💥 Operation failed:', error.message);
        process.exit(1);
    }
}

// Export for use in other modules
module.exports = NFTMinter;

// Run if called directly
if (require.main === module) {
    main().catch(console.error);
}
