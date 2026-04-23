#!/usr/bin/env node

/**
 * Deployment script for Student Profile NFT contracts
 * Deploys both the Soroban smart contract and sets up the JavaScript integration
 */

const { Server, Networks, TransactionBuilder, Keypair, Contract } = require('@stellar/stellar-sdk');
const fs = require('fs');
const path = require('path');

// Configuration
const CONFIG = {
    testnet: {
        network: Networks.TESTNET,
        horizon: 'https://horizon-testnet.stellar.org',
        rpc: 'https://soroban-testnet.stellar.org',
    },
    standalone: {
        network: Networks.STANDALONE,
        horizon: 'http://localhost:8000',
        rpc: 'http://localhost:8000/soroban/rpc',
    }
};

class NFTDeployer {
    constructor(network = 'testnet') {
        this.network = CONFIG[network];
        this.server = new Server(this.network.horizon);
        this.networkPassphrase = this.network.network;
    }

    /**
     * Deploy the Student Profile NFT contract
     */
    async deployContract(deployerKeypair) {
        try {
            console.log('🚀 Deploying Student Profile NFT Contract...');
            
            // Load the compiled WASM file
            const wasmPath = path.join(__dirname, '../contracts/scholar_contracts/target/wasm32-unknown-unknown/release/scholar_contracts.wasm');
            
            if (!fs.existsSync(wasmPath)) {
                throw new Error(`WASM file not found at ${wasmPath}. Please build the contract first.`);
            }

            const wasm = fs.readFileSync(wasmPath);
            
            // Get deployer account
            const account = await this.server.loadAccount(deployerKeypair.publicKey());
            
            // Create contract deployment transaction
            const contract = new Contract();
            
            const transaction = new TransactionBuilder(account, {
                fee: '10000',
                networkPassphrase: this.networkPassphrase
            })
            .addOperation(contract.deploy({
                wasm: wasm
            }))
            .setTimeout(30)
            .build();

            // Sign and submit transaction
            transaction.sign(deployerKeypair);
            const result = await this.server.submitTransaction(transaction);
            
            if (!result.successful) {
                throw new Error(`Contract deployment failed: ${result.resultXdr}`);
            }

            // Extract contract ID from result
            const contractId = this.extractContractId(result);
            
            console.log(`✅ Contract deployed successfully!`);
            console.log(`📄 Contract ID: ${contractId}`);
            console.log(`🔗 Transaction Hash: ${result.hash}`);

            // Initialize the contract
            await this.initializeContract(contractId, deployerKeypair);

            return {
                contractId,
                transactionHash: result.hash,
                network: this.networkPassphrase
            };
        } catch (error) {
            console.error('❌ Deployment failed:', error.message);
            throw error;
        }
    }

    /**
     * Initialize the deployed contract
     */
    async initializeContract(contractId, deployerKeypair) {
        try {
            console.log('🔧 Initializing contract...');
            
            const account = await this.server.loadAccount(deployerKeypair.publicKey());
            const contract = new Contract(contractId);
            
            const transaction = new TransactionBuilder(account, {
                fee: '10000',
                networkPassphrase: this.networkPassphrase
            })
            .addOperation(contract.call(
                'init'
            ))
            .setTimeout(30)
            .build();

            transaction.sign(deployerKeypair);
            const result = await this.server.submitTransaction(transaction);
            
            if (!result.successful) {
                throw new Error(`Contract initialization failed: ${result.resultXdr}`);
            }

            console.log('✅ Contract initialized successfully!');
            return result;
        } catch (error) {
            console.error('❌ Contract initialization failed:', error.message);
            throw error;
        }
    }

    /**
     * Extract contract ID from deployment transaction result
     */
    extractContractId(transactionResult) {
        // This is a simplified extraction - in practice you'd parse the XDR
        // For now, return a placeholder that should be replaced with actual parsing
        const operations = transactionResult.operations || [];
        for (const op of operations) {
            if (op.type === 'createContract') {
                return op.contractId || 'CONTRACT_ID_PLACEHOLDER';
            }
        }
        throw new Error('Could not extract contract ID from transaction result');
    }

