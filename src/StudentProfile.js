/**
 * Student Profile Data Structure and Leveling System
 * Manages student data, achievements, and progression
 */

class StudentProfile {
    constructor(studentId, initialData = {}) {
        this.studentId = studentId;
        this.createdAt = initialData.createdAt || new Date().toISOString();
        this.updatedAt = initialData.updatedAt || new Date().toISOString();
        
        // Core profile data
        this.personalInfo = {
            name: initialData.name || '',
            email: initialData.email || '',
            bio: initialData.bio || '',
            avatar: initialData.avatar || '',
            ...initialData.personalInfo
        };
        
        // Learning progress
        this.learning = {
            totalXP: initialData.totalXP || 0,
            level: initialData.level || 1,
            courses: initialData.courses || [],
            certificates: initialData.certificates || [],
            skills: initialData.skills || [],
            studyStreak: initialData.studyStreak || 0,
            lastStudyDate: initialData.lastStudyDate || null
        };
        
        // Achievements and badges
        this.achievements = initialData.achievements || [];
        this.badges = initialData.badges || [];
        
        // Social features
        this.social = {
            following: initialData.following || [],
            followers: initialData.followers || [],
            studyGroups: initialData.studyGroups || [],
            mentorships: initialData.mentorships || []
        };
        
        // Preferences
        this.preferences = {
            learningStyle: initialData.learningStyle || 'visual',
            studyGoals: initialData.studyGoals || [],
            notifications: initialData.notifications || true,
            privacy: initialData.privacy || 'public'
        };
    }

    /**
     * Add XP to student profile and update level
     */
    addXP(amount, source = 'general', metadata = {}) {
        const xpGain = {
            amount: amount,
            source: source,
            timestamp: new Date().toISOString(),
            metadata: metadata
        };
        
        this.learning.totalXP += amount;
        this.learning.level = this.calculateLevel(this.learning.totalXP);
        this.updatedAt = new Date().toISOString();
        
        // Update study streak if studying today
        this.updateStudyStreak();
        
        return {
            previousXP: this.learning.totalXP - amount,
            newXP: this.learning.totalXP,
            levelUp: this.didLevelUp(this.learning.totalXP - amount, this.learning.totalXP),
            xpGain: xpGain
        };
    }

    /**
     * Add achievement to profile
     */
    addAchievement(achievement) {
        const achievementData = {
            id: achievement.id || this.generateId(),
            title: achievement.title,
            description: achievement.description,
            icon: achievement.icon || '🏆',
            category: achievement.category || 'general',
            xpReward: achievement.xpReward || 0,
            requirements: achievement.requirements || [],
            unlockedAt: new Date().toISOString(),
            rarity: achievement.rarity || 'common',
            metadata: achievement.metadata || {}
        };
        
        // Check if achievement already exists
        const exists = this.achievements.find(a => a.id === achievementData.id);
        if (!exists) {
            this.achievements.push(achievementData);
            
            // Add XP reward
            if (achievementData.xpReward > 0) {
                this.addXP(achievementData.xpReward, 'achievement', {
                    achievementId: achievementData.id,
                    title: achievementData.title
                });
            }
            
            this.updatedAt = new Date().toISOString();
            return achievementData;
        }
        
        return null;
    }

    /**
     * Add course to profile
     */
    addCourse(course) {
        const courseData = {
            id: course.id || this.generateId(),
            title: course.title,
            description: course.description,
            instructor: course.instructor,
            category: course.category,
            difficulty: course.difficulty || 'beginner',
            duration: course.duration,
            progress: course.progress || 0,
            completed: course.completed || false,
            completedAt: course.completedAt || null,
            xpEarned: course.xpEarned || 0,
            certificate: course.certificate || null,
            enrolledAt: course.enrolledAt || new Date().toISOString(),
            lastAccessed: course.lastAccessed || new Date().toISOString()
        };
        
        // Check if course already exists
        const existingIndex = this.learning.courses.findIndex(c => c.id === courseData.id);
        if (existingIndex >= 0) {
            this.learning.courses[existingIndex] = { ...this.learning.courses[existingIndex], ...courseData };
        } else {
            this.learning.courses.push(courseData);
        }
        
        this.updatedAt = new Date().toISOString();
        return courseData;
    }

