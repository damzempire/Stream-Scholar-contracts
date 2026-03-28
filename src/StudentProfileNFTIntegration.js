/**
 * Integration layer between StudentProfile and NFT contract
 * Bridges the gap between student data management and blockchain NFT representation
 */

const StudentProfile = require('./StudentProfile');
const StudentProfileNFT = require('../contracts/StudentProfileNFT');

class StudentProfileNFTIntegration {
    constructor(networkPassphrase = 'TESTNET', horizonUrl = 'https://horizon-testnet.stellar.org') {
        this.nftContract = new StudentProfileNFT(networkPassphrase, horizonUrl);
        this.profiles = new Map(); // In-memory cache of student profiles
    }

    /**
     * Create a new student profile and mint corresponding NFT
     */
    async createStudentProfile(studentId, personalInfo, issuerKeypair) {
        try {
            // Create student profile
            const profile = new StudentProfile(studentId, {
                ...personalInfo,
                createdAt: new Date().toISOString()
            });

            // Cache the profile
            this.profiles.set(studentId, profile);

            // Prepare NFT metadata
            const nftMetadata = this.prepareNFTMetadata(profile);

            // Mint the NFT
            const nftResult = await this.nftContract.mintNFT(
                studentId,
                JSON.stringify(nftMetadata),
                issuerKeypair
            );

            // Update profile with NFT information
            profile.nftTokenId = nftResult.tokenId;
            profile.nftContractAddress = issuerKeypair.publicKey();

            return {
                profile,
                nft: nftResult
            };
        } catch (error) {
            console.error('Error creating student profile NFT:', error);
            throw error;
        }
    }

    /**
     * Update student XP and sync with NFT
     */
    async addStudentXP(studentId, xpAmount, source, metadata, signerKeypair) {
        try {
            // Get or create profile
            let profile = this.profiles.get(studentId);
            if (!profile) {
                // Load from blockchain if not in cache
                profile = await this.loadProfileFromBlockchain(studentId);
            }

            // Add XP to profile
            const xpResult = profile.addXP(xpAmount, source, metadata);

            // Update NFT with new XP and level
            if (profile.nftTokenId) {
                await this.nftContract.updateXP(studentId, xpAmount, signerKeypair);
            }

            // Check for level up achievements
            if (xpResult.levelUp) {
                await this.handleLevelUp(profile, signerKeypair);
            }

            return xpResult;
        } catch (error) {
            console.error('Error adding student XP:', error);
            throw error;
        }
    }

    /**
     * Complete a course and update profile/NFT
     */
    async completeCourse(studentId, courseData, signerKeypair) {
        try {
            const profile = await this.getOrCreateProfile(studentId);
            
            // Update course progress
            const course = profile.updateCourseProgress(courseData.id, 100, true);
            
            // Calculate and award XP
            const xpEarned = profile.calculateCourseXP(course);
            await this.addStudentXP(studentId, xpEarned, 'course_completion', {
                courseId: course.id,
                title: course.title
            }, signerKeypair);

            // Add course completion achievement
            profile.addAchievement({
                id: `course_${course.id}_completed`,
                title: `Completed: ${course.title}`,
                description: `Successfully completed ${course.title}`,
                icon: '🎓',
                category: 'courses',
                xpReward: 0, // XP already awarded above
                rarity: 'common'
            });

            // Update NFT with achievement
            if (profile.nftTokenId) {
                await this.nftContract.addAchievement(
                    studentId,
                    `Completed: ${course.title}`,
                    signerKeypair
                );
            }

            return course;
        } catch (error) {
            console.error('Error completing course:', error);
            throw error;
        }
    }

    /**
     * Add achievement to profile and NFT
     */
    async addAchievement(studentId, achievement, signerKeypair) {
        try {
            const profile = await this.getOrCreateProfile(studentId);
            
            // Add to profile
            const achievementData = profile.addAchievement(achievement);

            if (achievementData && profile.nftTokenId) {
                // Add to NFT
                await this.nftContract.addAchievement(
                    studentId,
                    achievementData.title,
                    signerKeypair
                );
            }

            return achievementData;
        } catch (error) {
            console.error('Error adding achievement:', error);
            throw error;
        }
    }

    /**
     * Handle level up events
     */
    async handleLevelUp(profile, signerKeypair) {
        const levelInfo = profile.calculateLevel(profile.learning.totalXP);
        
        // Add level up achievement
        await this.addAchievement(profile.studentId, {
            id: `level_${levelInfo.level}`,
            title: `Level ${levelInfo.level}: ${levelInfo.name}`,
            description: `Reached ${levelInfo.name} level with ${profile.learning.totalXP} XP`,
            icon: '⬆️',
            category: 'leveling',
            xpReward: levelInfo.level * 10,
            rarity: levelInfo.level >= 6 ? 'epic' : levelInfo.level >= 4 ? 'rare' : 'common'
        }, signerKeypair);
    }

