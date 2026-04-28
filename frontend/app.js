/**
 * Stream-Scholar NFT Frontend Application
 * Handles NFT minting, display, and management
 */

class StreamScholarNFT {
    constructor() {
        this.stellar = window.StellarSdk;
        this.server = new this.stellar.Server('https://horizon-testnet.stellar.org');
        this.network = this.stellar.Networks.TESTNET;
        this.connectedAccount = null;
        this.studentProfile = null;
        this.nftContract = null;
        this.currentDonationCourseId = null;
        
        this.initializeEventListeners();
        this.loadSavedProfile();
    }

    /**
     * Initialize all event listeners
     */
    initializeEventListeners() {
        // Wallet connection
        document.getElementById('connectWallet').addEventListener('click', () => this.connectWallet());
        
        // Form submission
        document.getElementById('mintForm').addEventListener('submit', (e) => this.handleMintForm(e));
        
        // Profile actions
        document.getElementById('updateXP').addEventListener('click', () => this.showXPModal());
        document.getElementById('addAchievement').addEventListener('click', () => this.showAchievementModal());
        document.getElementById('transferNFT').addEventListener('click', () => this.showTransferModal());
        
        // Modal controls
        document.getElementById('cancelModal').addEventListener('click', () => this.closeModal());
        document.getElementById('actionForm').addEventListener('submit', (e) => this.handleActionForm(e));
        
        // Form preview updates
        document.getElementById('studentId').addEventListener('input', (e) => this.updatePreview());
        document.getElementById('displayName').addEventListener('input', (e) => this.updatePreview());
        
        // Close modal on background click
        document.getElementById('actionModal').addEventListener('click', (e) => {
            if (e.target.id === 'actionModal') {
                this.closeModal();
            }
        });
    }

    /**
     * Connect to Stellar wallet
     */
    async connectWallet() {
        try {
            // For demo purposes, create a random keypair
            // In production, integrate with actual wallet providers like Freighter, Albedo, etc.
            const keypair = this.stellar.Keypair.random();
            this.connectedAccount = {
                publicKey: keypair.publicKey(),
                secretKey: keypair.secret(),
                keypair: keypair
            };
            
            // Fund the account on testnet
            await this.fundTestnetAccount(keypair.publicKey());
            
            // Update UI
            document.getElementById('walletAddress').textContent = `Connected: ${keypair.publicKey().substring(0, 8)}...`;
            document.getElementById('walletAddress').classList.remove('hidden');
            document.getElementById('connectWallet').textContent = 'Wallet Connected';
            document.getElementById('connectWallet').disabled = true;
            
            this.showToast('Wallet connected successfully!', 'success');
            
            // Load existing NFTs for this account
            await this.loadAccountNFTs();
            
        } catch (error) {
            console.error('Error connecting wallet:', error);
            this.showToast('Failed to connect wallet: ' + error.message, 'error');
        }
    }

    /**
     * Fund testnet account
     */
    async fundTestnetAccount(publicKey) {
        try {
            const response = await fetch(`https://friendbot.stellar.org?addr=${publicKey}`);
            const result = await response.json();
            
            if (result.success) {
                console.log('Account funded successfully');
            } else {
                throw new Error('Friendbot funding failed');
            }
        } catch (error) {
            console.error('Error funding account:', error);
            // Account might already exist, continue
        }
    }

    /**
     * Handle mint form submission
     */
    async handleMintForm(event) {
        event.preventDefault();
        
        if (!this.connectedAccount) {
            this.showToast('Please connect your wallet first', 'error');
            return;
        }
        
        const formData = {
            studentId: document.getElementById('studentId').value,
            displayName: document.getElementById('displayName').value,
            email: document.getElementById('email').value,
            bio: document.getElementById('bio').value,
            learningStyle: document.getElementById('learningStyle').value
        };
        
        try {
            this.showToast('Minting NFT...', 'info');
            
            // Create student profile
            this.studentProfile = new StudentProfile(formData.studentId, {
                name: formData.displayName,
                email: formData.email,
                bio: formData.bio,
                learningStyle: formData.learningStyle
            });
            
            // Mint NFT
            const nftResult = await this.mintNFT(formData.studentId, this.studentProfile.exportForNFT());
            
            // Save profile locally
            this.saveProfile();
            
            // Show profile section
            this.showProfileSection();
            
            this.showToast('NFT minted successfully!', 'success');
            
        } catch (error) {
            console.error('Error minting NFT:', error);
            this.showToast('Failed to mint NFT: ' + error.message, 'error');
        }
    }

