# Mobazha Escrow 合约设计逻辑文档

## 概述

Mobazha Escrow 合约是一个基于 BSC (Binance Smart Chain) 的托管合约，用于支持使用 ETH 和 ERC20 代币进行的商品交易。该合约实现了类似 UTXO 加密货币的 2-of-3 多重签名托管机制，确保交易的安全性和公平性。

## 核心设计理念

### 1. 多重签名托管机制
- **2-of-3 签名模式**: 买家、卖家、仲裁人三方中任意两方同意即可释放资金
- **阈值控制**: 通过 `threshold` 参数控制所需签名数量
- **超时机制**: 卖家可在超时后单方面释放资金（当 `timeoutHours > 0` 时）

### 2. 支持多种支付方式
- **ETH 交易**: 原生 BSC 代币交易
- **ERC20 代币交易**: 支持符合标准的 ERC20 代币
- **统一接口**: 两种交易方式使用相同的托管逻辑

## 合约状态管理

### 交易状态 (Status)
```solidity
enum Status {FUNDED, RELEASED}
```
- **FUNDED**: 资金已存入托管，等待释放
- **RELEASED**: 资金已释放，交易完成

### 交易类型 (TransactionType)
```solidity
enum TransactionType {ETH, TOKEN}
```
- **ETH**: 使用原生 BSC 代币
- **TOKEN**: 使用 ERC20 代币

## 核心数据结构

### Transaction 结构体
```solidity
struct Transaction {
    uint256 value;                    // 托管金额
    address payerAddress;             // 支付方地址（记录实际支付交易的地址）
    uint256 lastModified;             // 最后修改时间
    Status status;                    // 交易状态
    TransactionType transactionType;  // 交易类型
    uint8 threshold;                  // 所需签名数量
    uint32 timeoutHours;              // 超时时间（小时）
    address buyer;                    // 买家地址
    address seller;                   // 卖家地址
    address tokenAddress;             // ERC20 代币地址
    address moderator;                // 仲裁人地址
    uint256 released;                 // 已释放金额
    uint256 noOfReleases;             // 释放次数
    mapping(address => bool) isOwner; // 参与者映射
    mapping(bytes32 => bool) voted;   // 投票记录
    mapping(address => bool) beneficiaries; // 受益人记录
}
```

## 业务流程

### 1. 交易创建与资金托管

#### ETH 交易流程
```solidity
function addTransaction(
    address buyer,
    address seller,
    address moderator,
    uint8 threshold,
    uint32 timeoutHours,
    bytes32 scriptHash,
    bytes20 uniqueId
) external payable
```

#### ERC20 代币交易流程
```solidity
function addTokenTransaction(
    address buyer,
    address seller,
    address moderator,
    uint8 threshold,
    uint32 timeoutHours,
    bytes32 scriptHash,
    uint256 value,
    bytes20 uniqueId,
    address tokenAddress
) external
```

**流程步骤:**
1. 买家调用合约，传入交易参数
2. 合约验证参数有效性（地址非零、阈值合理等）
3. 计算并验证 scriptHash
4. 创建 Transaction 记录
5. 转移资金到合约（ETH 直接转账，ERC20 通过 transferFrom）
6. 设置参与者权限映射
7. 更新交易计数和参与者交易列表

### 2. 资金释放机制

#### 正常交易完成
- **参与者**: 买家 + 卖家
- **触发条件**: 商品交付完成，双方达成一致
- **签名要求**: 买家签名 + 卖家签名

#### 纠纷仲裁
- **参与者**: 仲裁人 + 买家/卖家之一
- **触发条件**: 交易出现纠纷，需要仲裁
- **签名要求**: 仲裁人签名 + 争议方签名
- **资金分配**: 根据仲裁结果分配资金

#### 超时释放
- **参与者**: 仅卖家
- **触发条件**: 超过 `timeoutHours` 时间
- **签名要求**: 仅需卖家签名
- **适用场景**: 买家收到商品但不释放资金

#### 卖家退款
- **参与者**: 仅卖家
- **触发条件**: 不满足 `threshold` 要求，但满足退款条件
- **签名要求**: 仅需卖家签名
- **适用场景**: 卖家主动退款给原始支付方
- **退款条件**:
  - 只有一个收款地址
  - 收款地址是 `payerAddress`（原始支付方地址）
  - 卖家确实签名了退款交易
