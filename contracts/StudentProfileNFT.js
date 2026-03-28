const { TransactionBuilder, Networks, Operation, Asset, Keypair, StrKey, Server } = require('@stellar/stellar-sdk');

/**
 * Student Profile NFT Contract for Stellar
 * Creates non-fungible tokens that represent student learning profiles
 * with dynamic leveling based on learning achievements
 */
class StudentProfileNFT {
    constructor(networkPassphrase, horizonUrl) {
        this.networkPassphrase = networkPassphrase || Networks.TESTNET;
        this.horizonUrl = horizonUrl || 'https://horizon-testnet.stellar.org';
        this.server = new Server(this.horizonUrl);
        
        // Contract configuration
        this.contractCode = this.generateContractCode();
        this.levels = {
            1: { name: "Beginner", requiredXP: 0, color: "#808080" },
            2: { name: "Novice", requiredXP: 100, color: "#C0C0C0" },
            3: { name: "Apprentice", requiredXP: 250, color: "#CD7F32" },
            4: { name: "Scholar", requiredXP: 500, color: "#FFD700" },
            5: { name: "Expert", requiredXP: 1000, color: "#50C878" },
            6: { name: "Master", requiredXP: 2000, color: "#4169E1" },
            7: { name: "Grandmaster", requiredXP: 5000, color: "#FF1493" },
            8: { name: "Legend", requiredXP: 10000, color: "#FF4500" }
        };
    }

    /**
     * Generate the Stellar smart contract code for NFT functionality
     */
    generateContractCode() {
        return `
            // Student Profile NFT Contract
            // Implements ERC-721-like functionality on Stellar
            
            // Storage structure
            // Data Entry: "NFT_{tokenId}" -> { owner, studentId, level, xp, metadata }
            // Data Entry: "STUDENT_{studentId}" -> { tokenId, achievements }
            
            export function create_nft(student_id: String, initial_metadata: String) -> Void {
                let token_id = generate_token_id();
                let nft_data = {
                    owner: get_current_signer(),
                    student_id: student_id,
                    level: 1,
                    xp: 0,
                    metadata: initial_metadata,
                    created_at: now(),
                    updated_at: now()
                };
                
                put_data("NFT_" + token_id, nft_data);
                put_data("STUDENT_" + student_id, { token_id: token_id, achievements: [] });
                
                // Mint the NFT as a unique asset
                let asset_name = "STUDENT_" + token_id;
                let nft_asset = new Asset(
                    asset_name,
                    get_current_signer()
                );
                
                // Create payment to establish ownership
                create_payment(nft_asset, 1);
            }
            
            export function update_xp(student_id: String, xp_amount: i64) -> Void {
                let student_data = get_data("STUDENT_" + student_id);
                let nft_data = get_data("NFT_" + student_data.token_id);
                
                nft_data.xp = nft_data.xp + xp_amount;
                nft_data.level = calculate_level(nft_data.xp);
                nft_data.updated_at = now();
                
                put_data("NFT_" + student_data.token_id, nft_data);
            }
            
            export function add_achievement(student_id: String, achievement: String) -> Void {
                let student_data = get_data("STUDENT_" + student_id);
                student_data.achievements.push(achievement);
                put_data("STUDENT_" + student_id, student_data);
            }
            
            export function transfer_nft(token_id: String, new_owner: Address) -> Void {
                let nft_data = get_data("NFT_" + token_id);
                require(nft_data.owner == get_current_signer(), "Only owner can transfer");
                
                nft_data.owner = new_owner;
                put_data("NFT_" + token_id, nft_data);
            }
            
            function calculate_level(xp: i64) -> i64 {
                if (xp < 100) return 1;
                if (xp < 250) return 2;
                if (xp < 500) return 3;
                if (xp < 1000) return 4;
                if (xp < 2000) return 5;
                if (xp < 5000) return 6;
                if (xp < 10000) return 7;
                return 8;
            }
        `;
    }

    /**
     * Deploy the Student Profile NFT contract to Stellar
     */
    async deployContract(distributorKeypair) {
        try {
            const account = await this.server.loadAccount(distributorKeypair.publicKey());
            
            const transaction = new TransactionBuilder(account, {
                fee: '100',
                networkPassphrase: this.networkPassphrase
            })
            .addOperation(Operation.createCustomContract({
                contractCode: this.contractCode,
                source: distributorKeypair.publicKey()
            }))
            .setTimeout(30)
            .build();

            transaction.sign(distributorKeypair);
            
            const result = await this.server.submitTransaction(transaction);
            console.log('Contract deployed successfully:', result.hash);
            return result;
        } catch (error) {
            console.error('Error deploying contract:', error);
            throw error;
        }
    }