    /**
     * Mint NFT on Stellar
     */
    async mintNFT(studentId, metadata) {
        try {
            const account = await this.server.loadAccount(this.connectedAccount.publicKey);
            const tokenId = this.generateTokenId();
            
            // Create unique asset for the NFT
            const nftAsset = new this.stellar.Asset(`STUDENT_${tokenId}`, this.connectedAccount.publicKey);
            
            const transaction = new this.stellar.TransactionBuilder(account, {
                fee: this.stellar.BASE_FEE,
                networkPassphrase: this.network
            })
            .addOperation(this.stellar.Operation.changeTrust({
                asset: nftAsset,
                limit: '1'
            }))
            .addOperation(this.stellar.Operation.payment({
                destination: this.connectedAccount.publicKey,
                asset: nftAsset,
                amount: '1'
            }))
            .addOperation(this.stellar.Operation.manageData({
                name: `NFT_${tokenId}`,
                value: JSON.stringify({
                    owner: this.connectedAccount.publicKey,
                    studentId: studentId,
                    ...metadata,
                    tokenId: tokenId,
                    createdAt: new Date().toISOString()
                })
            }))
            .setTimeout(30)
            .build();
            
            transaction.sign(this.connectedAccount.keypair);
            
            const result = await this.server.submitTransaction(transaction);
            
            // Store NFT info
            this.studentProfile.nftTokenId = tokenId;
            this.studentProfile.nftAsset = nftAsset;
            
            return { tokenId, transaction: result };
            
        } catch (error) {
            console.error('Error minting NFT:', error);
            throw error;
        }
    }

    /**
     * Show profile section with NFT display
     */
    showProfileSection() {
        document.getElementById('mintSection').classList.add('hidden');
        document.getElementById('profileSection').classList.remove('hidden');
        document.getElementById('marketSection').classList.remove('hidden');
        
        this.updateNFTCard();
        this.updateLevelProgress();
        this.updateAchievementsList();
        this.updateCoursesList();
        this.updateActivityLog();
    }

    /**
     * Update NFT card display
     */
    updateNFTCard() {
        if (!this.studentProfile) return;
        
        const nftCard = document.getElementById('nftCard');
        const levelInfo = this.getLevelInfo(this.studentProfile.learning.level);
        const progress = this.getLevelProgress();
        
        nftCard.innerHTML = `
            <div class="text-center mb-4">
                <div class="w-24 h-24 bg-gradient-to-br from-${levelInfo.color} to-purple-600 rounded-full mx-auto mb-4 flex items-center justify-center">
                    <span class="text-3xl">🎓</span>
                </div>
                <h4 class="text-xl font-bold">${this.studentProfile.personalInfo.name}</h4>
                <p class="text-sm opacity-75">${this.studentProfile.studentId}</p>
            </div>
            
            <div class="mb-4">
                <div class="flex justify-between items-center mb-2">
                    <span class="text-sm font-semibold">Level ${this.studentProfile.learning.level} - ${levelInfo.name}</span>
                    <span class="text-sm">${this.studentProfile.learning.totalXP} XP</span>
                </div>
                <div class="w-full bg-gray-700 rounded-full h-3">
                    <div class="progress-bar bg-gradient-to-r from-purple-500 to-pink-500 h-3 rounded-full" 
                         style="width: ${progress.progress * 100}%"></div>
                </div>
                <p class="text-xs mt-1 opacity-75">${Math.round(progress.progress * 100)}% to next level</p>
            </div>
            
            <div class="grid grid-cols-2 gap-3 text-sm">
                <div class="bg-gray-800 rounded-lg p-3 text-center">
                    <div class="text-2xl mb-1">📚</div>
                    <div class="font-semibold">${this.studentProfile.learning.courses.filter(c => c.completed).length} Courses</div>
                </div>
                <div class="bg-gray-800 rounded-lg p-3 text-center">
                    <div class="text-2xl mb-1">🏆</div>
                    <div class="font-semibold">${this.studentProfile.achievements.length} Achievements</div>
                </div>
                <div class="bg-gray-800 rounded-lg p-3 text-center">
                    <div class="text-2xl mb-1">🔥</div>
                    <div class="font-semibold">${this.studentProfile.learning.studyStreak} Day Streak</div>
                </div>
                <div class="bg-gray-800 rounded-lg p-3 text-center">
                    <div class="text-2xl mb-1">⚡</div>
                    <div class="font-semibold">${this.studentProfile.learning.skills.length} Skills</div>
                </div>
            </div>
            
            <div class="mt-4 pt-4 border-t border-gray-700">
                <p class="text-xs opacity-75">Token ID: ${this.studentProfile.nftTokenId}</p>
                <p class="text-xs opacity-75">Created: ${new Date(this.studentProfile.createdAt).toLocaleDateString()}</p>
            </div>
        `;
    }

