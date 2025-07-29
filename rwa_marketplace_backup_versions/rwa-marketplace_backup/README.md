# RWA Token Marketplace 智能合约

## 概述

RWA Token Marketplace是一个专门用于Real-World Asset (RWA) Token交易的智能合约系统。该系统支持买家下单、卖家发货、RWA Token转移等完整交易流程，并集成了KYC验证、合规检查等RWA Token特有的功能。

## 合约架构

```
rwa-marketplace/
├── RWAMarketplace.sol          # 主合约 - RWA Token交易市场
├── IRWAToken.sol              # RWA Token接口定义
├── ExampleRWAToken.sol        # 示例RWA Token实现
├── deploy/
│   └── RWAMarketplaceDeploy.sol # 部署脚本
├── test/
│   └── RWAMarketplaceTest.sol   # 测试文件
└── README.md                  # 本文档
```

## 核心功能

### 1. 订单管理
- **创建订单**: 买家创建RWA Token购买订单
- **付款**: 买家支付对应金额到智能合约
- **发货**: 卖家转移RWA Token给买家
- **确认收货**: 买家确认收货，完成交易
- **取消订单**: 买卖双方可以取消订单
- **争议处理**: 支持争议订单的处理

### 2. 支付支持
- **ETH支付**: 支持原生ETH支付
- **ERC20代币支付**: 支持USDT、USDC等ERC20代币支付
- **平台费用**: 自动计算和扣除平台费用

### 3. RWA Token特性
- **KYC验证**: 支持KYC验证要求
- **合规检查**: 内置合规状态检查
- **风险等级**: 支持风险等级评估
- **收益率**: 支持收益率信息
- **底层资产**: 支持底层资产信息查询

## 合约接口

### RWAMarketplace.sol

#### 主要函数

```solidity
// 创建订单
function createOrder(
    address seller,
    address rwaTokenAddress,
    address paymentTokenAddress,
    address buyerReceiveAddress,
    uint256 rwaTokenAmount,
    uint256 paymentAmount
) external returns (uint256 orderId)

// 买家付款
function payOrder(uint256 orderId) external payable

// 卖家发货
function shipOrder(uint256 orderId) external

// 买家确认收货
function completeOrder(uint256 orderId) external

// 取消订单
function cancelOrder(uint256 orderId) external

// 争议订单
function disputeOrder(uint256 orderId) external

// 处理争议
function resolveDispute(uint256 orderId, bool refundBuyer) external
```

#### 查询函数

```solidity
// 获取订单信息
function getOrder(uint256 orderId) external view returns (Order memory)

// 获取买家订单列表
function getBuyerOrders(address buyer) external view returns (uint256[] memory)

// 获取卖家订单列表
function getSellerOrders(address seller) external view returns (uint256[] memory)
```

### IRWAToken.sol

RWA Token接口定义了所有RWA Token应该实现的标准方法：

```solidity
// 基本信息
function name() external view returns (string memory)
function symbol() external view returns (string memory)
function decimals() external view returns (uint8)
function totalSupply() external view returns (uint256)

// RWA特有信息
function getUnderlyingAssetType() external view returns (string memory)
function getComplianceStatus() external view returns (bool)
function getKYCRequired() external view returns (bool)
function isKYCVerified(address account) external view returns (bool)
function getRiskLevel() external view returns (uint8)
function getYieldRate() external view returns (uint256)
```

## 交易流程

### 1. 买家下单
```javascript
// 前端调用
const orderId = await marketplace.createOrder(
    sellerAddress,
    rwaTokenAddress,
    paymentTokenAddress, // 0表示ETH
    buyerReceiveAddress,
    rwaTokenAmount,
    paymentAmount
);
```

### 2. 买家付款
```javascript
// ETH支付
await marketplace.payOrder(orderId, { value: paymentAmount });

// ERC20代币支付
await paymentToken.approve(marketplace.address, paymentAmount);
await marketplace.payOrder(orderId);
```

### 3. 卖家发货
```javascript
// 卖家授权Marketplace使用RWA Token
await rwaToken.approve(marketplace.address, rwaTokenAmount);

// 卖家发货
await marketplace.shipOrder(orderId);
```

