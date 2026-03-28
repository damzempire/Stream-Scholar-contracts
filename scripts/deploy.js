#!/usr/bin/env node

/**
 * Deploy Stream-Scholar NFT Contract to Stellar
 * This script deploys the Student Profile NFT smart contract
 */

const { TransactionBuilder, Networks, Operation, Keypair, Server } = require('@stellar/stellar-sdk');
const StudentProfileNFT = require('../contracts/StudentProfileNFT');

// Configuration
const config = {
    network: Networks.TESTNET,
    horizonUrl: 'https://horizon-testnet.stellar.org',
    // Add your deployer account secret key here
    deployerSecret: process.env.DEPLOYER_SECRET || null
};

async function deployContract() {
    console.log('🚀 Deploying Stream-Scholar NFT Contract...');
    
    if (!config.deployerSecret) {
        console.error('❌ DEPLOYER_SECRET environment variable is required');
        process.exit(1);
    }
    
    try {
        // Initialize contract
        const nftContract = new StudentProfileNFT(config.network, config.horizonUrl);
        
        // Create deployer keypair
        const deployerKeypair = Keypair.fromSecret(config.deployerSecret);
        console.log(`📝 Deployer Account: ${deployerKeypair.publicKey()}`);
        
        // Check account balance
        const server = new Server(config.horizonUrl);
        const account = await server.loadAccount(deployerKeypair.publicKey());
        console.log(`💰 Account Balance: ${account.balances[0].balance} XLM`);
        
        // Deploy the contract
        console.log('🔨 Building deployment transaction...');
        const deploymentResult = await nftContract.deployContract(deployerKeypair);
        
        console.log('✅ Contract deployed successfully!');
        console.log(`📋 Transaction Hash: ${deploymentResult.hash}`);
        
        // Save deployment info
        const deploymentInfo = {
            contractId: deploymentResult.contractId,
            transactionHash: deploymentResult.hash,
            deployerPublicKey: deployerKeypair.publicKey(),
            network: config.network,
            deployedAt: new Date().toISOString()
        };
        
        require('fs').writeFileSync(
            './deployment.json', 
            JSON.stringify(deploymentInfo, null, 2)
        );
        
        console.log('💾 Deployment info saved to deployment.json');
        console.log('🎉 Stream-Scholar NFT Contract is ready!');
        
    } catch (error) {
        console.error('❌ Deployment failed:', error.message);
        if (error.response && error.response.data) {
            console.error('📄 Error details:', JSON.stringify(error.response.data, null, 2));
        }
        process.exit(1);
    }
}

// Run deployment
if (require.main === module) {
    deployContract();
}

module.exports = { deployContract };
