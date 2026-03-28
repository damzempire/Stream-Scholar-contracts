/**
 * Tests for Student Profile NFT Integration
 * Tests the complete flow from profile creation to NFT management
 */

const StudentProfileNFTIntegration = require('../src/StudentProfileNFTIntegration');
const StudentProfile = require('../src/StudentProfile');
const { Keypair } = require('@stellar/stellar-sdk');

// Mock Stellar SDK for testing
jest.mock('@stellar/stellar-sdk', () => ({
    Keypair: {
        random: () => ({
            publicKey: () => 'TEST_PUBLIC_KEY',
            secret: () => 'TEST_SECRET_KEY'
        }),
        fromSecret: (secret) => ({
            publicKey: () => 'TEST_PUBLIC_KEY',
            secret: () => secret
        })
    },
    Server: jest.fn().mockImplementation(() => ({
        loadAccount: jest.fn().mockResolvedValue({
            accountId: 'TEST_ACCOUNT',
            sequence: 1
        }),
        submitTransaction: jest.fn().mockResolvedValue({
            successful: true,
            hash: 'TEST_TRANSACTION_HASH'
        })
    }),
    TransactionBuilder: jest.fn().mockImplementation(() => ({
        addOperation: jest.fn().mockReturnThis(),
        setTimeout: jest.fn().mockReturnThis(),
        build: jest.fn().mockReturnValue({
            sign: jest.fn().mockReturnThis()
        })
    })),
    Networks: {
        TESTNET: 'TESTNET'
    }
}));

describe('StudentProfileNFTIntegration', () => {
    let integration;
    let testKeypair;

    beforeEach(() => {
        integration = new StudentProfileNFTIntegration('TESTNET');
        testKeypair = Keypair.random();
    });

    describe('Profile Creation', () => {
        test('should create student profile and mint NFT', async () => {
            const studentData = {
                studentId: 'test_student_001',
                personalInfo: {
                    name: 'Test Student',
                    email: 'test@example.com',
                    bio: 'Test bio'
                }
            };

            // Mock the NFT minting
            integration.nftContract.mintNFT = jest.fn().mockResolvedValue({
                tokenId: 'TEST_TOKEN_ID',
                transaction: { hash: 'TEST_HASH' }
            });

            const result = await integration.createStudentProfile(
                studentData.studentId,
                studentData.personalInfo,
                testKeypair
            );

            expect(result.profile).toBeDefined();
            expect(result.profile.studentId).toBe(studentData.studentId);
            expect(result.profile.personalInfo.name).toBe(studentData.personalInfo.name);
            expect(result.nft).toBeDefined();
            expect(result.nft.tokenId).toBe('TEST_TOKEN_ID');
        });

        test('should handle profile creation errors', async () => {
            const studentData = {
                studentId: 'test_student_001',
                personalInfo: { name: 'Test Student' }
            };

            integration.nftContract.mintNFT = jest.fn().mockRejectedValue(
                new Error('Network error')
            );

            await expect(
                integration.createStudentProfile(
                    studentData.studentId,
                    studentData.personalInfo,
                    testKeypair
                )
            ).rejects.toThrow('Network error');
        });
    });

    describe('XP Management', () => {
        test('should add XP and update level', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            integration.profiles.set(studentId, profile);

            // Mock NFT update
            integration.nftContract.updateXP = jest.fn().mockResolvedValue({
                newXP: 150,
                newLevel: 2,
                transaction: { hash: 'TEST_HASH' }
            });

            const result = await integration.addStudentXP(
                studentId,
                150,
                'course_completion',
                { courseId: 'test_course' },
                testKeypair
            );

            expect(result.newXP).toBe(150);
            expect(result.levelUp).toBe(true);
            expect(profile.learning.totalXP).toBe(150);
            expect(profile.learning.level).toBe(2);
        });

        test('should handle level up achievements', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            profile.addXP(250); // Level 3 threshold
            integration.profiles.set(studentId, profile);

            integration.nftContract.updateXP = jest.fn().mockResolvedValue({
                newXP: 250,
                newLevel: 3,
                transaction: { hash: 'TEST_HASH' }
            });

            const spy = jest.spyOn(integration, 'handleLevelUp');

            await integration.addStudentXP(
                studentId,
                250,
                'achievement',
                {},
                testKeypair
            );

            expect(spy).toHaveBeenCalledWith(profile, testKeypair);
        });
    });

    describe('Course Completion', () => {
        test('should complete course and award XP', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            integration.profiles.set(studentId, profile);

            const courseData = {
                id: 'test_course_001',
                title: 'Test Course',
                difficulty: 'beginner',
                duration: 1800,
                category: 'test'
            };

            const result = await integration.completeCourse(studentId, courseData, testKeypair);

            expect(result.completed).toBe(true);
            expect(result.progress).toBe(100);
            expect(profile.learning.courses).toHaveLength(1);
            expect(profile.learning.totalXP).toBeGreaterThan(0);
        });

        test('should add course completion achievement', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            integration.profiles.set(studentId, profile);

            const courseData = {
                id: 'test_course_001',
                title: 'Test Course',
                difficulty: 'beginner',
                duration: 1800,
                category: 'test'
            };

            await integration.completeCourse(studentId, courseData, testKeypair);

            const courseAchievement = profile.achievements.find(
                a => a.title === 'Completed: Test Course'
            );
            expect(courseAchievement).toBeDefined();
        });
    });

    describe('Achievement Management', () => {
        test('should add achievement to profile and NFT', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            integration.profiles.set(studentId, profile);

            const achievementData = {
                id: 'test_achievement',
                title: 'Test Achievement',
                description: 'Test achievement description',
                icon: '🏆',
                category: 'test',
                xpReward: 50,
                rarity: 'common'
            };

            integration.nftContract.addAchievement = jest.fn().mockResolvedValue({
                hash: 'TEST_HASH'
            });

            const result = await integration.addAchievement(
                studentId,
                achievementData,
                testKeypair
            );

            expect(result).toBeDefined();
            expect(result.title).toBe(achievementData.title);
            expect(profile.achievements).toHaveLength(1);
            expect(profile.learning.totalXP).toBe(50); // XP reward
        });

        test('should not add duplicate achievements', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            const achievementData = {
                id: 'test_achievement',
                title: 'Test Achievement'
            };

            // Add achievement first time
            profile.addAchievement(achievementData);
            integration.profiles.set(studentId, profile);

            const result = await integration.addAchievement(
                studentId,
                achievementData,
                testKeypair
            );

            expect(result).toBeNull(); // Should return null for duplicate
            expect(profile.achievements).toHaveLength(1); // Still only one
        });
    });

    describe('Level Management', () => {
        test('should handle level up correctly', async () => {
            const profile = new StudentProfile('test_student');
            profile.addXP(500); // Level 4

            integration.nftContract.addAchievement = jest.fn().mockResolvedValue({
                hash: 'TEST_HASH'
            });

            await integration.handleLevelUp(profile, testKeypair);

            const levelUpAchievement = profile.achievements.find(
                a => a.title.includes('Level 4')
            );
            expect(levelUpAchievement).toBeDefined();
            expect(levelUpAchievement.xpReward).toBe(40); // 4 * 10
        });

        test('should award correct XP for different levels', async () => {
            const profile = new StudentProfile('test_student');
            
            // Test level 6 (should be epic rarity)
            profile.addXP(2000); // Level 6
            integration.nftContract.addAchievement = jest.fn().mockResolvedValue({
                hash: 'TEST_HASH'
            });

            await integration.handleLevelUp(profile, testKeypair);

            const levelUpAchievement = profile.achievements.find(
                a => a.title.includes('Level 6')
            );
            expect(levelUpAchievement.rarity).toBe('epic');
        });
    });

    describe('Profile Retrieval', () => {
        test('should get complete profile information', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId, {
                personalInfo: { name: 'Test Student' }
            });
            profile.addXP(100);
            profile.addAchievement({
                id: 'test_achievement',
                title: 'Test Achievement'
            });

            integration.profiles.set(studentId, profile);

            integration.nftContract.getNFTData = jest.fn().mockResolvedValue({
                tokenId: 'TEST_TOKEN',
                xp: 100,
                level: 2
            });

            const result = await integration.getCompleteProfile(studentId);

            expect(result.profile).toBeDefined();
            expect(result.stats).toBeDefined();
            expect(result.metadata).toBeDefined();
            expect(result.stats.totalXP).toBe(100);
            expect(result.stats.achievementsUnlocked).toBe(1);
        });

        test('should load profile from blockchain if not cached', async () => {
            const studentId = 'test_student_001';

            integration.nftContract.getStudentData = jest.fn().mockResolvedValue({
                tokenId: 'TEST_TOKEN'
            });

            integration.nftContract.getNFTData = jest.fn().mockResolvedValue({
                tokenId: 'TEST_TOKEN',
                xp: 250,
                level: 3,
                createdAt: '2023-01-01T00:00:00Z',
                updatedAt: '2023-01-01T00:00:00Z'
            });

            const profile = await integration.getOrCreateProfile(studentId);

            expect(profile).toBeDefined();
            expect(profile.learning.totalXP).toBe(250);
            expect(profile.learning.level).toBe(3);
            expect(integration.profiles.has(studentId)).toBe(true);
        });
    });

    describe('NFT Transfer', () => {
        test('should transfer NFT to new owner', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            profile.nftTokenId = 'TEST_TOKEN_ID';
            integration.profiles.set(studentId, profile);

            const newOwnerAddress = 'NEW_OWNER_ADDRESS';

            integration.nftContract.transferNFT = jest.fn().mockResolvedValue({
                hash: 'TRANSFER_HASH'
            });

            const result = await integration.transferNFT(
                studentId,
                newOwnerAddress,
                testKeypair
            );

            expect(result).toBeDefined();
            expect(profile.nftOwner).toBe(newOwnerAddress);
            expect(integration.nftContract.transferNFT).toHaveBeenCalledWith(
                'TEST_TOKEN_ID',
                newOwnerAddress,
                testKeypair
            );
        });

        test('should handle transfer errors', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            // No NFT token ID set

            await expect(
                integration.transferNFT(studentId, 'NEW_OWNER', testKeypair)
            ).rejects.toThrow('No NFT found for this student profile');
        });
    });

    describe('Blockchain Sync', () => {
        test('should sync profile with blockchain', async () => {
            const studentId = 'test_student_001';
            const profile = new StudentProfile(studentId);
            profile.nftTokenId = 'TEST_TOKEN_ID';
            profile.addXP(300); // Profile has 300 XP
            integration.profiles.set(studentId, profile);

            // Blockchain has 250 XP
            integration.nftContract.getNFTData = jest.fn().mockResolvedValue({
                tokenId: 'TEST_TOKEN_ID',
                xp: 250,
                level: 3
            });

            integration.nftContract.updateXP = jest.fn().mockResolvedValue({
                hash: 'SYNC_HASH'
            });

            integration.nftContract.addAchievement = jest.fn().mockResolvedValue({
                hash: 'SYNC_HASH'
            });

            const result = await integration.syncWithBlockchain(studentId, testKeypair);

            expect(result.synced).toBe(true);
            expect(integration.nftContract.updateXP).toHaveBeenCalledWith(
                studentId,
                50, // Difference: 300 - 250
                testKeypair
            );
        });
    });

    describe('Metadata Generation', () => {
        test('should generate proper NFT metadata', () => {
            const profile = new StudentProfile('test_student', {
                personalInfo: { name: 'Test Student' }
            });
            profile.addXP(150);
            profile.addAchievement({
                id: 'test_achievement',
                title: 'Test Achievement'
            });

            const metadata = integration.prepareNFTMetadata(profile);

            expect(metadata.name).toBe('Stream-Scholar Profile: test_student');
            expect(metadata.description).toContain('Level 2');
            expect(metadata.attributes).toBeDefined();
            expect(metadata.attributes).toEqual(
                expect.arrayContaining([
                    expect.objectContaining({
                        trait_type: 'Student ID',
                        value: 'test_student'
                    }),
                    expect.objectContaining({
                        trait_type: 'Level',
                        value: 2
                    }),
                    expect.objectContaining({
                        trait_type: 'XP',
                        value: 150
                    })
                ])
            );
        });
    });
});

