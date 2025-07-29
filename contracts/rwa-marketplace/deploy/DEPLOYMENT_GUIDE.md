# RWA Marketplace 部署指南

## 前置要求

### 1. 安装依赖
```bash
npm install
# 或者
yarn install
```

### 2. 环境配置

创建 `.env` 文件并配置以下环境变量：

```env
# 助记词（用于生成账户）
MNEMONIC=your_twelve_word_mnemonic_phrase_here

# Alchemy项目ID（用于Sepolia）
alchemy_PROJECT_ID=your_alchemy_project_id_here

# API密钥（用于合约验证）
ETHERSCAN_API_KEY=your_etherscan_api_key_here
BSCSCAN_API_KEY=your_bscscan_api_key_here
POLYGONSCAN_API_KEY=your_polygonscan_api_key_here

# Gas报告
REPORT_GAS=true
```

### 3. 获取测试币

#### Sepolia测试币
- [Sepolia Faucet](https://sepoliafaucet.com/)
- [Infura Sepolia Faucet](https://www.infura.io/faucet/sepolia)
- [Alchemy Sepolia Faucet](https://sepoliafaucet.com/)

#### BSC测试币
- [BSC Faucet](https://testnet.binance.org/faucet-smart)

## 部署方式

### 方式一：使用Hardhat（推荐）

#### 1. 编译合约
```bash
npm run compile:hardhat
# 或
npx hardhat compile
```

#### 2. 运行测试
```bash
npm run test:hardhat
# 或
npx hardhat test
```

#### 3. 部署到不同网络

**Sepolia测试网**
```bash
npm run deploy:rwa-sepolia
# 或
npx hardhat run contracts/rwa-marketplace/deploy/deploy-sepolia.js --network sepolia
```

**BSC测试网**
```bash
npm run deploy:rwa-bsc
# 或
npx hardhat run contracts/rwa-marketplace/deploy/deploy-bsc.js --network bscTestnet
```

#### 4. 测试部署
```bash
npm run test:rwa-deployment
# 或
npx hardhat run contracts/rwa-marketplace/deploy/test-deployment.js --network sepolia
```

### 方式二：使用Truffle（兼容原有配置）

#### 1. 编译合约
```bash
npm run compile
# 或
truffle compile
```

#### 2. 运行测试
```bash
npm run test
# 或
truffle test
```

#### 3. 部署到不同网络

**BSC测试网**
```bash
npm run migrate:bsc
# 或
truffle migrate --network bscTestnet
```

**Polygon主网**
```bash
npm run migrate:polygon
# 或
truffle migrate --network polygonMainnet
```

## 部署的合约

### 1. RWA Marketplace合约
- **功能**: RWA Token交易市场
- **特性**: 
  - 创建订单并付款
  - 发货完成交易
  - 取消订单
  - 平台费用管理

### 2. 森林碳汇信用代币合约
- **名称**: Forest Carbon Credit Token
- **符号**: FCC
- **总供应量**: 500,000 tokens
- **特性**:
  - KYC验证
  - 碳汇量追踪
  - 环保认证

### 3. 示例USDT代币合约
- **名称**: Mock USDT
- **符号**: USDT
- **总供应量**: 1,000,000 USDT
- **小数位**: 6位

## 网络配置

### 支持的网络

| 网络 | 网络ID | RPC URL | 状态 |
|------|--------|---------|------|
| Sepolia | 11155111 | Alchemy | ✅ 支持 |
| BSC测试网 | 97 | Binance | ✅ 支持 |
| BSC主网 | 56 | Binance | ✅ 支持 |
| Polygon测试网 | 80002 | Polygon | ✅ 支持 |
| Polygon主网 | 137 | Polygon | ✅ 支持 |
| Conflux测试网 | 71 | Conflux | ✅ 支持 |
| Conflux主网 | 1030 | Conflux | ✅ 支持 |

### 环境变量配置

```env
# 基础配置
MNEMONIC=your_twelve_word_mnemonic_phrase_here

# 网络配置
alchemy_PROJECT_ID=your_alchemy_project_id_here

# API密钥
ETHERSCAN_API_KEY=your_etherscan_api_key_here
BSCSCAN_API_KEY=your_bscscan_api_key_here
POLYGONSCAN_API_KEY=your_polygonscan_api_key_here
```

## 部署后配置

### 1. 检查部署信息
部署完成后，会在 `deploy/` 目录下生成部署信息文件：
- `deployment-sepolia.json` - Sepolia部署信息
- `deployment-bsc.json` - BSC部署信息

### 2. 更新前端配置
将部署的合约地址更新到前端配置文件中：

```javascript
// frontend/src/config/rwaMarketplace.js
export const RWA_MARKETPLACE_CONFIG = {
  networks: {
    ethereum: {
      testnet: {
        chainId: 11155111, // Sepolia
        name: 'Ethereum Sepolia Testnet',
        rpcUrl: 'https://eth-sepolia.g.alchemy.com/v2/YOUR_PROJECT_ID',
        explorer: 'https://sepolia.etherscan.io',
        marketplaceContract: 'DEPLOYED_CONTRACT_ADDRESS',
        rwaTokenContract: 'DEPLOYED_RWA_TOKEN_ADDRESS',
        paymentTokens: {
          ETH: '0x0000000000000000000000000000000000000000',
          USDT: 'DEPLOYED_MOCK_USDT_ADDRESS'
        }
      }
    },
    bsc: {
      testnet: {
        chainId: 97, // BSC测试网
        name: 'BSC Testnet',
        rpcUrl: 'https://data-seed-prebsc-1-s1.binance.org:8545/',
        explorer: 'https://testnet.bscscan.com',
        marketplaceContract: 'DEPLOYED_CONTRACT_ADDRESS',
        rwaTokenContract: 'DEPLOYED_RWA_TOKEN_ADDRESS',
        paymentTokens: {
          BNB: '0x0000000000000000000000000000000000000000',
          USDT: 'DEPLOYED_MOCK_USDT_ADDRESS'
        }
      }
    }
  }
};
```

## 故障排除

### 1. 常见错误

#### 余额不足
```
Error: insufficient funds for gas * price + value
```
**解决方案**: 获取更多测试币

#### 网络连接问题
```
Error: could not detect network
```
**解决方案**: 检查RPC URL配置

#### 合约验证失败
```
Error: Already Verified
```
**解决方案**: 这是正常情况，合约已经验证过了

### 2. Gas费用优化

如果Gas费用过高，可以调整配置：

```javascript
// hardhat.config.js
networks: {
  sepolia: {
    url: `https://eth-sepolia.g.alchemy.com/v2/${process.env.alchemy_PROJECT_ID}`,
    accounts: process.env.MNEMONIC ? [process.env.MNEMONIC] : [],
    gasPrice: 15000000000, // 15 gwei
    gas: 3000000, // 3M gas limit
  }
}
```

### 3. 合约验证问题

如果自动验证失败，可以手动验证：

```bash
# 验证RWA Marketplace
npx hardhat verify --network sepolia MARKETPLACE_ADDRESS

# 验证RWA Token
npx hardhat verify --network sepolia RWA_TOKEN_ADDRESS "Forest Carbon Credit Token" "FCC" "500000000000000000000000" DEPLOYER_ADDRESS

# 验证Mock USDT
npx hardhat verify --network sepolia MOCK_USDT_ADDRESS "Mock USDT" "USDT" "1000000000000"
```

## 安全注意事项

1. **助记词安全**: 永远不要将助记词提交到代码仓库
2. **测试网络**: 确保只在测试网络上部署测试合约
3. **合约验证**: 部署后立即验证合约代码
4. **权限管理**: 确保只有授权账户可以调用管理函数

## 监控和维护

### 1. 监控合约事件
```javascript
// 监听订单创建事件
marketplace.on("OrderCreated", (orderId, buyer, seller, rwaTokenAmount, paymentAmount) => {
  console.log("新订单创建:", { orderId, buyer, seller, rwaTokenAmount, paymentAmount });
});
```

### 2. 检查合约状态
```javascript
// 检查平台费用
const platformFee = await marketplace.platformFee();
console.log("平台费用:", platformFee.toString());

// 检查订单数量
const orderCounter = await marketplace.orderCounter();
console.log("订单总数:", orderCounter.toString());
```

### 3. 紧急操作
```javascript
// 暂停合约
await marketplace.pause();

// 恢复合约
await marketplace.unpause();

// 提取资金
await marketplace.withdrawFunds(recipientAddress, amount, tokenAddress);
```

## 下一步

部署完成后，您可以：

1. **集成前端**: 更新前端配置使用新部署的合约
2. **测试功能**: 进行完整的端到端测试
3. **用户培训**: 准备用户文档和培训材料
4. **监控部署**: 设置监控和告警系统 