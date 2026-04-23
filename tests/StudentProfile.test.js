/**
 * Student Profile Tests
 * Test suite for the StudentProfile class functionality
 */

const StudentProfile = require('../src/StudentProfile');

describe('StudentProfile', () => {
    let profile;

    beforeEach(() => {
        profile = new StudentProfile('test123', {
            name: 'Test Student',
            email: 'test@example.com',
            bio: 'Test bio'
        });
    });

    describe('Constructor', () => {
        test('should create profile with default values', () => {
            const emptyProfile = new StudentProfile('empty123');
            
            expect(emptyProfile.studentId).toBe('empty123');
            expect(emptyProfile.learning.totalXP).toBe(0);
            expect(emptyProfile.learning.level).toBe(1);
            expect(emptyProfile.achievements).toEqual([]);
            expect(emptyProfile.learning.courses).toEqual([]);
        });

        test('should create profile with provided data', () => {
            expect(profile.studentId).toBe('test123');
            expect(profile.personalInfo.name).toBe('Test Student');
            expect(profile.personalInfo.email).toBe('test@example.com');
            expect(profile.personalInfo.bio).toBe('Test bio');
        });

        test('should set creation timestamp', () => {
            const now = new Date().toISOString();
            expect(profile.createdAt).toBeDefined();
            expect(new Date(profile.createdAt).getTime()).toBeCloseTo(new Date(now).getTime(), -2);
        });
    });

    describe('XP Management', () => {
        test('should add XP correctly', () => {
            const result = profile.addXP(100, 'test');
            
            expect(profile.learning.totalXP).toBe(100);
            expect(result.newXP).toBe(100);
            expect(result.previousXP).toBe(0);
        });

        test('should level up when reaching XP threshold', () => {
            const result = profile.addXP(250, 'test');
            
            expect(profile.learning.level).toBe(3); // Should be level 3 (Apprentice)
            expect(result.levelUp).toBe(true);
        });

        test('should not level up if not enough XP', () => {
            const result = profile.addXP(50, 'test');
            
            expect(profile.learning.level).toBe(1);
            expect(result.levelUp).toBe(false);
        });

        test('should update timestamp when adding XP', () => {
            const originalUpdatedAt = profile.updatedAt;
            profile.addXP(100, 'test');
            
            expect(profile.updatedAt).not.toBe(originalUpdatedAt);
        });

        test('should update study streak when adding XP', () => {
            profile.addXP(100, 'test');
            
            expect(profile.learning.studyStreak).toBe(1);
            expect(profile.learning.lastStudyDate).toBeDefined();
        });
    });

    describe('Achievement System', () => {
        test('should add achievement correctly', () => {
            const achievement = {
                id: 'test_achievement',
                title: 'Test Achievement',
                description: 'Test description',
                xpReward: 50
            };
            
            const result = profile.addAchievement(achievement);
            
            expect(result).toBeDefined();
            expect(result.id).toBe('test_achievement');
            expect(profile.achievements).toContain(result);
            expect(profile.learning.totalXP).toBe(50); // Should get XP reward
        });

        test('should not add duplicate achievements', () => {
            const achievement = {
                id: 'test_achievement',
                title: 'Test Achievement',
                xpReward: 50
            };
            
            profile.addAchievement(achievement);
            const result = profile.addAchievement(achievement);
            
            expect(result).toBeNull();
            expect(profile.achievements.length).toBe(1);
        });

        test('should award XP for achievements', () => {
            const achievement = {
                id: 'xp_achievement',
                title: 'XP Achievement',
                xpReward: 100
            };
            
            profile.addAchievement(achievement);
            
            expect(profile.learning.totalXP).toBe(100);
        });

        test('should add timestamp to achievements', () => {
            const achievement = {
                id: 'timestamp_test',
                title: 'Timestamp Test'
            };
            
            profile.addAchievement(achievement);
            const added = profile.achievements.find(a => a.id === 'timestamp_test');
            
            expect(added.unlockedAt).toBeDefined();
            expect(new Date(added.unlockedAt).getTime()).toBeCloseTo(new Date().getTime(), -2);
        });
    });

    describe('Course Management', () => {
        test('should add course correctly', () => {
            const course = {
                id: 'course123',
                title: 'Test Course',
                instructor: 'Test Instructor'
            };
            
            const result = profile.addCourse(course);
            
            expect(result.id).toBe('course123');
            expect(profile.learning.courses).toContain(result);
        });

        test('should update existing course', () => {
            const course = {
                id: 'course123',
                title: 'Test Course',
                progress: 50
            };
            
            profile.addCourse(course);
            
            const updated = {
                id: 'course123',
                title: 'Updated Course',
                progress: 75
            };
            
            profile.addCourse(updated);
            
            const found = profile.learning.courses.find(c => c.id === 'course123');
            expect(found.title).toBe('Updated Course');
            expect(found.progress).toBe(75);
        });

        test('should complete course and award XP', () => {
            const course = {
                id: 'course123',
                title: 'Test Course',
                difficulty: 'intermediate',
                duration: 120 // minutes
            };
            
            profile.addCourse(course);
            const result = profile.updateCourseProgress('course123', 100, true);
            
            expect(result.completed).toBe(true);
            expect(result.completedAt).toBeDefined();
            expect(profile.learning.totalXP).toBeGreaterThan(0);
        });

        test('should check course achievements', () => {
            // Add and complete first course
            const course = {
                id: 'course123',
                title: 'First Course',
                difficulty: 'beginner'
            };
            
            profile.addCourse(course);
            profile.updateCourseProgress('course123', 100, true);
            
            // Should have first course achievement
            const firstCourseAchievement = profile.achievements.find(a => a.id === 'first_course');
            expect(firstCourseAchievement).toBeDefined();
        });
    });

    describe('Skill Management', () => {
        test('should add skill correctly', () => {
            const skill = {
                name: 'JavaScript',
                category: 'programming',
                level: 3
            };
            
            const result = profile.addSkill(skill);
            
            expect(result.name).toBe('JavaScript');
            expect(profile.learning.skills).toContain(result);
        });

        test('should update existing skill', () => {
            const skill = {
                name: 'JavaScript',
                level: 3
            };
            
            profile.addSkill(skill);
            
            const updated = {
                name: 'JavaScript',
                level: 5
            };
            
            profile.addSkill(updated);
            
            const found = profile.learning.skills.find(s => s.name === 'JavaScript');
            expect(found.level).toBe(5);
        });
    });

    describe('Level Calculation', () => {
        test('should calculate correct level for XP amounts', () => {
            const testCases = [
                { xp: 0, expectedLevel: 1 },
                { xp: 50, expectedLevel: 1 },
                { xp: 100, expectedLevel: 2 },
                { xp: 200, expectedLevel: 2 },
                { xp: 250, expectedLevel: 3 },
                { xp: 500, expectedLevel: 4 },
                { xp: 1000, expectedLevel: 5 },
                { xp: 2000, expectedLevel: 6 },
                { xp: 5000, expectedLevel: 7 },
                { xp: 10000, expectedLevel: 8 },
                { xp: 15000, expectedLevel: 8 }
            ];
            
            testCases.forEach(({ xp, expectedLevel }) => {
                const testProfile = new StudentProfile('test');
                testProfile.learning.totalXP = xp;
                const level = testProfile.calculateLevel(xp);
                
                expect(level.level).toBe(expectedLevel);
            });
        });
    });

    describe('Study Streak', () => {
        test('should start new streak', () => {
            profile.addXP(50, 'test');
            
            expect(profile.learning.studyStreak).toBe(1);
            expect(profile.learning.lastStudyDate).toBeDefined();
        });

        test('should increment streak for consecutive days', () => {
            // Simulate studying yesterday
            const yesterday = new Date();
            yesterday.setDate(yesterday.getDate() - 1);
            profile.learning.lastStudyDate = yesterday.toISOString();
            profile.learning.studyStreak = 1;
            
            // Study today
            profile.addXP(50, 'test');
            
            expect(profile.learning.studyStreak).toBe(2);
        });

        test('should reset streak for missed days', () => {
            // Simulate studying 2 days ago
            const twoDaysAgo = new Date();
            twoDaysAgo.setDate(twoDaysAgo.getDate() - 2);
            profile.learning.lastStudyDate = twoDaysAgo.toISOString();
            profile.learning.studyStreak = 5;
            
            // Study today
            profile.addXP(50, 'test');
            
            expect(profile.learning.studyStreak).toBe(1);
        });
    });

    describe('Statistics', () => {
        test('should return correct statistics', () => {
            // Add some data
            profile.addXP(500, 'test');
            profile.addAchievement({
                id: 'test_ach',
                title: 'Test',
                xpReward: 50
            });
            profile.addCourse({
                id: 'course1',
                title: 'Course 1',
                completed: true
            });
            profile.addCourse({
                id: 'course2',
                title: 'Course 2',
                completed: false
            });
            
            const stats = profile.getStats();
            
            expect(stats.totalXP).toBe(550);
            expect(stats.level).toBe(4); // Scholar
            expect(stats.coursesCompleted).toBe(1);
            expect(stats.totalCourses).toBe(2);
            expect(stats.achievementsUnlocked).toBe(1);
            expect(stats.studyStreak).toBe(1);
        });
    });

    describe('Level Progress', () => {
        test('should calculate progress correctly', () => {
            profile.learning.totalXP = 750; // Between level 4 (500) and 5 (1000)
            
            const progress = profile.getLevelProgress();
            
            expect(progress.progress).toBeCloseTo(0.5, 1); // 50% progress
            expect(progress.currentXP).toBe(750);
            expect(progress.nextLevelXP).toBe(1000);
            expect(progress.isMaxLevel).toBe(false);
        });

        test('should handle max level', () => {
            profile.learning.totalXP = 15000;
            profile.learning.level = 8;
            
            const progress = profile.getLevelProgress();
            
            expect(progress.isMaxLevel).toBe(true);
            expect(progress.nextLevelXP).toBeNull();
            expect(progress.progress).toBe(1);
        });
    });

    describe('Validation', () => {
        test('should validate correct profile', () => {
            const validation = profile.validate();
            
            expect(validation.isValid).toBe(true);
            expect(validation.errors).toEqual([]);
        });

        test('should detect invalid student ID', () => {
            profile.studentId = '';
            
            const validation = profile.validate();
            
            expect(validation.isValid).toBe(false);
            expect(validation.errors).toContain('Student ID is required and must be a string');
        });

        test('should detect negative XP', () => {
            profile.learning.totalXP = -100;
            
            const validation = profile.validate();
            
            expect(validation.isValid).toBe(false);
            expect(validation.errors).toContain('Total XP cannot be negative');
        });

        test('should detect invalid level', () => {
            profile.learning.level = 9;
            
            const validation = profile.validate();
            
            expect(validation.isValid).toBe(false);
            expect(validation.errors).toContain('Level must be between 1 and 8');
        });
    });

    describe('Serialization', () => {
        test('should convert to JSON correctly', () => {
            profile.addXP(100, 'test');
            profile.addAchievement({
                id: 'test_ach',
                title: 'Test Achievement'
            });
            
            const json = profile.toJSON();
            
            expect(json.studentId).toBe('test123');
            expect(json.learning.totalXP).toBe(100);
            expect(json.achievements).toHaveLength(1);
            expect(json.personalInfo.name).toBe('Test Student');
        });

        test('should create from JSON correctly', () => {
            const originalData = {
                studentId: 'json123',
                name: 'JSON Student',
                totalXP: 250,
                level: 3,
                achievements: [{
                    id: 'json_ach',
                    title: 'JSON Achievement'
                }]
            };
            
            const recreated = StudentProfile.fromJSON(originalData);
            
            expect(recreated.studentId).toBe('json123');
            expect(recreated.learning.totalXP).toBe(250);
            expect(recreated.learning.level).toBe(3);
            expect(recreated.achievements).toHaveLength(1);
        });
    });

    describe('NFT Export', () => {
        test('should export correct NFT metadata', () => {
            profile.addXP(500, 'test');
            profile.addAchievement({
                id: 'export_test',
                title: 'Export Test'
            });
            
            const nftData = profile.exportForNFT();
            
            expect(nftData.studentId).toBe('test123');
            expect(nftData.level).toBe(4);
            expect(nftData.xp).toBe(500);
            expect(nftData.achievements).toBe(1);
            expect(nftData.createdAt).toBeDefined();
        });
    });
});