    /**
     * Mint a new Student Profile NFT
     */
    async mintNFT(studentId, metadata, issuerKeypair) {
        try {
            const account = await this.server.loadAccount(issuerKeypair.publicKey());
            const tokenId = this.generateTokenId();
            
            // Create unique asset for the NFT
            const nftAsset = new Asset(`STUDENT_${tokenId}`, issuerKeypair.publicKey());
            
            const transaction = new TransactionBuilder(account, {
                fee: '100',
                networkPassphrase: this.networkPassphrase
            })
            .addOperation(Operation.createAccount({
                destination: issuerKeypair.publicKey(),
                startingBalance: '2'
            }))
            .addOperation(Operation.changeTrust({
                asset: nftAsset,
                limit: '1'
            }))
            .addOperation(Operation.payment({
                destination: issuerKeypair.publicKey(),
                asset: nftAsset,
                amount: '1'
            }))
            .addOperation(Operation.manageData({
                name: `NFT_${tokenId}`,
                value: JSON.stringify({
                    owner: issuerKeypair.publicKey(),
                    studentId: studentId,
                    level: 1,
                    xp: 0,
                    metadata: metadata,
                    createdAt: new Date().toISOString(),
                    updatedAt: new Date().toISOString()
                })
            }))
            .addOperation(Operation.manageData({
                name: `STUDENT_${studentId}`,
                value: JSON.stringify({
                    tokenId: tokenId,
                    achievements: []
                })
            }))
            .setTimeout(30)
            .build();

            transaction.sign(issuerKeypair);
            
            const result = await this.server.submitTransaction(transaction);
            console.log('NFT minted successfully:', result.hash);
            return { tokenId, transaction: result };
        } catch (error) {
            console.error('Error minting NFT:', error);
            throw error;
        }
    }

    /**
     * Update student XP and level
     */
    async updateXP(studentId, xpAmount, signerKeypair) {
        try {
            const studentData = await this.getStudentData(studentId);
            const nftData = await this.getNFTData(studentData.tokenId);
            
            const newXP = nftData.xp + xpAmount;
            const newLevel = this.calculateLevel(newXP);
            
            const account = await this.server.loadAccount(signerKeypair.publicKey());
            
            const transaction = new TransactionBuilder(account, {
                fee: '100',
                networkPassphrase: this.networkPassphrase
            })
            .addOperation(Operation.manageData({
                name: `NFT_${studentData.tokenId}`,
                value: JSON.stringify({
                    ...nftData,
                    xp: newXP,
                    level: newLevel,
                    updatedAt: new Date().toISOString()
                })
            }))
            .setTimeout(30)
            .build();

            transaction.sign(signerKeypair);
            
            const result = await this.server.submitTransaction(transaction);
            console.log('XP updated successfully:', result.hash);
            return { newXP, newLevel, transaction: result };
        } catch (error) {
            console.error('Error updating XP:', error);
            throw error;
        }
    }

    /**
     * Add achievement to student profile
     */
    async addAchievement(studentId, achievement, signerKeypair) {
        try {
            const studentData = await this.getStudentData(studentId);
            studentData.achievements.push(achievement);
            
            const account = await this.server.loadAccount(signerKeypair.publicKey());
            
            const transaction = new TransactionBuilder(account, {
                fee: '100',
                networkPassphrase: this.networkPassphrase
            })
            .addOperation(Operation.manageData({
                name: `STUDENT_${studentId}`,
                value: JSON.stringify(studentData)
            }))
            .setTimeout(30)
            .build();

            transaction.sign(signerKeypair);
            
            const result = await this.server.submitTransaction(transaction);
            console.log('Achievement added successfully:', result.hash);
            return result;
        } catch (error) {
            console.error('Error adding achievement:', error);
            throw error;
        }
    }

    /**
     * Get student data from the blockchain
     */
    async getStudentData(studentId) {
        try {
            const account = await this.server.accounts()
                .forSigner(studentId)
                .call();
            
            const dataEntry = account.data.find(entry => entry.name === `STUDENT_${studentId}`);
            if (dataEntry) {
                return JSON.parse(dataEntry.value);
            }
            throw new Error('Student data not found');
        } catch (error) {
            console.error('Error getting student data:', error);
            throw error;
        }
    }

