# Chainlink 价格预言机快速使用指南

## 概述

本指南将帮助您快速在 Polygon 链上配置和使用 Chainlink 价格预言机，为 RWA Marketplace 提供实时价格数据。

## 快速开始

### 1. 安装依赖

```bash
npm install @chainlink/contracts
```

### 2. 部署预言机

```bash
# 部署到 Polygon 主网
npx hardhat run contracts/escrow/deploy/scripts/deploy-chainlink-oracle.js --network polygonMainnet

# 部署到 Mumbai 测试网
npx hardhat run contracts/escrow/deploy/scripts/deploy-chainlink-oracle.js --network polygonTestnet
```

### 3. 配置价格源

```bash
# 设置环境变量
export ORACLE_ADDRESS="你的预言机地址"

# 配置价格源
npx hardhat run contracts/escrow/deploy/scripts/configure-price-feeds.js --network polygonMainnet
```

### 4. 测试预言机

```bash
# 测试价格查询
npx hardhat run contracts/escrow/deploy/scripts/test-chainlink-oracle.js --network polygonMainnet

# 验证特定价格源
npx hardhat run contracts/escrow/deploy/scripts/test-chainlink-oracle.js --network polygonMainnet 0x0A6513e40db6EB1b165753AD52E80663aeA50545
```

### 5. 完整部署（预言机 + Marketplace）

```bash
# 一键部署完整系统
npx hardhat run contracts/escrow/deploy/scripts/deploy-marketplace-with-chainlink.js --network polygonMainnet
```

## 主要功能

### 价格查询

```javascript
// 获取Token的USD价格
const price = await priceOracle.getPrice(tokenAddress);
console.log("价格:", ethers.utils.formatUnits(price, 18));

// 获取两个Token之间的价格比率
const ratio = await priceOracle.getPrice(token1, token2);
console.log("比率:", ethers.utils.formatUnits(ratio, 18));
```

### 价格源管理

```javascript
// 添加价格源
await priceOracle.setPriceFeed(tokenAddress, priceFeedAddress);

// 移除价格源
await priceOracle.removePriceFeed(tokenAddress);

// 检查Token是否支持
const isSupported = await priceOracle.isTokenSupported(tokenAddress);

// 获取所有支持的Token
const supportedTokens = await priceOracle.getSupportedTokens();
```

## Polygon 网络配置

### 主网 (Chain ID: 137)

| Token | 合约地址 | 价格源地址 |
|-------|----------|------------|
| USDT | 0xc2132D05D31c914a87C6611C10748AEb04B58e8F | 0x0A6513e40db6EB1b165753AD52E80663aeA50545 |
| USDC | 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 | 0xfE4A8cc5b5B2366C1B58Bea3858e81843581b2F7 |
| DAI | 0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063 | 0x4746DeC9e833A82EC7C2C1356372CcF2cfcD2F3D |
| WETH | 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619 | 0xF9680D99D6C9589e2a93a78A04A279e509205945 |
| WMATIC | 0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270 | 0xAB594600376Ec9fD91F8e885dADF0CE036862dE0 |

### 测试网 (Chain ID: 80001)

| Token | 合约地址 | 价格源地址 |
|-------|----------|------------|
| USDT | 0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832 | 0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832 |
| USDC | 0x572dDec9087154dC5dfBB1546Bb62713147e0Ab0 | 0x572dDec9087154dC5dfBB1546Bb62713147e0Ab0 |

## 安全特性

### 价格有效性检查

- ✅ 价格过期检查（5分钟）
- ✅ 零价格检查
- ✅ 价格源合约验证
- ✅ 精度转换验证

### 权限管理

- ✅ 只有Owner可以配置价格源
- ✅ 价格源地址验证
- ✅ 批量操作支持

## 常见问题

### Q: 如何添加新的Token价格源？

A: 使用 `setPriceFeed` 函数：

```javascript
await priceOracle.setPriceFeed(tokenAddress, priceFeedAddress);
```

### Q: 价格查询失败怎么办？

A: 检查以下几点：
1. Token是否已配置价格源
2. 价格源是否正常工作
3. 网络连接是否正常
4. 价格数据是否过期

### Q: 如何监控价格变化？

A: 使用事件监听：

```javascript
priceOracle.on("PriceFeedUpdated", (token, priceFeed) => {
  console.log("价格源更新:", token, priceFeed);
});
```

### Q: 如何备份价格源配置？

A: 使用 `getSupportedTokens` 和 `getPriceFeedInfo` 函数：

```javascript
const tokens = await priceOracle.getSupportedTokens();
for (const token of tokens) {
  const info = await priceOracle.getPriceFeedInfo(token);
  console.log(token, info.priceFeed);
}
```

## 成本估算

### 部署费用
- 预言机合约: ~0.1-0.3 MATIC
- 价格源配置: ~0.01-0.05 MATIC per feed
- 总费用: ~0.2-0.5 MATIC

### 运行费用
- 价格查询: ~0.001-0.005 MATIC per query
- 价格源更新: ~0.01-0.05 MATIC per update

## 技术支持

- [Chainlink 官方文档](https://docs.chain.link/)
- [Polygon 网络信息](https://polygon.technology/)
- [Chainlink 价格源地址](https://docs.chain.link/data-feeds/price-feeds/addresses?network=polygon)

## 示例代码

### 完整使用示例

```javascript
const { ethers } = require("hardhat");

async function example() {
  // 1. 获取预言机实例
  const oracleAddress = "你的预言机地址";
  const priceOracle = await ethers.getContractAt("ChainlinkPriceOracle", oracleAddress);
  
  // 2. 查询USDT价格
  const usdtAddress = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
  const usdtPrice = await priceOracle.getPrice(usdtAddress);
  console.log("USDT价格:", ethers.utils.formatUnits(usdtPrice, 18));
  
  // 3. 查询USDC价格
  const usdcAddress = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
  const usdcPrice = await priceOracle.getPrice(usdcAddress);
  console.log("USDC价格:", ethers.utils.formatUnits(usdcPrice, 18));
  
  // 4. 计算USDT/USDC比率
  const ratio = await priceOracle.getPrice(usdtAddress, usdcAddress);
  console.log("USDT/USDC比率:", ethers.utils.formatUnits(ratio, 18));
  
  // 5. 获取支持的Token列表
  const supportedTokens = await priceOracle.getSupportedTokens();
  console.log("支持的Token:", supportedTokens);
}

example();
``` 