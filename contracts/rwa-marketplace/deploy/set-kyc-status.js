const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("🔐 开始设置KYC验证状态...");

  // 读取部署信息
  const deploymentPath = path.join(__dirname, "deployment-sepolia.json");
  if (!fs.existsSync(deploymentPath)) {
    throw new Error("❌ 未找到部署信息文件，请先运行部署脚本");
  }

  const deploymentInfo = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));
  console.log("📋 部署信息:", deploymentInfo);

  // 要添加到KYC列表的地址
  const addressesToVerify = [
    "0x351b8cdb9698e2563be7f6dca1f3d70e8770e277",
    "0xC4736E41D02faa7D735819AA9afa2ffee1Ce5931"
  ];

  const [deployer] = await ethers.getSigners();
  console.log("🔑 操作账户:", deployer.address);

  try {
    // 获取RWA Token合约实例
    console.log("\n🌲 连接到森林碳汇信用代币合约...");
    const rwaToken = await ethers.getContractAt(
      "ExampleRWAToken", 
      deploymentInfo.contracts.exampleRWAToken.address
    );
    console.log("✅ 已连接到RWA Token合约:", deploymentInfo.contracts.exampleRWAToken.address);

    // 获取Marketplace合约实例
    console.log("\n📦 连接到RWA Marketplace合约...");
    const marketplace = await ethers.getContractAt(
      "RWAMarketplace", 
      deploymentInfo.contracts.rwaMarketplace.address
    );
    console.log("✅ 已连接到Marketplace合约:", deploymentInfo.contracts.rwaMarketplace.address);

    // 设置KYC状态
    console.log("\n🔐 设置KYC验证状态...");
    
    for (const address of addressesToVerify) {
      console.log(`\n📝 设置地址 ${address} 的KYC状态...`);
      
      // 检查当前KYC状态
      const currentStatus = await rwaToken.isKYCVerified(address);
      console.log(`当前KYC状态: ${currentStatus ? "已验证" : "未验证"}`);
      
      if (!currentStatus) {
        // 设置KYC状态为已验证
        const tx = await rwaToken.setKYCStatus(address, true);
        console.log("⏳ 等待交易确认...");
        await tx.wait();
        console.log("✅ KYC状态设置成功！交易哈希:", tx.hash);
      } else {
        console.log("✅ 地址已经通过KYC验证");
      }
    }

    // 验证设置结果
    console.log("\n🔍 验证KYC设置结果...");
    for (const address of addressesToVerify) {
      const status = await rwaToken.isKYCVerified(address);
      console.log(`地址 ${address}: ${status ? "✅ 已验证" : "❌ 未验证"}`);
    }

    // 设置Marketplace合约的KYC状态（用于交易）
    console.log("\n🏪 设置Marketplace合约的KYC状态...");
    const marketplaceAddress = await marketplace.getAddress();
    const marketplaceKYCStatus = await rwaToken.isKYCVerified(marketplaceAddress);
    
    if (!marketplaceKYCStatus) {
      const tx = await rwaToken.setKYCStatus(marketplaceAddress, true);
      console.log("⏳ 等待Marketplace KYC设置交易确认...");
      await tx.wait();
      console.log("✅ Marketplace KYC状态设置成功！");
    } else {
      console.log("✅ Marketplace已经通过KYC验证");
    }

    // 保存KYC设置信息
    const kycInfo = {
      network: "sepolia",
      setter: deployer.address,
      settingTime: new Date().toISOString(),
      verifiedAddresses: addressesToVerify,
      marketplaceAddress: marketplaceAddress,
      rwaTokenAddress: deploymentInfo.contracts.exampleRWAToken.address
    };

    const kycPath = path.join(__dirname, "kyc-status-sepolia.json");
    fs.writeFileSync(kycPath, JSON.stringify(kycInfo, null, 2));
    console.log("✅ KYC设置信息已保存到:", kycPath);

    // 打印设置摘要
    console.log("\n🎉 KYC设置完成！");
    console.log("=".repeat(50));
    console.log("📋 KYC设置摘要:");
    console.log("网络: Sepolia测试网");
    console.log("操作账户:", deployer.address);
    console.log("RWA Token合约:", deploymentInfo.contracts.exampleRWAToken.address);
    console.log("Marketplace合约:", deploymentInfo.contracts.rwaMarketplace.address);
    console.log("已验证地址:");
    for (const address of addressesToVerify) {
      console.log(`  - ${address}`);
    }
    console.log("=".repeat(50));

  } catch (error) {
    console.error("❌ KYC设置失败:", error);
    throw error;
  }
}

// 错误处理
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ KYC设置脚本执行失败:", error);
    process.exit(1);
  }); 