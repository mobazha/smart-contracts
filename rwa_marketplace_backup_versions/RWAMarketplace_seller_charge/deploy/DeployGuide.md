# Mobazha RWA Marketplace 部署指南

## 概述

本指南将帮助你部署和测试 Mobazha RWA Marketplace 合约。该合约支持多卖家多Token多币种的RWA分销，包含平台手续费、白名单管理、KYC校验、预言机集成等功能。

## 前置要求

1. **开发环境**
   - Node.js 16+
   - npm 或 yarn
   - Hardhat 或 Foundry

2. **依赖包**
   ```bash
   npm install @openzeppelin/contracts
   npm install --save-dev @nomiclabs/hardhat-ethers ethers
   ```

3. **网络配置**
   - 测试网：BSC Testnet
   - 主网：BSC Mainnet
   - 本地测试：Hardhat Network

## 部署步骤

### 1. 准备预言机合约

首先需要部署或配置预言机合约。推荐使用 Chainlink 预言机：

```solidity
// 示例：Chainlink 价格预言机
address CHAINLINK_PRICE_FEED = 0x...; // 根据具体Token配置
```

### 2. 准备多签钱包

建议使用 Gnosis Safe 作为平台管理地址：

1. 访问 [Gnosis Safe](https://gnosis-safe.io/)
2. 创建多签钱包（建议 2-of-3 或 3-of-5）
3. 记录多签钱包地址

### 3. 部署 Marketplace 合约

#### 使用 Hardhat 部署

```javascript
// scripts/deploy.js
const { ethers } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  
  // 部署参数
  const priceOracle = "0x..."; // 预言机地址
  const platform = "0x...";    // 多签钱包地址
  
  // 部署 Marketplace
  const RwaMarketplace = await ethers.getContractFactory("RwaMarketplace");
  const marketplace = await RwaMarketplace.deploy(priceOracle, platform);
  await marketplace.deployed();
  
  console.log("RwaMarketplace deployed to:", marketplace.address);
  
  // 注册到 ContractManager
  const contractManager = await ethers.getContractAt("ContractManager", "0x...");
  await contractManager.setContract("RwaMarketplace", marketplace.address);
  
  console.log("Marketplace registered to ContractManager");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
```

#### 使用 Foundry 部署

```bash
# 编译合约
forge build

# 部署合约
forge script script/Deploy.s.sol --rpc-url $BSC_RPC_URL --private-key $PRIVATE_KEY --broadcast
```

### 4. 初始化设置

部署完成后，需要进行以下初始化设置：

#### 设置白名单

```javascript
// 设置卖家白名单
await marketplace.setSellerWhitelist(sellerAddress, true);

// 设置RWA Token白名单
await marketplace.setRwaTokenWhitelist(rwaTokenAddress, true);

// 设置支付币种白名单
await marketplace.setPayTokenWhitelist(payTokenAddress, true);
```

#### 设置KYC状态

```javascript
// 设置用户KYC状态
await marketplace.setKycStatus(userAddress, true);
```

#### 配置手续费

```javascript
// 设置手续费比例（基点，如30表示0.3%）
await marketplace.setFeeBps(30);
```

## 测试步骤

### 1. 运行单元测试

```bash
# 使用 Hardhat
npx hardhat test

# 使用 Foundry
forge test
```

### 2. 测试用例覆盖

测试文件 `RwaMarketplaceTest.sol` 包含以下测试用例：

- ✅ 合约部署测试
- ✅ 卖家上架Token测试
- ✅ 卖家充值Token测试
- ✅ 买家购买Token测试
- ✅ 卖家提现测试
- ✅ 撤回未售Token测试
- ✅ KYC开关测试
- ✅ 合约暂停测试
- ✅ 手续费调整测试
- ✅ 权限控制测试
- ✅ 白名单验证测试

### 3. 集成测试

```bash
# 运行集成测试
npx hardhat test test/integration/
```

## 验证步骤

### 1. 合约验证

部署到测试网后，在 BSCScan 上验证合约：

```bash
npx hardhat verify --network bscTestnet DEPLOYED_CONTRACT_ADDRESS "PRICE_ORACLE_ADDRESS" "PLATFORM_ADDRESS"
```

### 2. 功能验证

1. **卖家操作验证**
   - 上架RWA Token
   - 充值Token到合约
   - 设置支持的支付币种

2. **买家操作验证**
   - 查看可购买的Token列表
   - 执行购买操作
   - 验证Token到账

3. **平台管理验证**
   - 调整手续费
   - 管理白名单
   - 设置KYC状态
   - 暂停/恢复合约

## 安全注意事项

### 1. 多签钱包安全

- 确保多签钱包的私钥安全存储
- 定期更换多签成员
- 设置合理的签名阈值

### 2. 预言机安全

- 使用可靠的预言机服务
- 监控预言机数据准确性
- 设置价格异常检测

### 3. 权限管理

- 定期审查白名单
- 监控异常交易
- 及时处理安全事件

### 4. 资金安全

- 定期检查合约余额
- 监控大额交易
- 设置交易限额

## 监控和维护

### 1. 事件监控

监听以下关键事件：

```javascript
// 监听购买事件
marketplace.on("Purchase", (buyer, seller, rwaToken, payToken, rwaAmount, payAmount, fee) => {
  console.log("Purchase:", { buyer, seller, rwaToken, payToken, rwaAmount, payAmount, fee });
});

// 监听手续费变更
marketplace.on("FeeChanged", (newFeeBps) => {
  console.log("Fee changed to:", newFeeBps);
});
```

### 2. 定期维护

- 更新预言机地址
- 调整手续费策略
- 优化白名单管理
- 升级合约版本

## 故障排除

### 常见问题

1. **预言机价格获取失败**
   - 检查预言机地址是否正确
   - 确认Token对是否支持
   - 验证网络连接

2. **KYC验证失败**
   - 检查用户KYC状态
   - 确认KYC开关设置
   - 验证权限配置

3. **白名单验证失败**
   - 检查地址是否在白名单
   - 确认白名单设置正确
   - 验证权限配置

### 紧急处理

1. **暂停合约**
   ```javascript
   await marketplace.setPaused(true);
   ```

2. **升级预言机**
   ```javascript
   await marketplace.setPriceOracle(newOracleAddress);
   ```

3. **调整手续费**
   ```javascript
   await marketplace.setFeeBps(0); // 临时取消手续费
   ```

## 联系支持

如遇到部署或测试问题，请联系开发团队或查看项目文档。 