    /**
     * Update level progress section
     */
    updateLevelProgress() {
        if (!this.studentProfile) return;
        
        const levelProgress = document.getElementById('levelProgress');
        const currentLevel = this.getLevelInfo(this.studentProfile.learning.level);
        const progress = this.getLevelProgress();
        
        levelProgress.innerHTML = `
            <div class="flex items-center justify-between mb-4">
                <div>
                    <h4 class="text-lg font-semibold">Level ${this.studentProfile.learning.level} - ${currentLevel.name}</h4>
                    <p class="text-sm opacity-75">${this.studentProfile.learning.totalXP} Total XP</p>
                </div>
                <div class="text-3xl">${currentLevel.icon || '🎯'}</div>
            </div>
            
            <div class="mb-4">
                <div class="flex justify-between text-sm mb-2">
                    <span>Progress to Level ${this.studentProfile.learning.level + 1}</span>
                    <span>${Math.round(progress.progress * 100)}%</span>
                </div>
                <div class="w-full bg-gray-600 rounded-full h-4">
                    <div class="progress-bar bg-gradient-to-r from-green-500 to-blue-500 h-4 rounded-full" 
                         style="width: ${progress.progress * 100}%"></div>
                </div>
                <div class="flex justify-between text-xs mt-1 opacity-75">
                    <span>${progress.currentXP} XP</span>
                    <span>${progress.nextLevelXP} XP</span>
                </div>
            </div>
            
            ${!progress.isMaxLevel ? `
                <div class="bg-gray-800 rounded-lg p-3">
                    <p class="text-sm">Next Level: ${this.getLevelInfo(this.studentProfile.learning.level + 1).name}</p>
                    <p class="text-xs opacity-75">Need ${progress.nextLevelXP - progress.currentXP} more XP</p>
                </div>
            ` : `
                <div class="bg-gradient-to-r from-yellow-600 to-orange-600 rounded-lg p-3">
                    <p class="text-sm font-semibold">🏆 Maximum Level Reached!</p>
                    <p class="text-xs opacity-75">You are a Stream-Scholar Legend</p>
                </div>
            `}
        `;
    }

    /**
     * Update achievements list
     */
    updateAchievementsList() {
        if (!this.studentProfile) return;
        
        const achievementsList = document.getElementById('achievementsList');
        const recentAchievements = this.studentProfile.achievements.slice(-6).reverse();
        
        if (recentAchievements.length === 0) {
            achievementsList.innerHTML = '<p class="text-gray-400 col-span-full text-center">No achievements yet. Start learning to unlock your first achievement!</p>';
            return;
        }
        
        achievementsList.innerHTML = recentAchievements.map(achievement => `
            <div class="achievement-badge bg-gray-800 rounded-lg p-4 text-center cursor-pointer hover:bg-gray-700">
                <div class="text-2xl mb-2">${achievement.icon}</div>
                <h5 class="font-semibold text-sm">${achievement.title}</h5>
                <p class="text-xs opacity-75 mt-1">${achievement.description}</p>
                <div class="mt-2">
                    <span class="text-xs px-2 py-1 bg-purple-600 rounded-full">${achievement.xpReward} XP</span>
                </div>
            </div>
        `).join('');
    }

