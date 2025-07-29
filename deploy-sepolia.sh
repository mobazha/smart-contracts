#!/bin/bash

# RWA Marketplace Sepolia部署脚本
echo "🚀 RWA Marketplace Sepolia部署脚本"
echo "=================================="

# 检查环境变量
if [ -z "$PRIVATE_KEY" ]; then
    echo "❌ 错误: 请设置PRIVATE_KEY环境变量"
    echo "export PRIVATE_KEY=your_private_key_here"
    exit 1
fi

if [ -z "$SEPOLIA_RPC_URL" ]; then
    echo "❌ 错误: 请设置SEPOLIA_RPC_URL环境变量"
    echo "export SEPOLIA_RPC_URL=https://sepolia.infura.io/v3/YOUR_PROJECT_ID"
    exit 1
fi

if [ -z "$ETHERSCAN_API_KEY" ]; then
    echo "⚠️ 警告: 未设置ETHERSCAN_API_KEY，合约验证可能失败"
fi

# 检查依赖
echo "📦 检查依赖..."
if ! command -v node &> /dev/null; then
    echo "❌ 错误: 未安装Node.js"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo "❌ 错误: 未安装npm"
    exit 1
fi

# 安装依赖
echo "📦 安装依赖..."
npm install

# 编译合约
echo "🔨 编译合约..."
npx hardhat compile

# 检查编译结果
if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi

echo "✅ 编译成功"

# 运行测试
echo "🧪 运行测试..."
npx hardhat test

if [ $? -ne 0 ]; then
    echo "❌ 测试失败"
    exit 1
fi

echo "✅ 测试通过"

# 部署到Sepolia
echo "🚀 部署到Sepolia测试网..."
npx hardhat run contracts/rwa-marketplace/deploy/deploy-sepolia.js --network sepolia

if [ $? -ne 0 ]; then
    echo "❌ 部署失败"
    exit 1
fi

echo "✅ 部署完成！"
echo "📋 部署信息已保存到 contracts/rwa-marketplace/deploy/deployment-sepolia.json" 