    /**
     * Mint a test NFT
     */
    async mintTestNFT(contractId, studentId, studentKeypair) {
        try {
            console.log(`🎨 Minting test NFT for student: ${studentId}`);
            
            const account = await this.server.loadAccount(studentKeypair.publicKey());
            const contract = new Contract(contractId);
            
            // Prepare initial metadata
            const metadata = {
                name: `Test Student: ${studentId}`,
                description: 'Test NFT for Stream-Scholar platform',
                image: 'https://via.placeholder.com/400',
                attributes: [
                    { trait_type: "Test", value: true }
                ]
            };
            
            const transaction = new TransactionBuilder(account, {
                fee: '10000',
                networkPassphrase: this.networkPassphrase
            })
            .addOperation(contract.call(
                'mint_nft',
                studentKeypair.publicKey(),
                studentId,
                metadata
            ))
            .setTimeout(30)
            .build();

            transaction.sign(studentKeypair);
            const result = await this.server.submitTransaction(transaction);
            
            if (!result.successful) {
                throw new Error(`NFT minting failed: ${result.resultXdr}`);
            }

            console.log('✅ Test NFT minted successfully!');
            console.log(`🔗 Transaction Hash: ${result.hash}`);
            
            return result;
        } catch (error) {
            console.error('❌ Test NFT minting failed:', error.message);
            throw error;
        }
    }

    /**
     * Save deployment configuration
     */
    saveDeploymentConfig(config, filename = 'deployment.json') {
        const configPath = path.join(__dirname, '../config', filename);
        
        // Ensure config directory exists
        const configDir = path.dirname(configPath);
        if (!fs.existsSync(configDir)) {
            fs.mkdirSync(configDir, { recursive: true });
        }
        
        fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
        console.log(`💾 Deployment configuration saved to: ${configPath}`);
    }

    /**
     * Generate environment file
     */
    generateEnvFile(contractId, filename = '.env.nft') {
        const envContent = `
# Student Profile NFT Configuration
NFT_CONTRACT_ID=${contractId}
NETWORK=${this.networkPassphrase}
HORIZON_URL=${this.network.horizon}
RPC_URL=${this.network.rpc}

# Deployment timestamp
DEPLOYED_AT=${new Date().toISOString()}
        `.trim();

        const envPath = path.join(__dirname, '../', filename);
        fs.writeFileSync(envPath, envContent);
        console.log(`📝 Environment file generated: ${envPath}`);
    }
}

// CLI interface
async function main() {
    const args = process.argv.slice(2);
    const network = args.includes('--standalone') ? 'standalone' : 'testnet';
    
    console.log(`🌐 Deploying to ${network} network`);
    
    const deployer = new NFTDeployer(network);
    
    try {
        // Get deployer keypair from environment or generate
        let deployerKeypair;
        const deployerSecret = process.env.DEPLOYER_SECRET;
        
        if (deployerSecret) {
            deployerKeypair = Keypair.fromSecret(deployerSecret);
        } else {
            console.log('⚠️  No DEPLOYER_SECRET found, generating new keypair...');
            deployerKeypair = Keypair.random();
            console.log(`🔑 New deployer public key: ${deployerKeypair.publicKey()}`);
            console.log(`🔐 New deployer secret: ${deployerKeypair.secret()}`);
            console.log('⚠️  Save this secret key safely and fund the account on the network!');
        }

        // Deploy contract
        const deployment = await deployer.deployContract(deployerKeypair);
        
        // Save configuration
        deployer.saveDeploymentConfig({
            ...deployment,
            network,
            deployerPublicKey: deployerKeypair.publicKey()
        });
        
        deployer.generateEnvFile(deployment.contractId);
        
        // Mint test NFT if requested
        if (args.includes('--mint-test')) {
            const testStudentId = 'test_student_001';
            const testKeypair = Keypair.random();
            
            console.log(`🎓 Minting test NFT for ${testStudentId}`);
            console.log(`🔑 Test student public key: ${testKeypair.publicKey()}`);
            
            await deployer.mintTestNFT(
                deployment.contractId,
                testStudentId,
                testKeypair
            );
        }
        
        console.log('🎉 Deployment completed successfully!');
        console.log('\n📋 Summary:');
        console.log(`   Contract ID: ${deployment.contractId}`);
        console.log(`   Network: ${network}`);
        console.log(`   Transaction: ${deployment.transactionHash}`);
        
    } catch (error) {
        console.error('💥 Deployment failed:', error.message);
        process.exit(1);
    }
}

// Export for use in other modules
module.exports = NFTDeployer;

// Run if called directly
if (require.main === module) {
    main().catch(console.error);
}
