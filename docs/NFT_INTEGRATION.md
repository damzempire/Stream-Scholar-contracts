# Student Profile NFT Integration

This document describes the complete Student Profile NFT integration for the Stream-Scholar platform, which creates dynamic NFTs that evolve with student learning achievements.

## Overview

The Student Profile NFT system transforms a student's learning journey into a unique, tradable non-fungible token on the Stellar blockchain. Each NFT represents a student's profile that dynamically evolves based on their educational achievements.

## Features

### 🎓 Dynamic Leveling System
- **8 Progressive Levels**: Beginner → Novice → Apprentice → Scholar → Expert → Master → Grandmaster → Legend
- **XP-Based Progression**: Students earn XP through course completion, achievements, and study streaks
- **Visual Evolution**: NFT artwork changes based on level and achievements

### 🏆 Achievement System
- **Multiple Categories**: Course completion, study streaks, milestones, special events
- **Rarity Tiers**: Common, Rare, Epic, Legendary achievements
- **XP Rewards**: Each achievement grants bonus XP toward level progression

### 📚 Course Integration
- **Automatic Tracking**: Course completion automatically updates NFT
- **Difficulty-Based Rewards**: Higher difficulty courses grant more XP
- **Progress Visualization**: Real-time progress tracking for enrolled courses

### 🔥 Study Streaks
- **Daily Engagement Tracking**: Rewards consistent learning habits
- **Bonus Rewards**: Streak milestones grant additional XP and achievements
- **Motivation System**: Visual indicators encourage continued engagement

### 🎨 Visual NFT Generation
- **Dynamic SVG Artwork**: NFT images change based on level and achievements
- **Level-Based Themes**: Each level has unique color schemes and visual elements
- **Progress Indicators**: Visual representation of XP progress and achievements

## Architecture

### Smart Contract Layer
```
┌─────────────────────────────────────────┐
│        Soroban Smart Contract          │
├─────────────────────────────────────────┤
│ • NFT Minting                      │
│ • XP Management                     │
│ • Achievement Tracking               │
│ • Level Calculation                 │
│ • Transfer Logic                    │
└─────────────────────────────────────────┘
```

### Integration Layer
```
┌─────────────────────────────────────────┐
│    StudentProfileNFTIntegration       │
├─────────────────────────────────────────┤
│ • Profile Management                │
│ • Blockchain Sync                  │
│ • Event Handling                   │
│ • Metadata Generation               │
└─────────────────────────────────────────┘
```

### Frontend Interface
```
┌─────────────────────────────────────────┐
│         Web Interface                │
├─────────────────────────────────────────┤
│ • Wallet Connection                │
│ • Profile Display                  │
│ • NFT Minting                    │
│ • Achievement Gallery             │
└─────────────────────────────────────────┘
```

## Installation

### Prerequisites
- Node.js 16+
- Rust 1.70+ (for contract compilation)
- Stellar account (testnet for development)

### Setup
```bash
# Clone repository
git clone https://github.com/damzempire/Stream-Scholar-contracts.git
cd Stream-Scholar-contracts

# Install dependencies
npm install

# Build contracts
cd contracts/scholar_contracts
cargo build --target wasm32-unknown-unknown --release

# Return to root
cd ../..
```

### Environment Configuration
```bash
# Copy environment template
cp .env.example .env

# Edit with your Stellar credentials
# TESTNET_SECRET_KEY=your_testnet_secret_key
# HORIZON_URL=https://horizon-testnet.stellar.org
# NETWORK_PASSPHRASE=Test SDF Network ; September 2015
```

## Usage

### Deploy NFT Contract
```bash
# Deploy to testnet
npm run deploy:nft

# Deploy with demo NFT minting
npm run deploy:nft -- --mint-test

# Deploy to standalone network
npm run deploy:nft -- --standalone
```

### Mint Student Profile NFT
```bash
# Create demo profiles
npm run mint:demo

# Mint individual profile
npm run mint:enhanced mint alice123 SECRET_KEY

# Interactive minting
node scripts/mint-nft-enhanced.js
```

### View Profiles
```bash
# Get profile information
npm run profile alice123

# Sync profile with blockchain
node scripts/mint-nft-enhanced.js sync alice123 SECRET_KEY
```

### Frontend Development
```bash
# Start development server
npm run dev

# Build for production
npm run build

# View enhanced frontend
open frontend/index-enhanced.html
```

## API Reference

### StudentProfileNFTIntegration

#### Constructor
```javascript
new StudentProfileNFTIntegration(networkPassphrase, horizonUrl)
```

#### Methods

##### createStudentProfile(studentData, issuerKeypair)
Creates a new student profile and mints corresponding NFT.

**Parameters:**
- `studentData`: Object containing student information
- `issuerKeypair`: Stellar keypair for signing transactions

**Returns:** Promise resolving to `{ profile, nft }`

##### addStudentXP(studentId, xpAmount, source, metadata, signerKeypair)
Adds XP to student profile and updates NFT.

**Parameters:**
- `studentId`: Unique student identifier
- `xpAmount`: Amount of XP to add
- `source`: Source of XP (course, achievement, etc.)
- `metadata`: Additional context data
- `signerKeypair`: Authorized signer keypair

**Returns:** Promise resolving to XP update result

##### completeCourse(studentId, courseData, signerKeypair)
Marks a course as completed and awards XP.

**Parameters:**
- `studentId`: Student identifier
- `courseData`: Course completion information
- `signerKeypair`: Authorized signer

**Returns:** Promise resolving to course completion result

##### addAchievement(studentId, achievement, signerKeypair)
Adds achievement to student profile and NFT.

