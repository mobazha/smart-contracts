const { ethers } = require("hardhat");

// Polygon 主网价格源配置
const POLYGON_PRICE_FEEDS = {
  // USDT/USD
  "0xc2132D05D31c914a87C6611C10748AEb04B58e8F": "0x0A6513e40db6EB1b165753AD52E80663aeA50545",
  
  // USDC/USD  
  "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174": "0xfE4A8cc5b5B2366C1B58Bea3858e81843581b2F7",
  
  // DAI/USD
  "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063": "0x4746DeC9e833A82EC7C2C1356372CcF2cfcD2F3D",
  
  // WETH/USD
  "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619": "0xF9680D99D6C9589e2a93a78A04A279e509205945",
  
  // WMATIC/USD
  "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270": "0xAB594600376Ec9fD91F8e885dADF0CE036862dE0",
  
  // WBTC/USD
  "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6": "0xDE31F8bFBD8c84b5360CFACCa3539B938dd78ae6",
  
  // LINK/USD
  "0x53E0bca35eC356BD5ddDFebbD1Fc0fD03FaBad39": "0xd9FFdb71EbE7496cC440f22A2Cb1Ca588444cEff"
};

// Mumbai 测试网价格源配置
const MUMBAI_PRICE_FEEDS = {
  // USDT/USD
  "0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832": "0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832",
  
  // USDC/USD
  "0x572dDec9087154dC5dfBB1546Bb62713147e0Ab0": "0x572dDec9087154dC5dfBB1546Bb62713147e0Ab0",
  
  // DAI/USD  
  "0x0FCAa9c899EC5A91eBc44D1156c234367e9C4a2c": "0x0FCAa9c899EC5A91eBc44D1156c234367e9C4a2c",
  
  // WETH/USD
  "0x0715A7794a1dc8e42615F059dD6e406A6594651A": "0x0715A7794a1dc8e42615F059dD6e406A6594651A",
  
  // WMATIC/USD
  "0xd0D5e3DB44DE05E9F294BB0a3bEEaF030DE24Ada": "0xd0D5e3DB44DE05E9F294BB0a3bEEaF030DE24Ada"
};

async function configurePriceFeeds() {
  const oracleAddress = process.env.ORACLE_ADDRESS;
  if (!oracleAddress) {
    console.error("请设置环境变量 ORACLE_ADDRESS");
    process.exit(1);
  }

  const network = await ethers.provider.getNetwork();
  const isPolygonMainnet = network.chainId === 137;
  const isMumbaiTestnet = network.chainId === 80001;
  
  console.log("配置 Chainlink 价格源");
  console.log("预言机地址:", oracleAddress);
  console.log("网络:", network.name);
  console.log("Chain ID:", network.chainId);
  console.log("=".repeat(50));

  // 选择价格源配置
  let priceFeeds;
  if (isPolygonMainnet) {
    priceFeeds = POLYGON_PRICE_FEEDS;
    console.log("使用 Polygon 主网价格源配置");
  } else if (isMumbaiTestnet) {
    priceFeeds = MUMBAI_PRICE_FEEDS;
    console.log("使用 Mumbai 测试网价格源配置");
  } else {
    console.error("不支持的网络，请使用 Polygon 主网或 Mumbai 测试网");
    process.exit(1);
  }

  try {
    const priceOracle = await ethers.getContractAt("ChainlinkPriceOracle", oracleAddress);
    
    // 获取当前支持的Token
    const currentSupportedTokens = await priceOracle.getSupportedTokens();
    console.log("当前支持的Token数量:", currentSupportedTokens.length);
    
    // 配置价格源
    console.log("\n开始配置价格源...");
    let successCount = 0;
    let failCount = 0;
    
    for (const [token, feed] of Object.entries(priceFeeds)) {
      try {
        // 检查是否已经配置
        const isSupported = await priceOracle.isTokenSupported(token);
        if (isSupported) {
          console.log(`Token ${token} 已配置，跳过`);
          continue;
        }
        
        console.log(`配置 Token ${token} -> ${feed}`);
        const tx = await priceOracle.setPriceFeed(token, feed);
        await tx.wait();
        console.log(`✅ Token ${token} 配置成功`);
        successCount++;
        
        // 等待一下，避免交易冲突
        await new Promise(resolve => setTimeout(resolve, 1000));
        
      } catch (error) {
        console.error(`❌ Token ${token} 配置失败:`, error.message);
        failCount++;
      }
    }
    
    console.log("\n配置结果:");
    console.log(`成功: ${successCount}`);
    console.log(`失败: ${failCount}`);
    
    // 验证配置结果
    console.log("\n验证配置结果...");
    const finalSupportedTokens = await priceOracle.getSupportedTokens();
    console.log("最终支持的Token数量:", finalSupportedTokens.length);
    
    for (const token of finalSupportedTokens) {
      try {
        const info = await priceOracle.getPriceFeedInfo(token);
        console.log(`Token ${token}: ${info.priceFeed}`);
      } catch (error) {
        console.error(`获取Token ${token} 信息失败:`, error.message);
      }
    }
    
    console.log("\n价格源配置完成!");
    
  } catch (error) {
    console.error("配置失败:", error);
    throw error;
  }
}

// 批量移除价格源
async function removePriceFeeds() {
  const oracleAddress = process.env.ORACLE_ADDRESS;
  if (!oracleAddress) {
    console.error("请设置环境变量 ORACLE_ADDRESS");
    process.exit(1);
  }

  console.log("移除价格源");
  console.log("预言机地址:", oracleAddress);
  console.log("=".repeat(50));

  try {
    const priceOracle = await ethers.getContractAt("ChainlinkPriceOracle", oracleAddress);
    
    const supportedTokens = await priceOracle.getSupportedTokens();
    console.log("当前支持的Token数量:", supportedTokens.length);
    
    if (supportedTokens.length === 0) {
      console.log("没有配置的价格源");
      return;
    }
    
    console.log("\n开始移除价格源...");
    let successCount = 0;
    let failCount = 0;
    
    for (const token of supportedTokens) {
      try {
        console.log(`移除 Token ${token}`);
        const tx = await priceOracle.removePriceFeed(token);
        await tx.wait();
        console.log(`✅ Token ${token} 移除成功`);
        successCount++;
        
        // 等待一下，避免交易冲突
        await new Promise(resolve => setTimeout(resolve, 1000));
        
      } catch (error) {
        console.error(`❌ Token ${token} 移除失败:`, error.message);
        failCount++;
      }
    }
    
    console.log("\n移除结果:");
    console.log(`成功: ${successCount}`);
    console.log(`失败: ${failCount}`);
    
  } catch (error) {
    console.error("移除失败:", error);
    throw error;
  }
}

// 如果直接运行此脚本
if (require.main === module) {
  const command = process.argv[2];
  
  if (command === "remove") {
    removePriceFeeds()
      .then(() => process.exit(0))
      .catch((error) => {
        console.error(error);
        process.exit(1);
      });
  } else {
    configurePriceFeeds()
      .then(() => process.exit(0))
      .catch((error) => {
        console.error(error);
        process.exit(1);
      });
  }
}

module.exports = { configurePriceFeeds, removePriceFeeds }; 