### 4. 买家确认收货
```javascript
// 买家确认收货，完成交易
await marketplace.completeOrder(orderId);
```

## 部署指南

### 1. 环境准备
```bash
npm install
npm install @openzeppelin/contracts
```

### 2. 编译合约
```bash
npx truffle compile
```

### 3. 部署合约
```javascript
// 部署RWA Marketplace
const RWAMarketplace = artifacts.require("RWAMarketplace");
const marketplace = await RWAMarketplace.new();

// 部署示例RWA Token
const ExampleRWAToken = artifacts.require("ExampleRWAToken");
const rwaToken = await ExampleRWAToken.new(
    "Real Estate Token",
    "RET",
    "1000000000000000000000000", // 100万代币
    issuerAddress
);
```

### 4. 配置合约
```javascript
// 设置授权操作员
await marketplace.setAuthorizedOperator(operatorAddress, true);

// 设置平台费用
await marketplace.setPlatformFee(25); // 0.25%
```

## 测试

### 运行测试
```bash
npx truffle test
```

### 测试覆盖
- 订单创建和状态管理
- 支付流程（ETH和ERC20）
- 发货和确认流程
- 订单取消和争议处理
- KYC验证功能
- 平台费用计算
- 合约暂停/恢复功能

## 安全特性

### 1. 重入攻击防护
- 使用OpenZeppelin的ReentrancyGuard
- 状态更新在外部调用之前

### 2. 权限控制
- 只有买家可以付款
- 只有卖家可以发货
- 只有授权操作员可以处理争议

### 3. 输入验证
- 地址非零检查
- 金额非零检查
- 订单状态验证

### 4. 紧急控制
- 合约暂停功能
- 资金提取功能
- 所有权转移功能

## 前端集成

### 1. 合约交互
```javascript
// 获取合约实例
const marketplace = new web3.eth.Contract(MarketplaceABI, marketplaceAddress);

// 创建订单
const orderId = await marketplace.methods.createOrder(
    seller,
    rwaTokenAddress,
    paymentTokenAddress,
    buyerReceiveAddress,
    rwaTokenAmount,
    paymentAmount
).send({ from: buyerAddress });
```

### 2. 事件监听
```javascript
// 监听订单创建事件
marketplace.events.OrderCreated({
    filter: { buyer: buyerAddress }
}, (error, event) => {
    console.log('Order created:', event.returnValues);
});
```

### 3. 状态查询
```javascript
// 获取订单信息
const order = await marketplace.methods.getOrder(orderId).call();

// 获取买家订单列表
const buyerOrders = await marketplace.methods.getBuyerOrders(buyerAddress).call();
```

## 配置参数

### 平台费用
- 默认: 0.25% (25基点)
- 最大: 10% (1000基点)
- 可调整: 通过`setPlatformFee`函数

### 订单状态
- `CREATED`: 订单已创建
- `PAID`: 买家已付款
- `SHIPPED`: 卖家已发货
- `COMPLETED`: 交易完成
- `CANCELLED`: 订单取消
- `DISPUTED`: 争议中

## 注意事项

### 1. Gas优化
- 批量操作减少Gas消耗
- 合理设置订单金额
- 避免不必要的状态查询

### 2. 用户体验
- 前端显示交易状态
- 提供交易进度反馈
- 错误信息本地化

### 3. 合规要求
- RWA Token需要KYC验证
- 遵守当地法规要求
- 定期更新合规状态

## 扩展功能

### 1. 多链支持
- 支持不同区块链网络
- 跨链资产转移
- 统一接口设计

### 2. 高级功能
- 自动做市商(AMM)集成
- 流动性池支持
- 衍生品交易

### 3. 治理功能
- DAO治理投票
- 参数调整投票
- 升级机制

## 许可证

本项目采用BUSL-1.1许可证。详见LICENSE文件。

## 贡献

欢迎提交Issue和Pull Request来改进这个项目。

## 联系方式

如有问题或建议，请通过以下方式联系：
- GitHub Issues
- 邮箱: support@mobazha.com 