    /**
     * Update course progress
     */
    updateCourseProgress(courseId, progress, completed = false) {
        const course = this.learning.courses.find(c => c.id === courseId);
        if (course) {
            const previousProgress = course.progress;
            course.progress = Math.min(100, Math.max(0, progress));
            course.lastAccessed = new Date().toISOString();
            
            if (completed && !course.completed) {
                course.completed = true;
                course.completedAt = new Date().toISOString();
                
                // Award completion XP
                const completionXP = this.calculateCourseXP(course);
                course.xpEarned = completionXP;
                this.addXP(completionXP, 'course_completion', {
                    courseId: course.id,
                    title: course.title
                });
                
                // Check for course-related achievements
                this.checkCourseAchievements();
            }
            
            this.updatedAt = new Date().toISOString();
            return course;
        }
        
        return null;
    }

    /**
     * Add skill to profile
     */
    addSkill(skill) {
        const skillData = {
            id: skill.id || this.generateId(),
            name: skill.name,
            category: skill.category || 'technical',
            level: skill.level || 1,
            xp: skill.xp || 0,
            endorsements: skill.endorsements || 0,
            acquiredAt: skill.acquiredAt || new Date().toISOString(),
            lastPracticed: skill.lastPracticed || new Date().toISOString()
        };
        
        // Check if skill already exists
        const existingIndex = this.learning.skills.findIndex(s => s.id === skillData.id || s.name === skillData.name);
        if (existingIndex >= 0) {
            this.learning.skills[existingIndex] = { ...this.learning.skills[existingIndex], ...skillData };
        } else {
            this.learning.skills.push(skillData);
        }
        
        this.updatedAt = new Date().toISOString();
        return skillData;
    }

    /**
     * Calculate level based on total XP
     */
    calculateLevel(xp) {
        const levels = [
            { level: 1, requiredXP: 0, name: "Beginner" },
            { level: 2, requiredXP: 100, name: "Novice" },
            { level: 3, requiredXP: 250, name: "Apprentice" },
            { level: 4, requiredXP: 500, name: "Scholar" },
            { level: 5, requiredXP: 1000, name: "Expert" },
            { level: 6, requiredXP: 2000, name: "Master" },
            { level: 7, requiredXP: 5000, name: "Grandmaster" },
            { level: 8, requiredXP: 10000, name: "Legend" }
        ];
        
        for (let i = levels.length - 1; i >= 0; i--) {
            if (xp >= levels[i].requiredXP) {
                return levels[i];
            }
        }
        
        return levels[0];
    }

    /**
     * Check if student leveled up
     */
    didLevelUp(previousXP, newXP) {
        return this.calculateLevel(previousXP).level < this.calculateLevel(newXP).level;
    }

    /**
     * Update study streak
     */
    updateStudyStreak() {
        const today = new Date().toDateString();
        const lastStudy = this.learning.lastStudyDate ? new Date(this.learning.lastStudyDate).toDateString() : null;
        
        if (lastStudy === today) {
            // Already studied today, no change
            return;
        }
        
        const yesterday = new Date();
        yesterday.setDate(yesterday.getDate() - 1);
        
        if (lastStudy === yesterday.toDateString()) {
            // Studied yesterday, increment streak
            this.learning.studyStreak += 1;
        } else {
            // Missed a day, reset streak
            this.learning.studyStreak = 1;
        }
        
        this.learning.lastStudyDate = new Date().toISOString();
        
        // Check for streak achievements
        this.checkStreakAchievements();
    }

    /**
     * Calculate XP earned from course completion
     */
    calculateCourseXP(course) {
        const baseXP = {
            beginner: 50,
            intermediate: 100,
            advanced: 200,
            expert: 300
        };
        
        const difficultyXP = baseXP[course.difficulty] || baseXP.beginner;
        const durationBonus = Math.min(course.duration / 60, 2) * 20; // Max 40 XP bonus for duration
        const difficultyMultiplier = course.difficulty === 'expert' ? 1.5 : course.difficulty === 'advanced' ? 1.2 : 1;
        
        return Math.round((difficultyXP + durationBonus) * difficultyMultiplier);
    }

    /**
     * Check for course-related achievements
     */
    checkCourseAchievements() {
        const completedCourses = this.learning.courses.filter(c => c.completed).length;
        
        // First course completion
        if (completedCourses === 1) {
            this.addAchievement({
                id: 'first_course',
                title: 'Course Beginner',
                description: 'Complete your first course',
                icon: '📚',
                category: 'courses',
                xpReward: 25,
                rarity: 'common'
            });
        }
        
        // Multiple course milestones
        const milestones = [5, 10, 25, 50, 100];
        milestones.forEach(milestone => {
            if (completedCourses === milestone) {
                this.addAchievement({
                    id: `courses_${milestone}`,
                    title: `Course Master ${milestone}`,
                    description: `Complete ${milestone} courses`,
                    icon: '🎓',
                    category: 'courses',
                    xpReward: milestone * 10,
                    rarity: milestone > 25 ? 'epic' : milestone > 10 ? 'rare' : 'common'
                });
            }
        });
    }

