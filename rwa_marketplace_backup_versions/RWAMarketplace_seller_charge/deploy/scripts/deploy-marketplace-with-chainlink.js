const { ethers } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("部署账户:", deployer.address);
  console.log("账户余额:", ethers.utils.formatEther(await deployer.getBalance()));

  const network = await ethers.provider.getNetwork();
  console.log("网络:", network.name);
  console.log("Chain ID:", network.chainId);
  console.log("=".repeat(50));

  try {
    // 1. 部署 Chainlink 价格预言机
    console.log("\n1. 部署 ChainlinkPriceOracle...");
    const ChainlinkPriceOracle = await ethers.getContractFactory("ChainlinkPriceOracle");
    const priceOracle = await ChainlinkPriceOracle.deploy();
    await priceOracle.deployed();
    console.log("ChainlinkPriceOracle 已部署到:", priceOracle.address);

    // 2. 配置价格源
    console.log("\n2. 配置价格源...");
    await configurePriceFeeds(priceOracle, network.chainId);

    // 3. 部署 RWA Marketplace
    console.log("\n3. 部署 RwaMarketplace...");
    const RwaMarketplace = await ethers.getContractFactory("RwaMarketplace");
    const marketplace = await RwaMarketplace.deploy(priceOracle.address, deployer.address);
    await marketplace.deployed();
    console.log("RwaMarketplace 已部署到:", marketplace.address);

    // 4. 配置白名单
    console.log("\n4. 配置白名单...");
    await configureWhitelist(marketplace, network.chainId);

    // 5. 验证部署
    console.log("\n5. 验证部署...");
    await verifyDeployment(priceOracle, marketplace);

    // 6. 输出部署信息
    console.log("\n6. 部署信息:");
    console.log("=".repeat(60));
    console.log("ChainlinkPriceOracle:", priceOracle.address);
    console.log("RwaMarketplace:", marketplace.address);
    console.log("平台管理员:", deployer.address);
    console.log("网络:", network.name);
    console.log("Chain ID:", network.chainId);
    console.log("=".repeat(60));

    // 7. 保存部署信息
    const deploymentInfo = {
      network: network.name,
      chainId: network.chainId,
      deployer: deployer.address,
      priceOracle: priceOracle.address,
      marketplace: marketplace.address,
      timestamp: new Date().toISOString()
    };

    console.log("\n部署信息已保存到 deployment-info.json");
    require('fs').writeFileSync(
      'deployment-info.json', 
      JSON.stringify(deploymentInfo, null, 2)
    );

    return { priceOracle: priceOracle.address, marketplace: marketplace.address };

  } catch (error) {
    console.error("部署失败:", error);
    throw error;
  }
}

// 配置价格源
async function configurePriceFeeds(priceOracle, chainId) {
  const isPolygonMainnet = chainId === 137;
  const isMumbaiTestnet = chainId === 80001;
  
  let priceFeeds;
  if (isPolygonMainnet) {
    priceFeeds = {
      "USDT": {
        token: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
        feed: "0x0A6513e40db6EB1b165753AD52E80663aeA50545"
      },
      "USDC": {
        token: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
        feed: "0xfE4A8cc5b5B2366C1B58Bea3858e81843581b2F7"
      },
      "DAI": {
        token: "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063",
        feed: "0x4746DeC9e833A82EC7C2C1356372CcF2cfcD2F3D"
      }
    };
  } else if (isMumbaiTestnet) {
    priceFeeds = {
      "USDT": {
        token: "0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832",
        feed: "0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832"
      },
      "USDC": {
        token: "0x572dDec9087154dC5dfBB1546Bb62713147e0Ab0",
        feed: "0x572dDec9087154dC5dfBB1546Bb62713147e0Ab0"
      }
    };
  } else {
    console.log("跳过价格源配置（不支持的网络）");
    return;
  }

  console.log("配置价格源...");
  for (const [name, config] of Object.entries(priceFeeds)) {
    try {
      console.log(`配置 ${name}...`);
      const tx = await priceOracle.setPriceFeed(config.token, config.feed);
      await tx.wait();
      console.log(`✅ ${name} 配置成功`);
      
      // 等待一下，避免交易冲突
      await new Promise(resolve => setTimeout(resolve, 1000));
    } catch (error) {
      console.error(`❌ ${name} 配置失败:`, error.message);
    }
  }
}

// 配置白名单
async function configureWhitelist(marketplace, chainId) {
  const isPolygonMainnet = chainId === 137;
  const isMumbaiTestnet = chainId === 80001;
  
  let tokens;
  if (isPolygonMainnet) {
    tokens = {
      "USDT": "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
      "USDC": "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
      "DAI": "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"
    };
  } else if (isMumbaiTestnet) {
    tokens = {
      "USDT": "0xA02f6adc7926efeBBd59Fd43A84f4E0c0c91e832",
      "USDC": "0x572dDec9087154dC5dfBB1546Bb62713147e0Ab0"
    };
  } else {
    console.log("跳过白名单配置（不支持的网络）");
    return;
  }

  console.log("配置支付币种白名单...");
  for (const [name, address] of Object.entries(tokens)) {
    try {
      console.log(`配置 ${name} 白名单...`);
      const tx = await marketplace.setPayTokenWhitelist(address, true);
      await tx.wait();
      console.log(`✅ ${name} 白名单配置成功`);
      
      // 等待一下，避免交易冲突
      await new Promise(resolve => setTimeout(resolve, 1000));
    } catch (error) {
      console.error(`❌ ${name} 白名单配置失败:`, error.message);
    }
  }
}

// 验证部署
async function verifyDeployment(priceOracle, marketplace) {
  console.log("验证部署...");
  
  // 验证价格预言机
  const supportedTokens = await priceOracle.getSupportedTokens();
  console.log("支持的Token数量:", supportedTokens.length);
  
  if (supportedTokens.length > 0) {
    try {
      const firstToken = supportedTokens[0];
      const price = await priceOracle.getPrice(firstToken);
      console.log(`示例Token价格: ${ethers.utils.formatUnits(price, 18)} USD`);
    } catch (error) {
      console.log("价格查询验证失败:", error.message);
    }
  }
  
  // 验证Marketplace
  const platform = await marketplace.platform();
  const oracle = await marketplace.priceOracle();
  const feeBps = await marketplace.feeBps();
  const kycRequired = await marketplace.kycRequired();
  
  console.log("Marketplace 配置:");
  console.log("  平台地址:", platform);
  console.log("  预言机地址:", oracle);
  console.log("  手续费率:", feeBps.toString(), "bps");
  console.log("  KYC要求:", kycRequired);
  
  // 验证预言机地址匹配
  if (oracle === priceOracle.address) {
    console.log("✅ 预言机地址匹配");
  } else {
    console.log("❌ 预言机地址不匹配");
  }
}

// 如果直接运行此脚本
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
}

module.exports = { main }; 