**Parameters:**
- `studentId`: Student identifier
- `achievement`: Achievement object with metadata
- `signerKeypair`: Authorized signer

**Returns:** Promise resolving to achievement result

### StudentProfile Class

#### Constructor
```javascript
new StudentProfile(studentId, initialData)
```

#### Methods

##### addXP(amount, source, metadata)
Adds XP to student profile and handles level progression.

##### addAchievement(achievement)
Adds achievement and awards associated XP rewards.

##### addCourse(course)
Adds course to student's learning history.

##### updateCourseProgress(courseId, progress, completed)
Updates course progress and handles completion rewards.

##### getStats()
Returns comprehensive student statistics.

##### getLevelProgress()
Calculates progress to next level.

## Level System

| Level | Name | Required XP | Visual Theme | Rarity |
|-------|------|-------------|--------------|---------|
| 1 | Beginner | 0 | Gray 🌱 | Common |
| 2 | Novice | 100 | Silver 📖 | Common |
| 3 | Apprentice | 250 | Bronze ⚒️ | Uncommon |
| 4 | Scholar | 500 | Gold 🎓 | Rare |
| 5 | Expert | 1,000 | Emerald 💎 | Rare |
| 6 | Master | 2,000 | Blue 👑 | Epic |
| 7 | Grandmaster | 5,000 | Purple 🔮 | Epic |
| 8 | Legend | 10,000 | Orange 🏆 | Legendary |

## Achievement Categories

### 📚 Course Achievements
- First course completion
- Course milestones (5, 10, 25, 50, 100 courses)
- Subject mastery achievements

### 🔥 Streak Achievements
- 7, 30, 100, 365 day streaks
- Perfect attendance
- Consistent learning patterns

### 🎯 Level Achievements
- Level progression milestones
- Level-specific challenges
- Mastery demonstrations

### ⭐ Special Achievements
- Early adopter bonuses
- Community contributions
- Exceptional performance

## Testing

### Run Tests
```bash
# Run all tests
npm test

# Run NFT integration tests
npm run test:nft

# Run tests with coverage
npm run test -- --coverage
```

### Test Structure
```
tests/
├── nft-integration.test.js     # Main integration tests
├── student-profile.test.js      # Profile class tests
└── nft-contract.test.js        # Contract interaction tests
```

### Test Categories
- **Profile Creation**: NFT minting and initialization
- **XP Management**: XP addition and level progression
- **Course Integration**: Course completion and rewards
- **Achievement System**: Achievement unlocking and rewards
- **Blockchain Sync**: Profile synchronization
- **Transfer Logic**: NFT ownership transfers
- **Error Handling**: Invalid inputs and network issues

## Frontend Features

### 🎨 NFT Display
- Dynamic SVG generation based on profile data
- Level-based visual themes
- Achievement badges display
- Progress indicators

### 👛 Wallet Integration
- Stellar wallet connection (Freighter, Albedo)
- Transaction signing
- Balance display
- Network switching

### 📊 Profile Management
- Profile creation and editing
- Achievement gallery
- Course history
- Statistics dashboard

### 🔗 Social Features
- Profile sharing
- NFT transfer interface
- Achievement showcase
- Leaderboard integration

## Security Considerations

### 🔐 Smart Contract Security
- Access control for minting and transfers
- Input validation for all parameters
- Reentrancy protection
- Overflow/underflow prevention

### 🛡️ Frontend Security
- Secure keypair handling
- Transaction validation
- XSS protection
- CSRF prevention

### 🔑 Key Management
- Never expose private keys in frontend
- Use secure wallet integrations
- Implement proper session management
- Validate all user inputs

## Performance Optimization

### ⚡ Contract Optimization
- Efficient storage patterns
- Minimal gas usage
- Batch operations where possible
- Lazy loading for large datasets

### 🚀 Frontend Performance
- Component lazy loading
- Image optimization
- Caching strategies
- Bundle size optimization

## Deployment

### 🌐 Networks
- **Testnet**: Development and testing
- **Standalone**: Local development
- **Mainnet**: Production deployment

### 📦 Build Process
```bash
# Build contracts
cargo build --target wasm32-unknown-unknown --release

# Build frontend
npm run build

# Deploy contracts
npm run deploy:nft

# Verify deployment
soroban contract info --id <CONTRACT_ID>
```

## Monitoring

### 📊 Analytics
- NFT minting statistics
- Level progression tracking
- Achievement completion rates
- User engagement metrics

### 🔍 Logging
- Transaction logging
- Error tracking
- Performance monitoring
- User behavior analysis

## Contributing

### 🤝 Development Workflow
1. Fork repository
2. Create feature branch
3. Implement changes
4. Add tests
5. Submit pull request

### 📝 Code Standards
- ESLint for JavaScript
- Clippy for Rust
- Comprehensive test coverage
- Documentation updates

## Support

### 📚 Documentation
- [API Reference](./docs/api.md)
- [Contract Guide](./docs/contracts.md)
- [Frontend Guide](./docs/frontend.md)

### 🐛 Issue Reporting
- [GitHub Issues](https://github.com/damzempire/Stream-Scholar-contracts/issues)
- [Discord Community](https://discord.gg/stream-scholar)

### 💬 Community
- [Discord](https://discord.gg/stream-scholar)
- [Twitter](https://twitter.com/streamscholar)
- [Blog](https://blog.stream-scholar.com)

## License

MIT License - see [LICENSE](./LICENSE) file for details.

---

**Built with ❤️ by the Stream-Scholar Team**

*Transforming education into verifiable digital achievements on the Stellar blockchain.*