    /**
     * Check for streak-related achievements
     */
    checkStreakAchievements() {
        const streak = this.learning.studyStreak;
        
        // Streak milestones
        const milestones = [7, 30, 100, 365];
        milestones.forEach(days => {
            if (streak === days) {
                this.addAchievement({
                    id: `streak_${days}`,
                    title: `${days} Day Streak`,
                    description: `Study for ${days} consecutive days`,
                    icon: '🔥',
                    category: 'streak',
                    xpReward: days * 2,
                    rarity: days >= 100 ? 'legendary' : days >= 30 ? 'epic' : days >= 7 ? 'rare' : 'common'
                });
            }
        });
    }

    /**
     * Get profile statistics
     */
    getStats() {
        return {
            totalXP: this.learning.totalXP,
            level: this.learning.level,
            coursesCompleted: this.learning.courses.filter(c => c.completed).length,
            totalCourses: this.learning.courses.length,
            achievementsUnlocked: this.achievements.length,
            studyStreak: this.learning.studyStreak,
            skillsCount: this.learning.skills.length,
            certificatesCount: this.learning.certificates.length,
            followersCount: this.social.followers.length,
            followingCount: this.social.following.length
        };
    }

    /**
     * Get progress to next level
     */
    getLevelProgress() {
        const currentLevel = this.calculateLevel(this.learning.totalXP);
        if (currentLevel.level >= 8) {
            return { progress: 1, currentXP: this.learning.totalXP, nextLevelXP: null, isMaxLevel: true };
        }
        
        const levels = [
            { level: 1, requiredXP: 0 },
            { level: 2, requiredXP: 100 },
            { level: 3, requiredXP: 250 },
            { level: 4, requiredXP: 500 },
            { level: 5, requiredXP: 1000 },
            { level: 6, requiredXP: 2000 },
            { level: 7, requiredXP: 5000 },
            { level: 8, requiredXP: 10000 }
        ];
        
        const currentLevelXP = currentLevel.requiredXP;
        const nextLevelXP = levels[currentLevel.level].requiredXP;
        
        const progress = (this.learning.totalXP - currentLevelXP) / (nextLevelXP - currentLevelXP);
        
        return {
            progress: Math.min(1, Math.max(0, progress)),
            currentXP: this.learning.totalXP,
            nextLevelXP: nextLevelXP,
            isMaxLevel: false
        };
    }

    /**
     * Generate unique ID
     */
    generateId() {
        return Date.now().toString(36) + Math.random().toString(36).substr(2);
    }

    /**
     * Export profile data for NFT metadata
     */
    exportForNFT() {
        return {
            studentId: this.studentId,
            level: this.learning.level,
            xp: this.learning.totalXP,
            achievements: this.achievements.length,
            courses: this.learning.courses.filter(c => c.completed).length,
            studyStreak: this.learning.studyStreak,
            skills: this.learning.skills.length,
            createdAt: this.createdAt,
            updatedAt: this.updatedAt
        };
    }

    /**
     * Validate profile data
     */
    validate() {
        const errors = [];
        
        if (!this.studentId || typeof this.studentId !== 'string') {
            errors.push('Student ID is required and must be a string');
        }
        
        if (this.learning.totalXP < 0) {
            errors.push('Total XP cannot be negative');
        }
        
        if (this.learning.level < 1 || this.learning.level > 8) {
            errors.push('Level must be between 1 and 8');
        }
        
        if (!Array.isArray(this.achievements)) {
            errors.push('Achievements must be an array');
        }
        
        if (!Array.isArray(this.learning.courses)) {
            errors.push('Courses must be an array');
        }
        
        return {
            isValid: errors.length === 0,
            errors: errors
        };
    }

    /**
     * Convert to JSON
     */
    toJSON() {
        return {
            studentId: this.studentId,
            createdAt: this.createdAt,
            updatedAt: this.updatedAt,
            personalInfo: this.personalInfo,
            learning: this.learning,
            achievements: this.achievements,
            badges: this.badges,
            social: this.social,
            preferences: this.preferences
        };
    }

    /**
     * Create profile from JSON
     */
    static fromJSON(data) {
        return new StudentProfile(data.studentId, data);
    }
}

module.exports = StudentProfile;
