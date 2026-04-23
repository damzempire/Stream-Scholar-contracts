#!/usr/bin/env node

/**
 * Mint Student Profile NFT Script
 * This script mints a new Student Profile NFT on Stellar
 */

const { Keypair, Server } = require('@stellar/stellar-sdk');
const StudentProfileNFT = require('../contracts/StudentProfileNFT');
const StudentProfile = require('../src/StudentProfile');

// Configuration
const config = {
    network: Networks.TESTNET,
    horizonUrl: 'https://horizon-testnet.stellar.org',
    // Add your minter account secret key here
    minterSecret: process.env.MINTER_SECRET || null
};

// Sample student data for testing
const sampleStudentData = {
    name: 'Alice Johnson',
    email: 'alice@streamscholar.edu',
    bio: 'Passionate learner exploring blockchain and AI',
    learningStyle: 'visual',
    interests: ['blockchain', 'artificial intelligence', 'web development']
};

async function mintNFT(studentId, studentData) {
    console.log('🎨 Minting Student Profile NFT...');
    
    if (!config.minterSecret) {
        console.error('❌ MINTER_SECRET environment variable is required');
        process.exit(1);
    }
    
    try {
        // Initialize contract
        const nftContract = new StudentProfileNFT(config.network, config.horizonUrl);
        
        // Create minter keypair
        const minterKeypair = Keypair.fromSecret(config.minterSecret);
        console.log(`👤 Minter Account: ${minterKeypair.publicKey()}`);
        
        // Create student profile
        const studentProfile = new StudentProfile(studentId, {
            ...studentData,
            personalInfo: {
                name: studentData.name,
                email: studentData.email,
                bio: studentData.bio,
                learningStyle: studentData.learningStyle
            }
        });
        
        // Add some initial achievements
        studentProfile.addAchievement({
            id: 'first_steps',
            title: 'First Steps',
            description: 'Created your Stream-Scholar profile',
            icon: '👣',
            category: 'milestone',
            xpReward: 10,
            rarity: 'common'
        });
        
        studentProfile.addAchievement({
            id: 'profile_complete',
            title: 'Profile Complete',
            description: 'Completed your profile information',
            icon: '✨',
            category: 'profile',
            xpReward: 25,
            rarity: 'common'
        });
        
        // Prepare NFT metadata
        const nftMetadata = studentProfile.exportForNFT();
        nftMetadata.personalInfo = studentProfile.personalInfo;
        nftMetadata.achievements = studentProfile.achievements;
        
        console.log(`📋 Student ID: ${studentId}`);
        console.log(`📊 Initial Level: ${nftMetadata.level}`);
        console.log(`💫 Initial XP: ${nftMetadata.xp}`);
        console.log(`🏆 Initial Achievements: ${nftMetadata.achievements.length}`);
        
        // Mint the NFT
        console.log('🔨 Building mint transaction...');
        const mintResult = await nftContract.mintNFT(studentId, nftMetadata, minterKeypair);
        
        console.log('✅ NFT minted successfully!');
        console.log(`🎫 Token ID: ${mintResult.tokenId}`);
        console.log(`📋 Transaction Hash: ${mintResult.transaction.hash}`);
        
        // Save mint info
        const mintInfo = {
            tokenId: mintResult.tokenId,
            transactionHash: mintResult.transaction.hash,
            studentId: studentId,
            minterPublicKey: minterKeypair.publicKey(),
            network: config.network,
            mintedAt: new Date().toISOString(),
            profileData: studentProfile.toJSON()
        };
        
        require('fs').writeFileSync(
            `./mint-${studentId}.json`, 
            JSON.stringify(mintInfo, null, 2)
        );
        
        console.log(`💾 Mint info saved to mint-${studentId}.json`);
        console.log('🎉 Student Profile NFT is ready!');
        
        // Display NFT visualization
        console.log('\n🖼️  NFT Preview:');
        console.log('='.repeat(50));
        console.log(`👤 ${studentData.name}`);
        console.log(`🆔 ${studentId}`);
        console.log(`📊 Level ${nftMetadata.level} - ${getLevelName(nftMetadata.level)}`);
        console.log(`💫 ${nftMetadata.xp} XP`);
        console.log(`🏆 ${nftMetadata.achievements.length} Achievements`);
        console.log(`📚 ${nftMetadata.courses} Courses Completed`);
        console.log(`🔥 ${nftMetadata.studyStreak} Day Streak`);
        console.log('='.repeat(50));
        
        return mintInfo;
        
    } catch (error) {
        console.error('❌ Minting failed:', error.message);
        if (error.response && error.response.data) {
            console.error('📄 Error details:', JSON.stringify(error.response.data, null, 2));
        }
        process.exit(1);
    }
}

function getLevelName(level) {
    const levels = {
        1: "Beginner",
        2: "Novice", 
        3: "Apprentice",
        4: "Scholar",
        5: "Expert",
        6: "Master",
        7: "Grandmaster",
        8: "Legend"
    };
    return levels[level] || "Beginner";
}

// Command line interface
async function main() {
    const args = process.argv.slice(2);
    
    if (args.length === 0) {
        console.log('Usage: node mint-nft.js <student-id> [name] [email]');
        console.log('');
        console.log('Example:');
        console.log('  node mint-nft.js student123 "Alice Johnson" alice@example.com');
        console.log('');
        console.log('If no name/email provided, sample data will be used.');
        process.exit(1);
    }
    
    const studentId = args[0];
    const name = args[1] || sampleStudentData.name;
    const email = args[2] || sampleStudentData.email;
    
    const studentData = {
        ...sampleStudentData,
        name: name,
        email: email
    };
    
    await mintNFT(studentId, studentData);
}

// Interactive mode
async function interactiveMode() {
    const readline = require('readline');
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    });
    
    const question = (prompt) => new Promise(resolve => rl.question(prompt, resolve));
    
    console.log('🎨 Stream-Scholar NFT Minting Tool');
    console.log('==================================');
    
    try {
        const studentId = await question('Student ID: ');
        const name = await question('Display Name: ');
        const email = await question('Email (optional): ');
        const bio = await question('Bio (optional): ');
        
        const studentData = {
            ...sampleStudentData,
            name: name || sampleStudentData.name,
            email: email || sampleStudentData.email,
            bio: bio || sampleStudentData.bio
        };
        
        await mintNFT(studentId, studentData);
        
    } catch (error) {
        console.error('❌ Interactive mode failed:', error.message);
    } finally {
        rl.close();
    }
}

// Run script
if (require.main === module) {
    if (process.argv.includes('--interactive') || process.argv.includes('-i')) {
        interactiveMode();
    } else {
        main();
    }
}

module.exports = { mintNFT };
