#!/bin/bash

# Deploy Contract Manager Program
# This script deploys the contract manager program to the specified network

set -e

# Configuration
NETWORK=${1:-devnet}
PROGRAM_ID="6LmWMjAMAfVdc8mpgPjHvFLa2sbcudiLiJT3bAGRYMMD"

echo "🚀 Deploying Contract Manager Program to $NETWORK..."

# Check if anchor is installed
if ! command -v anchor &> /dev/null; then
    echo "❌ Anchor CLI not found. Please install Anchor first."
    exit 1
fi

# Check if solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo "❌ Solana CLI not found. Please install Solana CLI first."
    exit 1
fi

# Set the network
echo "📡 Setting network to $NETWORK..."
solana config set --url $NETWORK

# Check wallet
echo "💰 Checking wallet..."
WALLET_ADDRESS=$(solana address)
if [ -z "$WALLET_ADDRESS" ]; then
    echo "❌ No wallet found. Please create or import a wallet first."
    exit 1
fi
echo "✅ Wallet address: $WALLET_ADDRESS"

# Check balance
BALANCE=$(solana balance)
echo "💰 Current balance: $BALANCE SOL"

# Build the program
echo "🔨 Building contract manager program..."
anchor build --program-name mobazha_contract_manager

# Deploy the program
echo "🚀 Deploying contract manager program..."
anchor deploy --program-name mobazha_contract_manager

# Verify deployment
echo "✅ Verifying deployment..."
DEPLOYED_PROGRAM_ID=$(solana program show $PROGRAM_ID --output json | jq -r '.programId')
if [ "$DEPLOYED_PROGRAM_ID" = "$PROGRAM_ID" ]; then
    echo "✅ Contract Manager Program deployed successfully!"
    echo "📋 Program ID: $PROGRAM_ID"
    echo "🌐 Network: $NETWORK"
    echo "👤 Deployed by: $WALLET_ADDRESS"
else
    echo "❌ Deployment verification failed"
    exit 1
fi

# Initialize the contract manager
echo "🔧 Initializing contract manager..."
anchor run initialize-contract-manager

echo "🎉 Contract Manager setup complete!"
echo ""
echo "📝 Next steps:"
echo "1. Add versions using: anchor run add-version"
echo "2. Mark recommended versions using: anchor run mark-recommended"
echo "3. Use the Go client to fetch contract addresses dynamically"
