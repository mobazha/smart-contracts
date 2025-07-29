const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("🚀 开始部署RWA Marketplace合约到BSC测试网...");

  // 获取部署账户
  const [deployer] = await ethers.getSigners();
  console.log("📝 部署账户:", deployer.address);
  
  // 获取账户余额
  const balance = await ethers.provider.getBalance(deployer.address);
  console.log("💰 账户余额:", ethers.formatEther(balance));

  // 检查账户余额
  if (balance < ethers.parseEther("0.01")) {
    throw new Error("❌ 账户余额不足，请确保有足够的BSC测试币");
  }

  try {
    // 1. 部署RWA Marketplace合约
    console.log("\n📦 部署RWA Marketplace合约...");
    const RWAMarketplace = await ethers.getContractFactory("RWAMarketplace");
    const rwaMarketplace = await RWAMarketplace.deploy();
    await rwaMarketplace.waitForDeployment();
    const rwaMarketplaceAddress = await rwaMarketplace.getAddress();
    console.log("✅ RWA Marketplace合约已部署到:", rwaMarketplaceAddress);

    // 2. 部署示例RWA Token合约
    console.log("\n🌲 部署森林碳汇信用代币合约...");
    const ExampleRWAToken = await ethers.getContractFactory("ExampleRWAToken");
    const exampleRWAToken = await ExampleRWAToken.deploy(
      "Forest Carbon Credit Token", // 代币名称
      "FCC",                       // 代币符号
      ethers.parseEther("500000"), // 500,000 tokens
      deployer.address              // 发行人地址
    );
    await exampleRWAToken.waitForDeployment();
    const exampleRWATokenAddress = await exampleRWAToken.getAddress();
    console.log("✅ 森林碳汇信用代币合约已部署到:", exampleRWATokenAddress);

    // 3. 部署示例USDT代币合约（用于测试）
    console.log("\n💵 部署示例USDT代币合约...");
    const MockUSDT = await ethers.getContractFactory("MockUSDT");
    const mockUSDT = await MockUSDT.deploy();
    await mockUSDT.waitForDeployment();
    const mockUSDTAddress = await mockUSDT.getAddress();
    console.log("✅ 示例USDT代币合约已部署到:", mockUSDTAddress);

    // 4. 配置RWA Marketplace合约
    console.log("\n⚙️ 配置RWA Marketplace合约...");
    
    // 设置平台费用为0.25%
    const platformFee = 25; // 0.25% = 25/10000
    await rwaMarketplace.setPlatformFee(platformFee);
    console.log("✅ 平台费用已设置为:", platformFee / 100, "%");

    // 5. 验证合约
    console.log("\n🔍 验证合约...");
    await verifyContract(rwaMarketplaceAddress, []);
    await verifyContract(exampleRWATokenAddress, [
      "Forest Carbon Credit Token",
      "FCC",
      ethers.parseEther("500000"),
      deployer.address
    ]);
    await verifyContract(mockUSDTAddress, []);

    // 6. 保存部署信息
    const deploymentInfo = {
      network: "bscTestnet",
      deployer: deployer.address,
      deploymentTime: new Date().toISOString(),
      contracts: {
        rwaMarketplace: {
          address: rwaMarketplaceAddress,
          name: "RWAMarketplace",
          description: "RWA Token交易市场合约"
        },
        exampleRWAToken: {
          address: exampleRWATokenAddress,
          name: "ExampleRWAToken",
          description: "森林碳汇信用代币合约",
          symbol: "FCC",
          totalSupply: "500000"
        },
        mockUSDT: {
          address: mockUSDTAddress,
          name: "MockUSDT",
          description: "示例USDT代币合约",
          symbol: "USDT",
          totalSupply: "1000000"
        }
      },
      configuration: {
        platformFee: platformFee,
        platformFeePercentage: platformFee / 100
      }
    };

    // 保存到文件
    const deploymentPath = path.join(__dirname, "deployment-bsc.json");
    fs.writeFileSync(deploymentPath, JSON.stringify(deploymentInfo, null, 2));
    console.log("✅ 部署信息已保存到:", deploymentPath);

    // 7. 打印部署摘要
    console.log("\n🎉 部署完成！");
    console.log("=".repeat(50));
    console.log("📋 部署摘要:");
    console.log("网络: BSC测试网");
    console.log("部署账户:", deployer.address);
    console.log("RWA Marketplace:", rwaMarketplaceAddress);
    console.log("森林碳汇代币:", exampleRWATokenAddress);
    console.log("示例USDT:", mockUSDTAddress);
    console.log("平台费用:", platformFee / 100, "%");
    console.log("=".repeat(50));

    return deploymentInfo;

  } catch (error) {
    console.error("❌ 部署失败:", error);
    throw error;
  }
}

// 验证合约函数
async function verifyContract(contractAddress, constructorArguments) {
  try {
    console.log(`🔍 验证合约 ${contractAddress}...`);
    await hre.run("verify:verify", {
      address: contractAddress,
      constructorArguments: constructorArguments,
    });
    console.log("✅ 合约验证成功");
  } catch (error) {
    if (error.message.includes("Already Verified")) {
      console.log("✅ 合约已经验证过了");
    } else {
      console.log("⚠️ 合约验证失败:", error.message);
    }
  }
}

// 错误处理
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ 部署脚本执行失败:", error);
    process.exit(1);
  }); 