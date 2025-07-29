#!/bin/bash

# RWA Marketplace 多网络部署脚本
echo "🚀 RWA Marketplace 多网络部署脚本"
echo "=================================="

# 检查环境变量
if [ -z "$MNEMONIC" ]; then
    echo "❌ 错误: 请设置MNEMONIC环境变量"
    echo "export MNEMONIC=your_twelve_word_mnemonic_phrase_here"
    exit 1
fi

if [ -z "$alchemy_PROJECT_ID" ]; then
    echo "⚠️ 警告: 未设置alchemy_PROJECT_ID，Sepolia部署可能失败"
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

# 选择部署网络
echo ""
echo "🌐 请选择部署网络:"
echo "1) Sepolia测试网"
echo "2) BSC测试网"
echo "3) 两个网络都部署"
echo "4) 退出"
echo ""
read -p "请输入选择 (1-4): " choice

case $choice in
    1)
        echo "🚀 部署到Sepolia测试网..."
        npx hardhat run contracts/rwa-marketplace/deploy/deploy-sepolia.js --network sepolia
        ;;
    2)
        echo "🚀 部署到BSC测试网..."
        npx hardhat run contracts/rwa-marketplace/deploy/deploy-bsc.js --network bscTestnet
        ;;
    3)
        echo "🚀 部署到Sepolia测试网..."
        npx hardhat run contracts/rwa-marketplace/deploy/deploy-sepolia.js --network sepolia
        
        if [ $? -eq 0 ]; then
            echo "✅ Sepolia部署成功"
            echo ""
            echo "🚀 部署到BSC测试网..."
            npx hardhat run contracts/rwa-marketplace/deploy/deploy-bsc.js --network bscTestnet
        else
            echo "❌ Sepolia部署失败，跳过BSC部署"
            exit 1
        fi
        ;;
    4)
        echo "👋 退出部署"
        exit 0
        ;;
    *)
        echo "❌ 无效选择"
        exit 1
        ;;
esac

if [ $? -eq 0 ]; then
    echo ""
    echo "🎉 部署完成！"
    echo "📋 部署信息已保存到 contracts/rwa-marketplace/deploy/ 目录"
    echo ""
    echo "📊 部署摘要:"
    if [ -f "contracts/rwa-marketplace/deploy/deployment-sepolia.json" ]; then
        echo "✅ Sepolia部署信息: contracts/rwa-marketplace/deploy/deployment-sepolia.json"
    fi
    if [ -f "contracts/rwa-marketplace/deploy/deployment-bsc.json" ]; then
        echo "✅ BSC部署信息: contracts/rwa-marketplace/deploy/deployment-bsc.json"
    fi
    echo ""
    echo "🧪 运行部署测试..."
    if [ -f "contracts/rwa-marketplace/deploy/deployment-sepolia.json" ]; then
        echo "测试Sepolia部署..."
        npx hardhat run contracts/rwa-marketplace/deploy/test-deployment.js --network sepolia
    fi
else
    echo "❌ 部署失败"
    exit 1
fi 