describe('StudentProfile', () => {
    let profile;

    beforeEach(() => {
        profile = new StudentProfile('test_student');
    });

    describe('XP System', () => {
        test('should add XP correctly', () => {
            const result = profile.addXP(100, 'test');

            expect(result.newXP).toBe(100);
            expect(result.levelUp).toBe(true); // Should level up to level 2
            expect(profile.learning.totalXP).toBe(100);
            expect(profile.learning.level).toBe(2);
        });

        test('should track XP gains with metadata', () => {
            const metadata = { courseId: 'test_course' };
            profile.addXP(50, 'course_completion', metadata);

            expect(profile.learning.totalXP).toBe(50);
        });

        test('should calculate level correctly', () => {
            profile.addXP(500); // Should be level 4

            expect(profile.learning.level).toBe(4);
        });
    });

    describe('Achievement System', () => {
        test('should add achievements correctly', () => {
            const achievement = {
                id: 'test_achievement',
                title: 'Test Achievement',
                description: 'Test description',
                xpReward: 25
            };

            const result = profile.addAchievement(achievement);

            expect(result).toBeDefined();
            expect(result.title).toBe('Test Achievement');
            expect(profile.achievements).toHaveLength(1);
            expect(profile.learning.totalXP).toBe(25); // XP reward added
        });

        test('should prevent duplicate achievements', () => {
            const achievement = {
                id: 'test_achievement',
                title: 'Test Achievement'
            };

            profile.addAchievement(achievement);
            const result = profile.addAchievement(achievement);

            expect(result).toBeNull();
            expect(profile.achievements).toHaveLength(1);
        });
    });

    describe('Course System', () => {
        test('should add courses correctly', () => {
            const course = {
                id: 'test_course',
                title: 'Test Course',
                difficulty: 'beginner',
                duration: 1800
            };

            const result = profile.addCourse(course);

            expect(result.id).toBe('test_course');
            expect(profile.learning.courses).toHaveLength(1);
        });

        test('should complete courses and award XP', () => {
            const course = {
                id: 'test_course',
                title: 'Test Course',
                difficulty: 'beginner',
                duration: 1800
            };

            profile.addCourse(course);
            const result = profile.updateCourseProgress('test_course', 100, true);

            expect(result.completed).toBe(true);
            expect(profile.learning.totalXP).toBeGreaterThan(0);
        });
    });

    describe('Study Streak', () => {
        test('should update study streak correctly', () => {
            const today = new Date();
            profile.learning.lastStudyDate = new Date(today.getTime() - 24 * 60 * 60 * 1000).toISOString(); // Yesterday

            profile.updateStudyStreak();

            expect(profile.learning.studyStreak).toBe(2); // Should increment
        });

        test('should reset streak if day missed', () => {
            const threeDaysAgo = new Date(Date.now() - 3 * 24 * 60 * 60 * 1000);
            profile.learning.lastStudyDate = threeDaysAgo.toISOString();

            profile.updateStudyStreak();

            expect(profile.learning.studyStreak).toBe(1); // Should reset
        });
    });

    describe('Validation', () => {
        test('should validate profile correctly', () => {
            const validProfile = new StudentProfile('test_student');
            const validation = validProfile.validate();

            expect(validation.isValid).toBe(true);
            expect(validation.errors).toHaveLength(0);
        });

        test('should detect invalid profile data', () => {
            const invalidProfile = new StudentProfile('');
            invalidProfile.learning.totalXP = -1;

            const validation = invalidProfile.validate();

            expect(validation.isValid).toBe(false);
            expect(validation.errors.length).toBeGreaterThan(0);
        });
    });
});