    /**
     * Get student profile from cache or blockchain
     */
    async getOrCreateProfile(studentId) {
        let profile = this.profiles.get(studentId);
        if (!profile) {
            profile = await this.loadProfileFromBlockchain(studentId);
        }
        return profile;
    }

    /**
     * Load profile data from blockchain
     */
    async loadProfileFromBlockchain(studentId) {
        try {
            const studentData = await this.nftContract.getStudentData(studentId);
            const nftData = await this.nftContract.getNFTData(studentData.tokenId);

            const profile = new StudentProfile(studentId, {
                totalXP: nftData.xp,
                level: nftData.level,
                achievements: studentData.achievements,
                nftTokenId: studentData.tokenId,
                createdAt: nftData.createdAt,
                updatedAt: nftData.updatedAt
            });

            this.profiles.set(studentId, profile);
            return profile;
        } catch (error) {
            console.error('Error loading profile from blockchain:', error);
            throw new Error(`Profile not found for student: ${studentId}`);
        }
    }

    /**
     * Prepare NFT metadata from profile
     */
    prepareNFTMetadata(profile) {
        const profileData = profile.exportForNFT();
        return {
            name: `Stream-Scholar Profile: ${profile.studentId}`,
            description: `Dynamic NFT representing learning journey - Level ${profileData.level} with ${profileData.xp} XP`,
            image: this.nftContract.generateNFTImage(profileData),
            attributes: [
                {
                    trait_type: "Student ID",
                    value: profile.studentId
                },
                {
                    trait_type: "Level",
                    value: profileData.level
                },
                {
                    trait_type: "XP",
                    value: profileData.xp
                },
                {
                    trait_type: "Achievements",
                    value: profileData.achievements
                },
                {
                    trait_type: "Courses Completed",
                    value: profileData.courses
                },
                {
                    trait_type: "Study Streak",
                    value: profileData.studyStreak
                }
            ],
            external_url: `https://stream-scholar.com/profile/${profile.studentId}`,
            properties: {
                student_data: profileData,
                last_updated: new Date().toISOString()
            }
        };
    }

    /**
     * Get complete profile with NFT information
     */
    async getCompleteProfile(studentId) {
        const profile = await this.getOrCreateProfile(studentId);
        const profileData = profile.exportForNFT();
        
        let nftData = null;
        if (profile.nftTokenId) {
            try {
                nftData = await this.nftContract.getNFTData(profile.nftTokenId);
            } catch (error) {
                console.warn('Could not load NFT data:', error.message);
            }
        }

        return {
            profile: profile.toJSON(),
            nft: nftData,
            metadata: this.prepareNFTMetadata(profile),
            stats: profile.getStats(),
            levelProgress: profile.getLevelProgress()
        };
    }

    /**
     * Transfer NFT to new owner
     */
    async transferNFT(studentId, newOwnerAddress, signerKeypair) {
        try {
            const profile = await this.getOrCreateProfile(studentId);
            
            if (!profile.nftTokenId) {
                throw new Error('No NFT found for this student profile');
            }

            const result = await this.nftContract.transferNFT(
                profile.nftTokenId,
                newOwnerAddress,
                signerKeypair
            );

            // Update profile ownership
            profile.nftOwner = newOwnerAddress;

            return result;
        } catch (error) {
            console.error('Error transferring NFT:', error);
            throw error;
        }
    }

    /**
     * Get NFT metadata for display
     */
    async getNFTMetadata(studentId) {
        const profile = await this.getOrCreateProfile(studentId);
        return this.prepareNFTMetadata(profile);
    }

    /**
     * Sync profile with blockchain
     */
    async syncWithBlockchain(studentId, signerKeypair) {
        try {
            const profile = await this.getOrCreateProfile(studentId);
            
            if (!profile.nftTokenId) {
                throw new Error('No NFT associated with this profile');
            }

            // Update NFT with latest profile data
            const nftData = await this.nftContract.getNFTData(profile.nftTokenId);
            const profileData = profile.exportForNFT();

            // Update XP if different
            if (nftData.xp !== profileData.xp) {
                const xpDiff = profileData.xp - nftData.xp;
                await this.nftContract.updateXP(studentId, xpDiff, signerKeypair);
            }

            // Update achievements if different
            const currentAchievements = profile.achievements.map(a => a.title);
            const nftAchievements = nftData.achievements || [];
            
            for (const achievement of currentAchievements) {
                if (!nftAchievements.includes(achievement)) {
                    await this.nftContract.addAchievement(studentId, achievement, signerKeypair);
                }
            }

            return {
                synced: true,
                profile: profileData,
                nft: nftData
            };
        } catch (error) {
            console.error('Error syncing with blockchain:', error);
            throw error;
        }
    }
}

module.exports = StudentProfileNFTIntegration;
