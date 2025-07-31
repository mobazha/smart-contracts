# RWA Marketplace 合约

## 概述

这是一个完整的RWA（Real World Asset）Token交易智能合约系统，支持买家创建订单并付款，卖家发货后完成交易。系统包含KYC验证、合规检查、多种支付方式等企业级功能。

## 合约组成

### 核心合约
- **RWAMarketplace.sol**: 主交易合约
- **ExampleRWAToken.sol**: 示例RWA Token合约（森林碳汇信用代币）
- **IRWAToken.sol**: RWA Token接口定义
- **MockUSDT.sol**: 模拟USDT稳定币合约

### 部署脚本
- **deploy-sepolia.js**: Sepolia测试网部署脚本
- **deploy-bsc.js**: BSC测试网部署脚本
- **test-deployment.js**: 部署测试脚本

## 主要功能

### 1. 订单创建和付款

买家可以使用外部传入的唯一订单ID创建订单并同步付款：

```javascript
// 生成唯一的订单ID
const orderId = ethers.keccak256(ethers.toUtf8Bytes("ORDER_001"));

// 买家创建订单并付款
await marketplace.createOrderAndPay(
    orderId,                    // 外部传入的唯一订单ID
    buyerAddress,               // 买家地址
    sellerAddress,              // 卖家地址
    rwaTokenAddress,            // RWA Token合约地址
    paymentTokenAddress,        // 支付代币地址（0表示ETH）
    buyerReceiveAddress,        // 买家接收RWA Token的地址
    rwaTokenAmount,            // RWA Token数量
    paymentAmount,             // 支付金额
    { value: paymentAmount }   // ETH支付时发送value
);
```

### 2. 订单完成

卖家发货并完成交易：

```javascript
await marketplace.shipAndComplete(orderId, sellerReceiveAddress);
```

### 3. 订单取消

买家或卖家可以取消订单：

```javascript
await marketplace.cancelOrder(orderId);
```

## RWA Token特性

### KYC验证
- 所有RWA Token转移都需要通过KYC验证
- 只有通过KYC的用户才能转移代币
- 管理员可以设置用户的KYC状态

```javascript
// 设置用户KYC状态
await rwaToken.setKYCStatus(userAddress, true);

// 检查KYC状态
const isKYCVerified = await rwaToken.isKYCVerified(userAddress);
```

### 合规检查
- RWA Token具有合规状态检查
- 只有合规的代币才能进行交易
- 管理员可以更新合规状态

```javascript
// 检查合规状态
const complianceStatus = await rwaToken.getComplianceStatus();

// 设置合规状态
await rwaToken.setComplianceStatus(true);
```

### RWA Token信息
```javascript
// 获取底层资产类型
const assetType = await rwaToken.getUnderlyingAssetType(); // "Carbon Credit"

// 获取底层资产ID
const assetId = await rwaToken.getUnderlyingAssetId(); // "CARBON_CREDIT_001"

// 获取发行人地址
const issuer = await rwaToken.getIssuer();
```

## 支付方式

### ETH支付
```javascript
await marketplace.createOrderAndPay(
    orderId,
    buyerAddress,
    sellerAddress,
    rwaTokenAddress,
    "0x0000000000000000000000000000000000000000", // ETH地址
    buyerAddress,
    rwaTokenAmount,
    paymentAmount,
    { value: paymentAmount }
);
```

### ERC20代币支付（如USDT）
```javascript
// 先授权
await paymentToken.approve(marketplaceAddress, paymentAmount);

// 创建订单
await marketplace.createOrderAndPay(
    orderId,
    buyerAddress,
    sellerAddress,
    rwaTokenAddress,
    paymentTokenAddress,
    buyerAddress,
    rwaTokenAmount,
    paymentAmount,
    { value: 0 }
);
```

## 订单ID格式

- 使用`bytes32`类型
- 必须唯一，不能重复
- 建议使用哈希函数生成，如：
  ```javascript
  const orderId = ethers.keccak256(ethers.toUtf8Bytes("ORDER_001"));
  ```

## 查询功能

```javascript
// 获取订单信息
const order = await marketplace.getOrder(orderId);

// 获取买家订单列表
const buyerOrders = await marketplace.getBuyerOrders(buyerAddress);

// 获取卖家订单列表
const sellerOrders = await marketplace.getSellerOrders(sellerAddress);

// 获取合约余额
const ethBalance = await marketplace.getETHBalance();
const tokenBalance = await marketplace.getTokenBalance(tokenAddress);
```

## 平台费用

- 默认平台费用：0.25%（25个基点）
- 可配置，最大10%
- 在订单完成时自动扣除

```javascript
// 设置平台费用
await marketplace.setPlatformFee(50); // 0.5%

// 获取当前费用
const currentFee = await marketplace.platformFee();
```

## 管理员功能

### 合约管理
```javascript
// 暂停合约
await marketplace.pause();

// 恢复合约
await marketplace.unpause();

// 提取资金
await marketplace.withdrawFunds(recipient, amount, tokenAddress);

// 设置授权操作员
await marketplace.setAuthorizedOperator(operator, true);
```

### RWA Token管理
```javascript
// 铸造代币
await rwaToken.mint(to, amount);

// 销毁代币
await rwaToken.burn(from, amount);

// 设置KYC状态
await rwaToken.setKYCStatus(account, true);

// 设置合规状态
await rwaToken.setComplianceStatus(true);
```

## 安全特性

- **重入攻击防护**: 使用ReentrancyGuard
- **暂停功能**: 紧急情况下可暂停合约
- **KYC验证**: 所有RWA Token转移都需要KYC
- **订单状态验证**: 严格的订单状态检查
- **权限控制**: 基于角色的访问控制
- **零地址检查**: 防止无效地址操作
- **金额验证**: 确保金额大于0

## 事件

### Marketplace事件
- `OrderCreated`: 订单创建事件
- `OrderCompleted`: 订单完成事件
- `OrderCancelled`: 订单取消事件
- `FundsWithdrawn`: 资金提取事件

### RWA Token事件
- `RWAComplianceUpdated`: 合规状态更新事件
- `KYCStatusUpdated`: KYC状态更新事件

## 部署指南

详细的部署指南请参考：[DEPLOYMENT_GUIDE.md](./deploy/DEPLOYMENT_GUIDE.md)

### 快速部署
```bash
# 编译合约
npm run compile:hardhat

# 运行测试
npm run test:hardhat

# 部署到Sepolia测试网
npm run deploy:rwa-sepolia

# 部署到BSC测试网
npm run deploy:rwa-bsc
```

## 测试

运行完整的测试套件：

```bash
npm run test:hardhat
```

测试覆盖了以下场景：
- 基本功能测试
- KYC验证测试
- 管理员功能测试
- 稳定币付款功能测试
- 订单创建、完成、取消测试
- 安全特性测试 