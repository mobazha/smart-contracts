# Chainlink 价格预言机在 Polygon 链上的配置和使用指南

## 概述

本指南将帮助您在 Polygon 链上配置和使用 Chainlink 价格预言机，为 RWA Marketplace 提供实时价格数据。

## 1. Polygon 链上的 Chainlink 价格源

### 1.1 主要价格源地址

#### Polygon 主网 (Chain ID: 137)
```javascript
// 主流稳定币价格源
const POLYGON_PRICE_FEEDS = {
  // USDT/USD
  USDT: "0x0A6513e40db6EB1b165753AD52E80663aeA50545",
  
  // USDC/USD  
  USDC: "0xfE4A8cc5b5B2366C1B58Bea3858e81843581b2F7",
  
  // DAI/USD
  DAI: "0x4746DeC9e833A82EC7C2C1356372CcF2cfcD2F3D",
  
  // WETH/USD
  WETH: "0xF9680D99D6C9589e2a93a78A04A279e509205945",
  
  // WMATIC/USD
  WMATIC: "0xAB594600376Ec9fD91F8e885dADF0CE036862dE0",
  
  // WBTC/USD
  WBTC: "0xDE31F8bFBD8c84b5360CFACCa3539B938dd78ae6",
  
  // LINK/USD
  LINK: "0xd9FFdb71EbE7496cC440f22A2Cb1Ca588444cEff"
};

// Token 合约地址
const POLYGON_TOKENS = {
  USDT: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
  USDC: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", 
  DAI: "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063",
  WETH: "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619",
  WMATIC: "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270",
  WBTC: "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6",
  LINK: "0x53E0bca35eC356BD5ddDFebbD1Fc0fD03FaBad39"
};
```

#### Polygon 测试网 (Chain ID: 80001)
```javascript
// Mumbai 测试网价格源
const MUMBAI_PRICE_FEEDS = {
  // USDT/USD
  USDT: "0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832",
  
  // USDC/USD
  USDC: "0x572dDec9087154dC5dfBB1546Bb62713147e0Ab0",
  
  // DAI/USD  
  DAI: "0x0FCAa9c899EC5A91eBc44D1156c234367e9C4a2c",
  
  // WETH/USD
  WETH: "0x0715A7794a1dc8e42615F059dD6e406A6594651A",
  
  // WMATIC/USD
  WMATIC: "0xd0D5e3DB44DE05E9F294BB0a3bEEaF030DE24Ada"
};
```

### 1.2 价格源验证

在部署前，请验证价格源地址的有效性：

```javascript
// 验证价格源
async function verifyPriceFeed(priceFeedAddress) {
  const aggregator = await ethers.getContractAt("AggregatorV3Interface", priceFeedAddress);
  
  try {
    const latestRound = await aggregator.latestRoundData();
    console.log("价格源有效:", {
      roundId: latestRound.roundId.toString(),
      price: latestRound.answer.toString(),
      timestamp: new Date(latestRound.timestamp * 1000).toISOString(),
      decimals: await aggregator.decimals()
    });
    return true;
  } catch (error) {
    console.error("价格源无效:", error.message);
    return false;
  }
}
```

## 2. 部署步骤

### 2.1 安装依赖

```bash
npm install @chainlink/contracts
```

### 2.2 部署 ChainlinkPriceOracle 合约

```javascript
// scripts/deploy-chainlink-oracle.js
const { ethers } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("部署账户:", deployer.address);

  // 部署价格预言机
  const ChainlinkPriceOracle = await ethers.getContractFactory("ChainlinkPriceOracle");
  const priceOracle = await ChainlinkPriceOracle.deploy();
  await priceOracle.deployed();
  
  console.log("ChainlinkPriceOracle 已部署到:", priceOracle.address);
  
  // 验证部署
  const supportedTokens = await priceOracle.getSupportedTokens();
  console.log("支持的Token数量:", supportedTokens.length);
  
  return priceOracle.address;
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
```

### 2.3 配置价格源

```javascript
// scripts/configure-price-feeds.js
const { ethers } = require("hardhat");

async function configurePriceFeeds() {
  const oracleAddress = "YOUR_ORACLE_ADDRESS";
  const priceOracle = await ethers.getContractAt("ChainlinkPriceOracle", oracleAddress);
  
  // Polygon 主网价格源配置
  const priceFeeds = [
    {
      token: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F", // USDT
      feed: "0x0A6513e40db6EB1b165753AD52E80663aeA50545"
    },
    {
      token: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", // USDC
      feed: "0xfE4A8cc5b5B2366C1B58Bea3858e81843581b2F7"
    },
    {
      token: "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063", // DAI
      feed: "0x4746DeC9e833A82EC7C2C1356372CcF2cfcD2F3D"
    }
  ];
  
  for (const { token, feed } of priceFeeds) {
    try {
      const tx = await priceOracle.setPriceFeed(token, feed);
      await tx.wait();
      console.log(`价格源配置成功: ${token} -> ${feed}`);
    } catch (error) {
      console.error(`价格源配置失败: ${token}`, error.message);
    }
  }
}

configurePriceFeeds();
```

## 3. 集成到 RWA Marketplace

### 3.1 更新 RWA Marketplace 部署

