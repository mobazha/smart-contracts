const { ethers } = require("hardhat");

async function testChainlinkOracle() {
  const oracleAddress = process.env.ORACLE_ADDRESS;
  if (!oracleAddress) {
    console.error("请设置环境变量 ORACLE_ADDRESS");
    process.exit(1);
  }

  console.log("测试 Chainlink 价格预言机");
  console.log("预言机地址:", oracleAddress);
  console.log("=".repeat(50));

  try {
    // 获取预言机合约实例
    const priceOracle = await ethers.getContractAt("ChainlinkPriceOracle", oracleAddress);

    // 1. 获取支持的Token列表
    console.log("\n1. 获取支持的Token列表...");
    const supportedTokens = await priceOracle.getSupportedTokens();
    console.log("支持的Token数量:", supportedTokens.length);
    
    if (supportedTokens.length === 0) {
      console.log("没有配置价格源，请先配置价格源");
      return;
    }

    // 2. 测试每个Token的价格查询
    console.log("\n2. 测试价格查询...");
    for (const token of supportedTokens) {
      try {
        const price = await priceOracle.getPrice(token);
        const info = await priceOracle.getPriceFeedInfo(token);
        
        console.log(`Token: ${token}`);
        console.log(`  价格: ${ethers.utils.formatUnits(price, 18)} USD`);
        console.log(`  价格源: ${info.priceFeed}`);
        console.log(`  精度: ${info.decimals}`);
        console.log("");
      } catch (error) {
        console.error(`Token ${token} 价格查询失败:`, error.message);
      }
    }

    // 3. 测试Token间价格比率
    console.log("\n3. 测试Token间价格比率...");
    if (supportedTokens.length >= 2) {
      const token1 = supportedTokens[0];
      const token2 = supportedTokens[1];
      
      try {
        const ratio = await priceOracle.getPrice(token1, token2);
        console.log(`${token1} / ${token2} 比率: ${ethers.utils.formatUnits(ratio, 18)}`);
        
        // 反向比率
        const reverseRatio = await priceOracle.getPrice(token2, token1);
        console.log(`${token2} / ${token1} 比率: ${ethers.utils.formatUnits(reverseRatio, 18)}`);
      } catch (error) {
        console.error("比率计算失败:", error.message);
      }
    }

    // 4. 测试特定Token（Polygon主网常用Token）
    console.log("\n4. 测试特定Token...");
    const testTokens = {
      "USDT": "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
      "USDC": "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
      "DAI": "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063",
      "WETH": "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619",
      "WMATIC": "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"
    };

    for (const [name, address] of Object.entries(testTokens)) {
      try {
        const isSupported = await priceOracle.isTokenSupported(address);
        if (isSupported) {
          const price = await priceOracle.getPrice(address);
          console.log(`${name} (${address}): ${ethers.utils.formatUnits(price, 18)} USD`);
        } else {
          console.log(`${name} (${address}): 不支持`);
        }
      } catch (error) {
        console.log(`${name} (${address}): 查询失败 - ${error.message}`);
      }
    }

    // 5. 性能测试
    console.log("\n5. 性能测试...");
    const startTime = Date.now();
    const iterations = 10;
    
    for (let i = 0; i < iterations; i++) {
      try {
        await priceOracle.getPrice(supportedTokens[0]);
      } catch (error) {
        console.error(`性能测试第${i+1}次失败:`, error.message);
      }
    }
    
    const endTime = Date.now();
    const avgTime = (endTime - startTime) / iterations;
    console.log(`平均查询时间: ${avgTime.toFixed(2)}ms`);

    console.log("\n测试完成!");

  } catch (error) {
    console.error("测试失败:", error);
    throw error;
  }
}

// 验证价格源地址
async function verifyPriceFeed(priceFeedAddress) {
  console.log(`验证价格源: ${priceFeedAddress}`);
  
  try {
    const aggregator = await ethers.getContractAt("AggregatorV3Interface", priceFeedAddress);
    
    const latestRound = await aggregator.latestRoundData();
    const decimals = await aggregator.decimals();
    const description = await aggregator.description();
    
    console.log("价格源验证成功:");
    console.log(`  描述: ${description}`);
    console.log(`  精度: ${decimals}`);
    console.log(`  最新轮次: ${latestRound.roundId.toString()}`);
    console.log(`  价格: ${latestRound.answer.toString()}`);
    console.log(`  时间戳: ${new Date(latestRound.timestamp * 1000).toISOString()}`);
    console.log(`  开始时间: ${new Date(latestRound.startedAt * 1000).toISOString()}`);
    console.log(`  回答轮次: ${latestRound.answeredInRound.toString()}`);
    
    return true;
  } catch (error) {
    console.error("价格源验证失败:", error.message);
    return false;
  }
}

// 如果直接运行此脚本
if (require.main === module) {
  // 检查是否有价格源地址参数
  const priceFeedAddress = process.argv[2];
  
  if (priceFeedAddress) {
    // 验证价格源
    verifyPriceFeed(priceFeedAddress)
      .then(() => process.exit(0))
      .catch((error) => {
        console.error(error);
        process.exit(1);
      });
  } else {
    // 测试预言机
    testChainlinkOracle()
      .then(() => process.exit(0))
      .catch((error) => {
        console.error(error);
        process.exit(1);
      });
  }
}

module.exports = { testChainlinkOracle, verifyPriceFeed }; 