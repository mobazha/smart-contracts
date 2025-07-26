const { ethers } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with the account:", deployer.address);

  // 配置参数
  const PRICE_ORACLE_ADDRESS = process.env.PRICE_ORACLE_ADDRESS || "0x..."; // 预言机地址
  const PLATFORM_ADDRESS = process.env.PLATFORM_ADDRESS || "0x..."; // 多签钱包地址
  const CONTRACT_MANAGER_ADDRESS = process.env.CONTRACT_MANAGER_ADDRESS || "0x..."; // ContractManager地址

  console.log("Deployment parameters:");
  console.log("- Price Oracle:", PRICE_ORACLE_ADDRESS);
  console.log("- Platform:", PLATFORM_ADDRESS);
  console.log("- Contract Manager:", CONTRACT_MANAGER_ADDRESS);

  try {
    // 1. 部署 RwaMarketplace 合约
    console.log("\n1. Deploying RwaMarketplace...");
    const RwaMarketplace = await ethers.getContractFactory("RwaMarketplace");
    const marketplace = await RwaMarketplace.deploy(PRICE_ORACLE_ADDRESS, PLATFORM_ADDRESS);
    await marketplace.deployed();
    console.log("RwaMarketplace deployed to:", marketplace.address);

    // 2. 注册到 ContractManager
    console.log("\n2. Registering to ContractManager...");
    const contractManager = await ethers.getContractAt("ContractManager", CONTRACT_MANAGER_ADDRESS);
    const tx = await contractManager.setContract("RwaMarketplace", marketplace.address);
    await tx.wait();
    console.log("Marketplace registered to ContractManager");

    // 3. 验证部署
    console.log("\n3. Verifying deployment...");
    const platform = await marketplace.platform();
    const priceOracle = await marketplace.priceOracle();
    const feeBps = await marketplace.feeBps();
    const kycRequired = await marketplace.kycRequired();
    const paused = await marketplace.paused();

    console.log("Deployment verification:");
    console.log("- Platform:", platform);
    console.log("- Price Oracle:", priceOracle);
    console.log("- Fee (bps):", feeBps.toString());
    console.log("- KYC Required:", kycRequired);
    console.log("- Paused:", paused);

    // 4. 输出部署信息
    console.log("\n=== Deployment Summary ===");
    console.log("RwaMarketplace:", marketplace.address);
    console.log("Network:", network.name);
    console.log("Deployer:", deployer.address);
    console.log("==========================");

    // 5. 保存部署信息到文件
    const deploymentInfo = {
      network: network.name,
      deployer: deployer.address,
      marketplace: marketplace.address,
      priceOracle: PRICE_ORACLE_ADDRESS,
      platform: PLATFORM_ADDRESS,
      contractManager: CONTRACT_MANAGER_ADDRESS,
      deploymentTime: new Date().toISOString()
    };

    const fs = require("fs");
    fs.writeFileSync(
      `deployment-${network.name}.json`,
      JSON.stringify(deploymentInfo, null, 2)
    );
    console.log("\nDeployment info saved to deployment-" + network.name + ".json");

  } catch (error) {
    console.error("Deployment failed:", error);
    process.exit(1);
  }
}

// 初始化设置脚本
async function initialize() {
  const [deployer] = await ethers.getSigners();
  
  // 从部署文件读取合约地址
  const fs = require("fs");
  const deploymentFile = `deployment-${network.name}.json`;
  
  if (!fs.existsSync(deploymentFile)) {
    console.error("Deployment file not found:", deploymentFile);
    return;
  }

  const deploymentInfo = JSON.parse(fs.readFileSync(deploymentFile, "utf8"));
  const marketplace = await ethers.getContractAt("RwaMarketplace", deploymentInfo.marketplace);

  console.log("Initializing marketplace...");

  // 示例：设置初始白名单
  const INITIAL_SELLERS = process.env.INITIAL_SELLERS?.split(",") || [];
  const INITIAL_RWA_TOKENS = process.env.INITIAL_RWA_TOKENS?.split(",") || [];
  const INITIAL_PAY_TOKENS = process.env.INITIAL_PAY_TOKENS?.split(",") || [];

  try {
    // 设置卖家白名单
    for (const seller of INITIAL_SELLERS) {
      if (seller && seller !== "0x0000000000000000000000000000000000000000") {
        console.log("Setting seller whitelist:", seller);
        await marketplace.setSellerWhitelist(seller, true);
      }
    }

    // 设置RWA Token白名单
    for (const token of INITIAL_RWA_TOKENS) {
      if (token && token !== "0x0000000000000000000000000000000000000000") {
        console.log("Setting RWA token whitelist:", token);
        await marketplace.setRwaTokenWhitelist(token, true);
      }
    }

    // 设置支付币种白名单
    for (const token of INITIAL_PAY_TOKENS) {
      if (token && token !== "0x0000000000000000000000000000000000000000") {
        console.log("Setting pay token whitelist:", token);
        await marketplace.setPayTokenWhitelist(token, true);
      }
    }

    console.log("Initialization completed successfully");

  } catch (error) {
    console.error("Initialization failed:", error);
  }
}

// 脚本入口
if (require.main === module) {
  const command = process.argv[2];
  
  if (command === "initialize") {
    initialize()
      .then(() => process.exit(0))
      .catch((error) => {
        console.error(error);
        process.exit(1);
      });
  } else {
    main()
      .then(() => process.exit(0))
      .catch((error) => {
        console.error(error);
        process.exit(1);
      });
  }
}

module.exports = { main, initialize }; 