    /**
     * Get NFT data from the blockchain
     */
    async getNFTData(tokenId) {
        try {
            // Find the account that holds this NFT
            const accounts = await this.server.accounts()
                .forSigner(tokenId)
                .call();
            
            const dataEntry = accounts.data.find(entry => entry.name === `NFT_${tokenId}`);
            if (dataEntry) {
                return JSON.parse(dataEntry.value);
            }
            throw new Error('NFT data not found');
        } catch (error) {
            console.error('Error getting NFT data:', error);
            throw error;
        }
    }

    /**
     * Generate unique token ID
     */
    generateTokenId() {
        return 'SP_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
    }

    /**
     * Calculate student level based on XP
     */
    calculateLevel(xp) {
        for (let level = 8; level >= 1; level--) {
            if (xp >= this.levels[level].requiredXP) {
                return level;
            }
        }
        return 1;
    }

    /**
     * Get level information
     */
    getLevelInfo(level) {
        return this.levels[level] || this.levels[1];
    }

    /**
     * Generate NFT metadata with visual properties
     */
    generateNFTMetadata(studentData) {
        const levelInfo = this.getLevelInfo(studentData.level);
        
        return {
            name: `Stream-Scholar Profile: ${studentData.studentId}`,
            description: `A dynamic NFT representing the learning journey of ${studentData.studentId}. Level ${studentData.level} ${levelInfo.name} with ${studentData.xp} XP.`,
            image: this.generateNFTImage(studentData),
            attributes: [
                {
                    trait_type: "Level",
                    value: studentData.level
                },
                {
                    trait_type: "Title",
                    value: levelInfo.name
                },
                {
                    trait_type: "XP",
                    value: studentData.xp
                },
                {
                    trait_type: "Achievements",
                    value: studentData.achievements.length
                },
                {
                    trait_type: "Created",
                    value: studentData.createdAt
                }
            ],
            properties: {
                level_color: levelInfo.color,
                next_level_xp: this.getNextLevelXP(studentData.level),
                progress_to_next: this.calculateProgress(studentData.xp, studentData.level)
            }
        };
    }

    /**
     * Generate SVG image for NFT
     */
    generateNFTImage(studentData) {
        const levelInfo = this.getLevelInfo(studentData.level);
        const progress = this.calculateProgress(studentData.xp, studentData.level);
        
        const svg = `
            <svg width="400" height="400" xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
                        <stop offset="0%" style="stop-color:${levelInfo.color};stop-opacity:0.3" />
                        <stop offset="100%" style="stop-color:${levelInfo.color};stop-opacity:0.8" />
                    </linearGradient>
                </defs>
                
                <rect width="400" height="400" fill="url(#bg)" />
                
                <circle cx="200" cy="150" r="60" fill="${levelInfo.color}" opacity="0.8" />
                <text x="200" y="160" text-anchor="middle" fill="white" font-size="36" font-weight="bold">Lv${studentData.level}</text>
                
                <text x="200" y="250" text-anchor="middle" fill="white" font-size="18">${levelInfo.name}</text>
                <text x="200" y="280" text-anchor="middle" fill="white" font-size="14">XP: ${studentData.xp}</text>
                
                <rect x="50" y="320" width="300" height="20" fill="rgba(255,255,255,0.3)" rx="10" />
                <rect x="50" y="320" width="${300 * progress}" height="20" fill="${levelInfo.color}" rx="10" />
                
                <text x="200" y="355" text-anchor="middle" fill="white" font-size="12">${Math.round(progress * 100)}% to next level</text>
                
                <text x="200" y="385" text-anchor="middle" fill="white" font-size="10" opacity="0.8">${studentData.studentId}</text>
            </svg>
        `;
        
        return 'data:image/svg+xml;base64,' + Buffer.from(svg).toString('base64');
    }

    /**
     * Calculate progress to next level
     */
    calculateProgress(xp, currentLevel) {
        if (currentLevel >= 8) return 1; // Max level
        
        const currentLevelXP = this.levels[currentLevel].requiredXP;
        const nextLevelXP = this.levels[currentLevel + 1].requiredXP;
        
        return (xp - currentLevelXP) / (nextLevelXP - currentLevelXP);
    }

    /**
     * Get XP required for next level
     */
    getNextLevelXP(currentLevel) {
        if (currentLevel >= 8) return null; // Max level
        return this.levels[currentLevel + 1].requiredXP;
    }
}

module.exports = StudentProfileNFT;
