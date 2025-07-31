const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("🔍 验证KYC状态...");

  // 读取部署信息
  const deploymentPath = path.join(__dirname, "deployment-sepolia.json");
  if (!fs.existsSync(deploymentPath)) {
    throw new Error("❌ 未找到部署信息文件，请先运行部署脚本");
  }

  const deploymentInfo = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));

  // 要检查的地址
  const addressesToCheck = [
    "0x351b8cdb9698e2563be7f6dca1f3d70e8770e277",
    "0xC4736E41D02faa7D735819AA9afa2ffee1Ce5931"
  ];

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
    const marketplaceAddress = await marketplace.getAddress();

    // 检查KYC状态
    console.log("\n🔍 检查KYC验证状态...");
    console.log("=".repeat(60));
    
    for (const address of addressesToCheck) {
      const status = await rwaToken.isKYCVerified(address);
      console.log(`地址: ${address}`);
      console.log(`KYC状态: ${status ? "✅ 已验证" : "❌ 未验证"}`);
      console.log("-".repeat(40));
    }

    // 检查Marketplace合约的KYC状态
    const marketplaceKYCStatus = await rwaToken.isKYCVerified(marketplaceAddress);
    console.log(`Marketplace合约: ${marketplaceAddress}`);
    console.log(`KYC状态: ${marketplaceKYCStatus ? "✅ 已验证" : "❌ 未验证"}`);
    console.log("-".repeat(40));

    // 检查RWA Token的其他信息
    console.log("\n📊 RWA Token合约信息:");
    console.log("=".repeat(60));
    console.log("代币名称:", await rwaToken.name());
    console.log("代币符号:", await rwaToken.symbol());
    console.log("总供应量:", ethers.formatEther(await rwaToken.totalSupply()));
    console.log("发行人:", await rwaToken.getIssuer());
    console.log("资产类型:", await rwaToken.getUnderlyingAssetType());
    console.log("资产ID:", await rwaToken.getUnderlyingAssetId());
    console.log("合规状态:", await rwaToken.getComplianceStatus() ? "✅ 合规" : "❌ 不合规");
    console.log("=".repeat(60));

    // 检查Marketplace合约信息
    console.log("\n🏪 Marketplace合约信息:");
    console.log("=".repeat(60));
    const platformFee = await marketplace.platformFee();
    console.log("平台费用:", platformFee.toString(), "基点");
    console.log("平台费用百分比:", Number(platformFee) / 100, "%");
    console.log("订单计数器:", (await marketplace.orderCounter()).toString());
    console.log("暂停状态:", await marketplace.paused() ? "⏸️ 已暂停" : "▶️ 正常运行");
    console.log("=".repeat(60));

    console.log("\n🎉 KYC状态验证完成！");

  } catch (error) {
    console.error("❌ KYC状态验证失败:", error);
    throw error;
  }
}

// 错误处理
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ KYC验证脚本执行失败:", error);
    process.exit(1);
  }); 