```javascript
// scripts/deploy-marketplace-with-chainlink.js
const { ethers } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  
  // 1. 部署 Chainlink 价格预言机
  const ChainlinkPriceOracle = await ethers.getContractFactory("ChainlinkPriceOracle");
  const priceOracle = await ChainlinkPriceOracle.deploy();
  await priceOracle.deployed();
  console.log("ChainlinkPriceOracle:", priceOracle.address);
  
  // 2. 部署 RWA Marketplace
  const RwaMarketplace = await ethers.getContractFactory("RwaMarketplace");
  const marketplace = await RwaMarketplace.deploy(priceOracle.address, deployer.address);
  await marketplace.deployed();
  console.log("RwaMarketplace:", marketplace.address);
  
  // 3. 配置白名单
  const usdtAddress = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
  const usdcAddress = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
  
  await marketplace.setPayTokenWhitelist(usdtAddress, true);
  await marketplace.setPayTokenWhitelist(usdcAddress, true);
  
  console.log("部署完成!");
}

main();
```

### 3.2 测试价格查询

```javascript
// scripts/test-price-oracle.js
const { ethers } = require("hardhat");

async function testPriceOracle() {
  const oracleAddress = "YOUR_ORACLE_ADDRESS";
  const priceOracle = await ethers.getContractAt("ChainlinkPriceOracle", oracleAddress);
  
  const usdtAddress = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
  const usdcAddress = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
  
  // 获取 USDT 价格
  const usdtPrice = await priceOracle.getPrice(usdtAddress);
  console.log("USDT 价格:", ethers.utils.formatUnits(usdtPrice, 18));
  
  // 获取 USDC 价格
  const usdcPrice = await priceOracle.getPrice(usdcAddress);
  console.log("USDC 价格:", ethers.utils.formatUnits(usdcPrice, 18));
  
  // 获取 USDT/USDC 比率
  const usdtUsdcRatio = await priceOracle.getPrice(usdtAddress, usdcAddress);
  console.log("USDT/USDC 比率:", ethers.utils.formatUnits(usdtUsdcRatio, 18));
}

testPriceOracle();
```

## 4. 安全考虑

### 4.1 价格有效性检查

- **价格过期检查**: 确保价格数据不超过5分钟
- **零价格检查**: 拒绝零价格或负价格
- **价格偏差检查**: 检查价格变化是否在合理范围内

### 4.2 权限管理

- 价格源配置权限应归属多签钱包
- 定期审查和更新价格源地址
- 监控价格源的健康状态

### 4.3 应急机制

```javascript
// 紧急暂停价格源
async function emergencyPausePriceFeed(token) {
  const priceOracle = await ethers.getContractAt("ChainlinkPriceOracle", oracleAddress);
  await priceOracle.removePriceFeed(token);
  console.log(`价格源已暂停: ${token}`);
}
```

## 5. 监控和维护

### 5.1 价格监控

```javascript
// 监控价格变化
async function monitorPrices() {
  const priceOracle = await ethers.getContractAt("ChainlinkPriceOracle", oracleAddress);
  const supportedTokens = await priceOracle.getSupportedTokens();
  
  for (const token of supportedTokens) {
    try {
      const price = await priceOracle.getPrice(token);
      const info = await priceOracle.getPriceFeedInfo(token);
      console.log(`Token: ${token}, Price: ${ethers.utils.formatUnits(price, 18)}, Decimals: ${info.decimals}`);
    } catch (error) {
      console.error(`价格查询失败: ${token}`, error.message);
    }
  }
}
```

### 5.2 定期维护

- 每周检查价格源状态
- 监控价格更新频率
- 备份价格源配置
- 更新支持的Token列表

## 6. 故障排除

### 6.1 常见问题

1. **价格源地址错误**
   - 验证地址格式
   - 检查网络配置
   - 确认价格源合约存在

2. **价格数据过期**
   - 检查网络连接
   - 验证价格源状态
   - 考虑更换备用价格源

3. **精度转换错误**
   - 确认Token精度设置
   - 检查价格源精度
   - 验证计算逻辑

### 6.2 调试工具

```javascript
// 调试价格源
async function debugPriceFeed(priceFeedAddress) {
  const aggregator = await ethers.getContractAt("AggregatorV3Interface", priceFeedAddress);
  
  const [roundId, price, startedAt, updatedAt, answeredInRound] = await aggregator.latestRoundData();
  
  console.log("价格源调试信息:", {
    roundId: roundId.toString(),
    price: price.toString(),
    startedAt: new Date(startedAt * 1000).toISOString(),
    updatedAt: new Date(updatedAt * 1000).toISOString(),
    answeredInRound: answeredInRound.toString(),
    decimals: await aggregator.decimals(),
    description: await aggregator.description()
  });
}
```

## 7. 成本估算

### 7.1 Polygon 网络费用

- **部署费用**: ~0.1-0.5 MATIC
- **价格源配置**: ~0.01-0.05 MATIC per feed
- **价格查询**: ~0.001-0.005 MATIC per query

### 7.2 优化建议

- 批量配置价格源
- 缓存价格数据
- 使用批量查询接口
- 考虑使用 Chainlink 的聚合器

## 8. 参考资料

- [Chainlink 官方文档](https://docs.chain.link/)
- [Polygon 网络信息](https://polygon.technology/)
- [Chainlink 价格源地址](https://docs.chain.link/data-feeds/price-feeds/addresses)
- [Polygon 上的 Chainlink](https://docs.chain.link/data-feeds/price-feeds/addresses?network=polygon) 