    /**
     * Update courses list
     */
    updateCoursesList() {
        if (!this.studentProfile) return;
        
        const coursesList = document.getElementById('coursesList');
        const courses = this.studentProfile.learning.courses.slice(-3).reverse();
        
        if (courses.length === 0) {
            coursesList.innerHTML = '<p class="text-gray-400 text-center">No courses enrolled yet.</p>';
            return;
        }
        
        coursesList.innerHTML = courses.map(course => `
            <div class="bg-gray-800 rounded-lg p-4">
                <div class="flex justify-between items-start">
                    <div>
                        <h5 class="font-semibold">${course.title}</h5>
                        <p class="text-sm opacity-75">${course.instructor || 'Unknown Instructor'}</p>
                        ${course.isFree ? '<span class="text-xs px-2 py-1 bg-green-600 rounded-full">Free</span>' : ''}
                    </div>
                    <div class="text-right">
                        <span class="text-xs px-2 py-1 bg-${course.completed ? 'green' : 'blue'}-600 rounded-full">
                            ${course.completed ? 'Completed' : 'In Progress'}
                        </span>
                    </div>
                </div>
                ${!course.completed ? `
                    <div class="mt-3">
                        <div class="flex justify-between text-xs mb-1">
                            <span>Progress</span>
                            <span>${course.progress}%</span>
                        </div>
                        <div class="w-full bg-gray-700 rounded-full h-2">
                            <div class="bg-blue-500 h-2 rounded-full" style="width: ${course.progress}%"></div>
                        </div>
                    </div>
                ` : `
                    <div class="mt-2">
                        <p class="text-xs text-green-400">✓ Completed on ${new Date(course.completedAt).toLocaleDateString()}</p>
                    </div>
                `}
                ${course.isFree && course.acceptDonations ? `
                    <div class="mt-3">
                        <button onclick="streamScholarNFT.showDonationModal(${course.courseId}, '${course.instructor || 'Instructor'}')" 
                                class="w-full bg-gradient-to-r from-purple-500 to-pink-500 text-white px-3 py-2 rounded-lg text-sm font-medium hover:from-purple-600 hover:to-pink-600 transition-colors">
                            💝 Tip Instructor
                        </button>
                    </div>
                ` : ''}
            </div>
        `).join('');
    }

    /**
     * Update activity log
     */
    updateActivityLog() {
        if (!this.studentProfile) return;
        
        const activityLog = document.getElementById('activityLog');
        const activities = this.generateRecentActivity();
        
        activityLog.innerHTML = activities.map(activity => `
            <div class="flex items-center space-x-3 bg-gray-800 rounded-lg p-3">
                <div class="text-2xl">${activity.icon}</div>
                <div class="flex-1">
                    <p class="text-sm">${activity.description}</p>
                    <p class="text-xs opacity-75">${activity.timestamp}</p>
                </div>
                ${activity.xp ? `<span class="text-xs px-2 py-1 bg-green-600 rounded-full">+${activity.xp} XP</span>` : ''}
            </div>
        `).join('');
    }

