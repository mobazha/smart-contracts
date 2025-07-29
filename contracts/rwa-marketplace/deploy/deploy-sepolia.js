const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("🚀 开始部署RWA Marketplace合约到Sepolia测试网...");

  // 获取部署账户
  const [deployer] = await ethers.getSigners();
  console.log("📝 部署账户:", deployer.address);
  console.log("💰 账户余额:", ethers.utils.formatEther(await deployer.getBalance()));

  // 检查账户余额
  const balance = await deployer.getBalance();
  if (balance.lt(ethers.utils.parseEther("0.1"))) {
    throw new Error("❌ 账户余额不足，请确保有足够的Sepolia ETH");
  }

  try {
    // 1. 部署RWA Marketplace合约
    console.log("\n📦 部署RWA Marketplace合约...");
    const RWAMarketplace = await ethers.getContractFactory("RWAMarketplace");
    const rwaMarketplace = await RWAMarketplace.deploy();
    await rwaMarketplace.deployed();
    console.log("✅ RWA Marketplace合约已部署到:", rwaMarketplace.address);

    // 2. 部署示例RWA Token合约
    console.log("\n🌲 部署森林碳汇信用代币合约...");
    const ExampleRWAToken = await ethers.getContractFactory("ExampleRWAToken");
    const exampleRWAToken = await ExampleRWAToken.deploy(
      "Forest Carbon Credit Token", // 代币名称
      "FCC",                       // 代币符号
      ethers.utils.parseEther("500000"), // 500,000 tokens
      deployer.address              // 发行人地址
    );
    await exampleRWAToken.deployed();
    console.log("✅ 森林碳汇信用代币合约已部署到:", exampleRWAToken.address);

    // 3. 部署示例USDT代币合约（用于测试）
    console.log("\n💵 部署示例USDT代币合约...");
    const MockUSDT = await ethers.getContractFactory("MockUSDT");
    const mockUSDT = await MockUSDT.deploy(
      "Mock USDT",                 // 代币名称
      "USDT",                      // 代币符号
      ethers.utils.parseUnits("1000000", 6) // 1,000,000 USDT (6位小数)
    );
    await mockUSDT.deployed();
    console.log("✅ 示例USDT代币合约已部署到:", mockUSDT.address);

    // 4. 配置RWA Marketplace合约
    console.log("\n⚙️ 配置RWA Marketplace合约...");
    
    // 设置平台费用为0.25%
    const platformFee = 25; // 0.25% = 25/10000
    await rwaMarketplace.setPlatformFee(platformFee);
    console.log("✅ 平台费用已设置为:", platformFee / 100, "%");

    // 5. 验证合约
    console.log("\n🔍 验证合约...");
    await verifyContract(rwaMarketplace.address, []);
    await verifyContract(exampleRWAToken.address, [
      "Forest Carbon Credit Token",
      "FCC",
      ethers.utils.parseEther("500000"),
      deployer.address
    ]);
    await verifyContract(mockUSDT.address, [
      "Mock USDT",
      "USDT",
      ethers.utils.parseUnits("1000000", 6)
    ]);

    // 6. 保存部署信息
    const deploymentInfo = {
      network: "sepolia",
      deployer: deployer.address,
      deploymentTime: new Date().toISOString(),
      contracts: {
        rwaMarketplace: {
          address: rwaMarketplace.address,
          name: "RWAMarketplace",
          description: "RWA Token交易市场合约"
        },
        exampleRWAToken: {
          address: exampleRWAToken.address,
          name: "ExampleRWAToken",
          description: "森林碳汇信用代币合约",
          symbol: "FCC",
          totalSupply: "500000"
        },
        mockUSDT: {
          address: mockUSDT.address,
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
    const deploymentPath = path.join(__dirname, "deployment-sepolia.json");
    fs.writeFileSync(deploymentPath, JSON.stringify(deploymentInfo, null, 2));
    console.log("✅ 部署信息已保存到:", deploymentPath);

    // 7. 打印部署摘要
    console.log("\n🎉 部署完成！");
    console.log("=" * 50);
    console.log("📋 部署摘要:");
    console.log("网络: Sepolia测试网");
    console.log("部署账户:", deployer.address);
    console.log("RWA Marketplace:", rwaMarketplace.address);
    console.log("森林碳汇代币:", exampleRWAToken.address);
    console.log("示例USDT:", mockUSDT.address);
    console.log("平台费用:", platformFee / 100, "%");
    console.log("=" * 50);

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