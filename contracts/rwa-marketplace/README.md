# RWA Token Marketplace 智能合约

## 概述

RWA Token Marketplace是一个简化的Real-World Asset (RWA) Token交易智能合约系统。该系统将买家创建订单和付款合并为一个操作，大大简化了交易流程，适合demo演示使用。

## 简化特性

### 🚀 **一键下单付款**
- 买家创建订单和付款同步完成
- 减少操作步骤，提升用户体验
- 支持ETH和ERC20代币支付

### 📦 **简化的交易流程**
1. **买家下单付款** → 一步完成订单创建和付款
2. **卖家发货完成** → 转移RWA Token并完成交易
3. **可选取消** → 买卖双方都可以取消订单

### 🛡️ **安全可靠**
- 重入攻击防护
- 权限控制
- 输入验证
- 紧急暂停功能

## 合约架构

```
rwa-marketplace/
├── RWAMarketplace.sol          # 主合约 - 简化的RWA Token交易市场
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
- **创建订单并付款**: 买家一步完成订单创建和付款
- **发货完成**: 卖家转移RWA Token并完成交易
- **取消订单**: 买卖双方可以取消订单

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
// 买家创建订单并付款（同步完成）
function createOrderAndPay(
    address seller,
    address rwaTokenAddress,
    address paymentTokenAddress,
    address buyerReceiveAddress,
    uint256 rwaTokenAmount,
    uint256 paymentAmount
) external payable returns (uint256 orderId)

// 卖家发货并完成交易
function shipAndComplete(uint256 orderId) external

// 取消订单（买家或卖家都可以取消）
function cancelOrder(uint256 orderId) external
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

## 简化的交易流程

### 1. 买家下单付款
```javascript
// ETH支付
const orderId = await marketplace.createOrderAndPay(
    sellerAddress,
    rwaTokenAddress,
    address(0), // 0表示ETH
    buyerReceiveAddress,
    rwaTokenAmount,
    paymentAmount,
    { value: paymentAmount } // ETH支付
);

// ERC20代币支付
await paymentToken.approve(marketplace.address, paymentAmount);
const orderId = await marketplace.createOrderAndPay(
    sellerAddress,
    rwaTokenAddress,
    paymentTokenAddress,
    buyerReceiveAddress,
    rwaTokenAmount,
    paymentAmount
);
```

### 2. 卖家发货完成
```javascript
// 卖家授权Marketplace使用RWA Token
await rwaToken.approve(marketplace.address, rwaTokenAmount);

// 卖家发货并完成交易
await marketplace.shipAndComplete(orderId);
```

### 3. 取消订单（可选）
```javascript
// 买家或卖家都可以取消订单
await marketplace.cancelOrder(orderId);
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
// 设置平台费用
await marketplace.setPlatformFee(25); // 0.25%
```

## 测试

### 运行测试
```bash
npx truffle test
```

### 测试覆盖
- 创建订单并付款
- 发货完成交易
- 取消订单
- ERC20代币支付
- 平台费用计算
- 合约暂停/恢复功能

## 前端集成示例

### 1. 买家下单付款
```javascript
// 获取合约实例
const marketplace = new web3.eth.Contract(MarketplaceABI, marketplaceAddress);

// ETH支付
const orderId = await marketplace.methods.createOrderAndPay(
    sellerAddress,
    rwaTokenAddress,
    "0x0000000000000000000000000000000000000000", // ETH地址
    buyerReceiveAddress,
    rwaTokenAmount,
    paymentAmount
).send({ 
    from: buyerAddress,
    value: paymentAmount 
});

// ERC20代币支付
await paymentToken.methods.approve(marketplaceAddress, paymentAmount).send({ from: buyerAddress });
const orderId = await marketplace.methods.createOrderAndPay(
    sellerAddress,
    rwaTokenAddress,
    paymentTokenAddress,
    buyerReceiveAddress,
    rwaTokenAmount,
    paymentAmount
).send({ from: buyerAddress });
```

### 2. 卖家发货完成
```javascript
// 授权RWA Token
await rwaToken.methods.approve(marketplaceAddress, rwaTokenAmount).send({ from: sellerAddress });

// 发货完成
await marketplace.methods.shipAndComplete(orderId).send({ from: sellerAddress });
```

### 3. 事件监听
```javascript
// 监听订单创建事件
marketplace.events.OrderCreated({
    filter: { buyer: buyerAddress }
}, (error, event) => {
    console.log('Order created:', event.returnValues);
});

// 监听订单完成事件
marketplace.events.OrderCompleted({
    filter: { seller: sellerAddress }
}, (error, event) => {
    console.log('Order completed:', event.returnValues);
});
```

## 订单状态

### 简化状态管理
- `PAID`: 买家已付款，等待卖家发货
- `COMPLETED`: 交易完成
- `CANCELLED`: 订单取消

## 配置参数

### 平台费用
- 默认: 0.25% (25基点)
- 最大: 10% (1000基点)
- 可调整: 通过`setPlatformFee`函数

## 安全特性

### 1. 重入攻击防护
- 使用OpenZeppelin的ReentrancyGuard
- 状态更新在外部调用之前

### 2. 权限控制
- 只有买家可以创建订单并付款
- 只有卖家可以发货完成
- 买卖双方都可以取消订单

### 3. 输入验证
- 地址非零检查
- 金额非零检查
- 订单状态验证

### 4. 紧急控制
- 合约暂停功能
- 资金提取功能
- 所有权转移功能

## 优势对比

### 简化前（复杂版本）
1. 买家创建订单
2. 买家付款
3. 卖家发货
4. 买家确认收货
5. 交易完成

### 简化后（当前版本）
1. 买家创建订单并付款
2. 卖家发货完成
3. 交易完成

**操作步骤从5步减少到2步，大大简化了用户体验！**

## 注意事项

### 1. Gas优化
- 合并操作减少Gas消耗
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