const { ethers } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("部署账户:", deployer.address);
  console.log("账户余额:", ethers.utils.formatEther(await deployer.getBalance()));

  try {
    // 1. 部署 ChainlinkPriceOracle 合约
    console.log("\n1. 部署 ChainlinkPriceOracle...");
    const ChainlinkPriceOracle = await ethers.getContractFactory("ChainlinkPriceOracle");
    const priceOracle = await ChainlinkPriceOracle.deploy();
    await priceOracle.deployed();
    console.log("ChainlinkPriceOracle 已部署到:", priceOracle.address);

    // 2. 验证部署
    console.log("\n2. 验证部署...");
    const supportedTokens = await priceOracle.getSupportedTokens();
    console.log("支持的Token数量:", supportedTokens.length);
    console.log("支持的Token列表:", supportedTokens);

    // 3. 测试价格查询
    console.log("\n3. 测试价格查询...");
    if (supportedTokens.length > 0) {
      try {
        const firstToken = supportedTokens[0];
        const price = await priceOracle.getPrice(firstToken);
        console.log(`Token ${firstToken} 价格:`, ethers.utils.formatUnits(price, 18));
      } catch (error) {
        console.log("价格查询测试失败:", error.message);
      }
    }

    // 4. 输出部署信息
    console.log("\n4. 部署信息:");
    console.log("=".repeat(50));
    console.log("ChainlinkPriceOracle 地址:", priceOracle.address);
    console.log("部署者:", deployer.address);
    console.log("网络:", network.name);
    console.log("=".repeat(50));

    return priceOracle.address;

  } catch (error) {
    console.error("部署失败:", error);
    throw error;
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