    /**
     * Generate recent activity
     */
    generateRecentActivity() {
        const activities = [];
        
        // Add recent achievements
        this.studentProfile.achievements.slice(-3).forEach(achievement => {
            activities.push({
                icon: achievement.icon,
                description: `Unlocked achievement: ${achievement.title}`,
                timestamp: new Date(achievement.unlockedAt).toLocaleString(),
                xp: achievement.xpReward
            });
        });
        
        // Add recent course completions
        this.studentProfile.learning.courses
            .filter(c => c.completed)
            .slice(-2)
            .forEach(course => {
                activities.push({
                    icon: '📚',
                    description: `Completed course: ${course.title}`,
                    timestamp: new Date(course.completedAt).toLocaleString(),
                    xp: course.xpEarned
                });
            });
        
        // Add study streak activity
        if (this.studentProfile.learning.studyStreak > 0) {
            activities.push({
                icon: '🔥',
                description: `${this.studentProfile.learning.studyStreak} day study streak!`,
                timestamp: new Date(this.studentProfile.learning.lastStudyDate).toLocaleString(),
                xp: null
            });
        }
        
        return activities.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp)).slice(0, 5);
    }

    /**
     * Show XP modal
     */
    showXPModal() {
        document.getElementById('modalTitle').textContent = 'Add Experience Points';
        document.getElementById('modalLabel').textContent = 'XP Amount';
        document.getElementById('modalInput').type = 'number';
        document.getElementById('modalInput').placeholder = 'Enter XP amount';
        document.getElementById('modalInput').min = '1';
        document.getElementById('modalInput').max = '1000';
        document.getElementById('actionModal').classList.remove('hidden');
        this.currentAction = 'addXP';
    }

    /**
     * Show achievement modal
     */
    showAchievementModal() {
        document.getElementById('modalTitle').textContent = 'Add Achievement';
        document.getElementById('modalLabel').textContent = 'Achievement Name';
        document.getElementById('modalInput').type = 'text';
        document.getElementById('modalInput').placeholder = 'Enter achievement name';
        document.getElementById('actionModal').classList.remove('hidden');
        this.currentAction = 'addAchievement';
    }

    /**
     * Show transfer modal
     */
    showTransferModal() {
        document.getElementById('modalTitle').textContent = 'Transfer NFT';
        document.getElementById('modalLabel').textContent = 'Recipient Address';
        document.getElementById('modalInput').type = 'text';
        document.getElementById('modalInput').placeholder = 'Enter Stellar public key';
        document.getElementById('actionModal').classList.remove('hidden');
        this.currentAction = 'transferNFT';
    }

    /**
     * Close modal
     */
    closeModal() {
        document.getElementById('actionModal').classList.add('hidden');
        document.getElementById('actionForm').reset();
        this.currentAction = null;
    }

    /**
     * Handle action form submission
     */
    async handleActionForm(event) {
        event.preventDefault();
        
        const value = document.getElementById('modalInput').value;
        
        try {
            switch (this.currentAction) {
                case 'addXP':
                    await this.addXP(parseInt(value));
                    break;
                case 'addAchievement':
                    await this.addCustomAchievement(value);
                    break;
                case 'transferNFT':
                    await this.transferNFT(value);
                    break;
            }
            
            this.closeModal();
            
        } catch (error) {
            console.error('Error handling action:', error);
            this.showToast('Error: ' + error.message, 'error');
        }
    }

    /**
     * Add XP to student profile
     */
    async addXP(amount) {
        if (!amount || amount <= 0) {
            throw new Error('Please enter a valid XP amount');
        }
        
        const result = this.studentProfile.addXP(amount, 'manual');
        
        // Add to activity log
        this.studentProfile.recentActivity = this.studentProfile.recentActivity || [];
        this.studentProfile.recentActivity.push({
            type: 'xp_gain',
            amount: amount,
            timestamp: new Date().toISOString()
        });
        
        // Update blockchain
        await this.updateXPOnChain(amount);
        
        // Save and update UI
        this.saveProfile();
        this.updateNFTCard();
        this.updateLevelProgress();
        this.updateActivityLog();
        
        if (result.levelUp) {
            this.showToast(`🎉 Level Up! You're now level ${this.studentProfile.learning.level}!`, 'success');
            this.animateLevelUp();
        } else {
            this.showToast(`Added ${amount} XP!`, 'success');
        }
    }

    /**
     * Add custom achievement
     */
    async addCustomAchievement(name) {
        const achievement = {
            id: this.generateId(),
            title: name,
            description: 'Custom achievement',
            icon: '🏆',
            category: 'custom',
            xpReward: 50,
            rarity: 'common'
        };
        
        this.studentProfile.addAchievement(achievement);
        
        // Update blockchain
        await this.addAchievementOnChain(achievement);
        
        // Save and update UI
        this.saveProfile();
        this.updateNFTCard();
        this.updateAchievementsList();
        this.updateActivityLog();
        
        this.showToast(`Achievement "${name}" added!`, 'success');
    }

    /**
     * Transfer NFT to another address
     */
    async transferNFT(recipientAddress) {
        if (!this.stellar.StrKey.isValidEd25519PublicKey(recipientAddress)) {
            throw new Error('Invalid Stellar address');
        }
        
        // Update blockchain ownership
        await this.transferNFTOnChain(recipientAddress);
        
        // Clear local profile
        this.studentProfile = null;
        localStorage.removeItem('streamScholarProfile');
        
        // Reset UI
        document.getElementById('profileSection').classList.add('hidden');
        document.getElementById('mintSection').classList.remove('hidden');
        
        this.showToast('NFT transferred successfully!', 'success');
    }

    /**
     * Update XP on blockchain
     */
    async updateXPOnChain(amount) {
        try {
            const account = await this.server.loadAccount(this.connectedAccount.publicKey);
            
            const transaction = new this.stellar.TransactionBuilder(account, {
                fee: this.stellar.BASE_FEE,
                networkPassphrase: this.network
            })
            .addOperation(this.stellar.Operation.manageData({
                name: `NFT_${this.studentProfile.nftTokenId}`,
                value: JSON.stringify({
                    owner: this.connectedAccount.publicKey,
                    studentId: this.studentProfile.studentId,
                    ...this.studentProfile.exportForNFT(),
                    updatedAt: new Date().toISOString()
                })
            }))
            .setTimeout(30)
            .build();
            
            transaction.sign(this.connectedAccount.keypair);
            await this.server.submitTransaction(transaction);
            
        } catch (error) {
            console.error('Error updating XP on chain:', error);
            // Continue anyway for demo purposes
        }
    }

    /**
     * Add achievement on blockchain
     */
    async addAchievementOnChain(achievement) {
        // Similar implementation to updateXPOnChain
        // For demo purposes, we'll skip the blockchain update
    }

    /**
     * Transfer NFT on blockchain
     */
    async transferNFTOnChain(recipientAddress) {
        try {
            const account = await this.server.loadAccount(this.connectedAccount.publicKey);
            const nftAsset = new this.stellar.Asset(`STUDENT_${this.studentProfile.nftTokenId}`, this.connectedAccount.publicKey);
            
            const transaction = new this.stellar.TransactionBuilder(account, {
                fee: this.stellar.BASE_FEE,
                networkPassphrase: this.network
            })
            .addOperation(this.stellar.Operation.payment({
                destination: recipientAddress,
                asset: nftAsset,
                amount: '1'
            }))
            .setTimeout(30)
            .build();
            
            transaction.sign(this.connectedAccount.keypair);
            await this.server.submitTransaction(transaction);
            
        } catch (error) {
            console.error('Error transferring NFT:', error);
            throw error;
        }
    }

    /**
     * Animate level up
     */
    animateLevelUp() {
        const nftCard = document.getElementById('nftCard');
        nftCard.classList.add('level-up-animation');
        
        setTimeout(() => {
            nftCard.classList.remove('level-up-animation');
        }, 600);
    }

    /**
     * Update preview in mint form
     */
    updatePreview() {
        const studentId = document.getElementById('studentId').value;
        const displayName = document.getElementById('displayName').value;
        
        document.getElementById('previewName').textContent = displayName || 'Your Name';
        document.getElementById('previewId').textContent = studentId || 'Student ID';
    }

    /**
     * Load account NFTs
     */
    async loadAccountNFTs() {
        try {
            const account = await this.server.loadAccount(this.connectedAccount.publicKey);
            const nftDataEntries = account.data.filter(entry => entry.name.startsWith('NFT_'));
            
            if (nftDataEntries.length > 0) {
                // Load existing NFT
                const nftData = JSON.parse(nftDataEntries[0].value);
                this.studentProfile = StudentProfile.fromJSON(nftData);
                this.showProfileSection();
                this.showToast('Existing NFT profile loaded!', 'info');
            }
        } catch (error) {
            console.error('Error loading NFTs:', error);
        }
    }

    /**
     * Save profile to localStorage
     */
    saveProfile() {
        if (this.studentProfile) {
            localStorage.setItem('streamScholarProfile', JSON.stringify(this.studentProfile.toJSON()));
        }
    }

    /**
     * Load saved profile
     */
    loadSavedProfile() {
        const saved = localStorage.getItem('streamScholarProfile');
        if (saved) {
            try {
                this.studentProfile = StudentProfile.fromJSON(JSON.parse(saved));
                this.showProfileSection();
            } catch (error) {
                console.error('Error loading saved profile:', error);
            }
        }
    }

    /**
     * Get level information
     */
    getLevelInfo(level) {
        const levels = {
            1: { name: "Beginner", requiredXP: 0, color: "gray", icon: "🌱" },
            2: { name: "Novice", requiredXP: 100, color: "silver", icon: "📖" },
            3: { name: "Apprentice", requiredXP: 250, color: "bronze", icon: "⚒️" },
            4: { name: "Scholar", requiredXP: 500, color: "gold", icon: "🎓" },
            5: { name: "Expert", requiredXP: 1000, color: "emerald", icon: "💎" },
            6: { name: "Master", requiredXP: 2000, color: "blue", icon: "👑" },
            7: { name: "Grandmaster", requiredXP: 5000, color: "purple", icon: "🔮" },
            8: { name: "Legend", requiredXP: 10000, color: "orange", icon: "🏆" }
        };
        
        return levels[level] || levels[1];
    }

    /**
     * Get level progress
     */
    getLevelProgress() {
        const currentLevel = this.studentProfile.learning.level;
        if (currentLevel >= 8) {
            return { progress: 1, currentXP: this.studentProfile.learning.totalXP, nextLevelXP: null, isMaxLevel: true };
        }
        
        const levels = [0, 100, 250, 500, 1000, 2000, 5000, 10000];
        const currentLevelXP = levels[currentLevel - 1];
        const nextLevelXP = levels[currentLevel];
        
        const progress = (this.studentProfile.learning.totalXP - currentLevelXP) / (nextLevelXP - currentLevelXP);
        
        return {
            progress: Math.min(1, Math.max(0, progress)),
            currentXP: this.studentProfile.learning.totalXP,
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
     * Generate unique token ID
     */
    generateTokenId() {
        return 'SP_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
    }

    /**
     * Show toast notification
     */
    showToast(message, type = 'info') {
        const toast = document.createElement('div');
        toast.className = `px-4 py-3 rounded-lg shadow-lg text-white ${
            type === 'success' ? 'bg-green-600' :
            type === 'error' ? 'bg-red-600' :
            type === 'warning' ? 'bg-yellow-600' :
            'bg-blue-600'
        }`;
        toast.textContent = message;
        
        const container = document.getElementById('toastContainer');
        container.appendChild(toast);
        
        setTimeout(() => {
            toast.style.opacity = '0';
            toast.style.transition = 'opacity 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, 3000);
    }
}

// Include StudentProfile class (simplified version for frontend)
class StudentProfile {
    constructor(studentId, data = {}) {
        this.studentId = studentId;
        this.createdAt = data.createdAt || new Date().toISOString();
        this.updatedAt = data.updatedAt || new Date().toISOString();
        
        this.personalInfo = {
            name: data.name || '',
            email: data.email || '',
            bio: data.bio || '',
            learningStyle: data.learningStyle || 'visual',
            ...data.personalInfo
        };
        
        this.learning = {
            totalXP: data.totalXP || 0,
            level: data.level || 1,
            courses: data.courses || [
                {
                    courseId: 1,
                    title: "Introduction to Blockchain",
                    instructor: "Dr. Sarah Chen",
                    progress: 75,
                    completed: false,
                    isFree: true,
                    acceptDonations: true,
                    enrolledAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString()
                },
                {
                    courseId: 2,
                    title: "Advanced Stellar Development",
                    instructor: "Prof. Michael Roberts",
                    progress: 100,
                    completed: true,
                    completedAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
                    isFree: false,
                    acceptDonations: false,
                    xpEarned: 150,
                    enrolledAt: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString()
                },
                {
                    courseId: 3,
                    title: "Smart Contract Security",
                    instructor: "Dr. Emily Johnson",
                    progress: 30,
                    completed: false,
                    isFree: true,
                    acceptDonations: true,
                    enrolledAt: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000).toISOString()
                }
            ],
            certificates: data.certificates || [],
            skills: data.skills || [],
            studyStreak: data.studyStreak || 0,
            lastStudyDate: data.lastStudyDate || null
        };
        
        this.achievements = data.achievements || [];
        this.badges = data.badges || [];
        this.recentActivity = data.recentActivity || [];
    }

    addXP(amount, source = 'general') {
        const previousXP = this.learning.totalXP;
        this.learning.totalXP += amount;
        this.learning.level = this.calculateLevel(this.learning.totalXP);
        this.updatedAt = new Date().toISOString();
        
        return {
            previousXP: previousXP,
            newXP: this.learning.totalXP,
            levelUp: this.calculateLevel(previousXP).level < this.learning.level,
            xpGain: amount
        };
    }

    addAchievement(achievement) {
        const exists = this.achievements.find(a => a.id === achievement.id);
        if (!exists) {
            this.achievements.push(achievement);
            
            if (achievement.xpReward > 0) {
                this.addXP(achievement.xpReward, 'achievement');
            }
            
            this.updatedAt = new Date().toISOString();
            return achievement;
        }
        
        return null;
    }

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

    toJSON() {
        return {
            studentId: this.studentId,
            createdAt: this.createdAt,
            updatedAt: this.updatedAt,
            personalInfo: this.personalInfo,
            learning: this.learning,
            achievements: this.achievements,
            badges: this.badges,
            recentActivity: this.recentActivity
        };
    }

    // Donation Stream Functions

    showDonationModal(courseId, instructorName) {
        if (!this.connectedAccount) {
            this.showToast('Please connect your wallet first!', 'error');
            return;
        }

        this.currentDonationCourseId = courseId;
        document.getElementById('instructorName').textContent = `Instructor: ${instructorName}`;
        document.getElementById('donationAmount').value = '';
        document.getElementById('donationModal').classList.remove('hidden');
    }

    closeDonationModal() {
        document.getElementById('donationModal').classList.add('hidden');
        this.currentDonationCourseId = null;
    }

    async processDonation() {
        try {
            const amount = parseFloat(document.getElementById('donationAmount').value);
            
            if (!amount || amount <= 0) {
                this.showToast('Please enter a valid donation amount', 'error');
                return;
            }

            if (!this.connectedAccount) {
                this.showToast('Wallet not connected', 'error');
                return;
            }

            // Convert amount to stroops (1 XLM = 10,000,000 stroops)
            const amountStroops = Math.floor(amount * 10000000);

            // For demo purposes, simulate the donation transaction
            // In production, this would call the smart contract's donate_to_instructor function
            console.log(`Processing donation of ${amount} XLM to course ${this.currentDonationCourseId}`);
            
            // Simulate transaction
            await this.simulateDonationTransaction(amountStroops);

            this.showToast(`Successfully sent ${amount} XLM tip to instructor!`, 'success');
            this.closeDonationModal();

            // Update the course display to reflect the donation
            this.updateCoursesList();

        } catch (error) {
            console.error('Error processing donation:', error);
            this.showToast('Failed to process donation: ' + error.message, 'error');
        }
    }

    async simulateDonationTransaction(amountStroops) {
        // Simulate blockchain transaction delay
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // In production, this would:
        // 1. Create a Stellar transaction
        // 2. Call the donate_to_instructor function on the smart contract
        // 3. Submit the transaction to the network
        // 4. Wait for confirmation
        
        console.log(`Donation transaction simulated: ${amountStroops} stroops`);
        return true;
    }

    static fromJSON(data) {
        return new StudentProfile(data.studentId, data);
    }
}

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
    window.streamScholarNFT = new StreamScholarNFT();
});