- **安全机制**: 通过 `ecrecover` 验证卖家签名，防止伪造

### 3. 签名验证与执行

#### 签名验证流程
```solidity
function _verifySignatures(
    uint8[] memory sigV,
    bytes32[] memory sigR,
    bytes32[] memory sigS,
    bytes32 scriptHash,
    address payable[] calldata destinations,
    uint256[] calldata amounts
) private
```

**验证步骤:**
1. 验证签名数组长度一致性
2. 计算交易哈希
3. 使用 ecrecover 恢复签名者地址
4. 验证签名者是否为有效参与者
5. 检查签名是否重复使用
6. 记录投票状态

**卖家退款验证:**
当不满足 `threshold` 要求时，系统会检查是否为卖家退款情况：
1. 检查是否只有一个收款地址
2. 验证收款地址是否为 `payerAddress`
3. 通过 `ecrecover` 验证签名是否来自卖家
4. 如果满足退款条件，则允许执行，无需满足 `threshold` 要求

#### 资金释放执行
```solidity
function execute(
    uint8[] calldata sigV,
    bytes32[] calldata sigR,
    bytes32[] calldata sigS,
    bytes32 scriptHash,
    PayData calldata payData
) external
```

**执行步骤:**
1. 验证交易存在且有剩余资金
2. 验证支付数据有效性
3. 验证签名和权限
4. 转移资金到指定地址
5. 更新交易状态和记录
6. 触发 Executed 事件

## 安全机制

### 1. 地址验证
- 禁止零地址作为参与者
- 买家不能与卖家相同
- 仲裁人不能是买家或卖家

### 2. 权限控制
- 只有合约所有者可以转移锁定资金
- 参与者必须通过签名验证
- 防止重复签名攻击

### 3. 资金安全
- 验证资金充足性
- 防止超额释放
- 支持多次释放（部分释放）

### 4. 时间控制
- 超时机制防止资金永久锁定
- 时间戳验证确保时序正确

## 特殊功能

### 1. 离线直接支付
- 设置 `threshold = 1` 实现 1-of-2 多重签名
- 适用于信任度高的交易

### 2. 卖家退款功能
- **功能描述**: 允许卖家在特定条件下主动退款给原始支付方
- **触发条件**: 不满足正常签名要求，但满足退款验证条件
- **安全机制**: 
  - 只能退给 `payerAddress`（原始支付方）
  - 必须通过卖家签名验证
  - 只能有一个收款地址
- **应用场景**: 
  - 卖家主动取消交易
  - 商品缺货或无法交付
  - 双方协商一致退款

### 3. 查询功能
- 查询交易状态和详情
- 查询参与者交易历史
- 验证受益人身份

### 4. 紧急资金转移
```solidity
function transferLockedFunds(
    address receiver,
    uint256 value,
    TransactionType transactionType,
    address tokenAddress
) external
```
- 仅合约所有者可调用
- 用于处理异常情况

## 事件系统

### 主要事件
- **Funded**: 资金托管事件
- **Executed**: 资金释放事件

### 事件用途
- 前端界面更新
- 交易状态追踪
- 审计和监控

## 限制和注意事项

### 1. ERC20 代币兼容性
- 仅支持严格符合 ERC20 标准的代币
- 要求 transfer/transferFrom 返回 true
- 非标准代币可能导致资金永久锁定

### 2. 合约地址限制
- 买家、卖家、仲裁人不能是合约地址
- 合约无法创建签名，会导致资金无法释放

### 3. 推送支付模式
- 使用 push 而非 pull 模式
- 受益人恶意行为可能导致支付失败
- 通过博弈论分析，此类攻击风险较低

## 总结

Mobazha Escrow 合约通过多重签名机制、超时控制、仲裁机制和卖家退款功能，为 BSC 生态系统的商品交易提供了安全可靠的托管服务。合约设计充分考虑了各种交易场景和异常情况，包括：

- **正常交易流程**: 买家、卖家、仲裁人的多重签名机制
- **纠纷处理**: 仲裁人参与的争议解决机制
- **超时保护**: 防止资金永久锁定的超时机制
- **主动退款**: 卖家在特定条件下的主动退款功能
- **支付方追踪**: 通过 `payerAddress` 字段准确记录支付方信息

这些机制确保了资金安全、交易公平性和系统的灵活性，为各种交易场景提供了完善的解决方案。 