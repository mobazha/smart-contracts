const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("💰 开始检查合约ETH余额并提取...");

  // 读取部署信息
  const deploymentPath = path.join(__dirname, "deployment-sepolia.json");
  if (!fs.existsSync(deploymentPath)) {
    throw new Error("❌ 未找到部署信息文件，请先运行部署脚本");
  }

  const deploymentInfo = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));

  const [deployer] = await ethers.getSigners();
  console.log("🔑 操作账户:", deployer.address);

  try {
    // 获取Marketplace合约实例
    console.log("\n📦 连接到RWA Marketplace合约...");
    const marketplace = await ethers.getContractAt(
      "RWAMarketplace", 
      deploymentInfo.contracts.rwaMarketplace.address
    );
    console.log("✅ 已连接到Marketplace合约:", deploymentInfo.contracts.rwaMarketplace.address);

    // 检查合约ETH余额
    const contractBalance = await ethers.provider.getBalance(await marketplace.getAddress());
    console.log("\n💰 合约ETH余额:", ethers.formatEther(contractBalance), "ETH");

    if (contractBalance === 0n) {
      console.log("ℹ️ 合约中没有ETH余额");
      return;
    }

    // 检查操作账户是否有权限提取
    console.log("\n🔍 检查提取权限...");
    const owner = await marketplace.owner();
    console.log("合约所有者:", owner);
    console.log("当前操作账户:", deployer.address);

    if (owner.toLowerCase() !== deployer.address.toLowerCase()) {
      console.log("❌ 当前账户不是合约所有者，无法提取ETH");
      console.log("请使用合约所有者账户:", owner);
      return;
    }

    // 检查操作账户余额
    const deployerBalance = await ethers.provider.getBalance(deployer.address);
    console.log("操作账户余额:", ethers.formatEther(deployerBalance), "ETH");

    // 估算gas费用
    const gasPrice = await ethers.provider.getFeeData();
    const estimatedGas = 21000n; // 基本转账gas
    const estimatedFee = estimatedGas * gasPrice.gasPrice;
    console.log("估算gas费用:", ethers.formatEther(estimatedFee), "ETH");

    if (deployerBalance < estimatedFee) {
      console.log("❌ 操作账户余额不足支付gas费用");
      return;
    }

    // 提取ETH
    console.log("\n💸 开始提取ETH...");
    
    // 使用withdrawFunds函数提取ETH
    try {
      console.log("使用withdrawFunds函数提取ETH...");
      const tx = await marketplace.withdrawFunds(
        deployer.address,  // 接收地址
        contractBalance,   // 提取全部余额
        ethers.ZeroAddress // ETH地址（0x0000...）
      );
      console.log("⏳ 等待withdrawFunds交易确认...");
      const receipt = await tx.wait();
      console.log("✅ ETH提取成功！交易哈希:", receipt.hash);
    } catch (error) {
      console.log("withdrawFunds函数调用失败:", error.message);
      
      // 如果withdrawFunds失败，尝试其他方法
      try {
        console.log("尝试使用withdraw函数...");
        const tx = await marketplace.withdraw();
        console.log("⏳ 等待withdraw交易确认...");
        const receipt = await tx.wait();
        console.log("✅ ETH提取成功！交易哈希:", receipt.hash);
      } catch (error2) {
        console.log("withdraw函数不存在或失败，尝试其他方法...");
        
        // 尝试使用emergencyWithdraw函数
        try {
          console.log("尝试使用emergencyWithdraw函数...");
          const tx = await marketplace.emergencyWithdraw();
          console.log("⏳ 等待emergencyWithdraw交易确认...");
          const receipt = await tx.wait();
          console.log("✅ ETH提取成功！交易哈希:", receipt.hash);
        } catch (error3) {
          console.log("emergencyWithdraw函数不存在或失败...");
          console.log("所有提取方法都失败，合约可能没有提取功能");
          console.log("建议检查合约代码，确认是否有提取ETH的功能");
        }
      }
    }

    // 检查提取后的余额
    console.log("\n🔍 检查提取后的余额...");
    const marketplaceAddress = await marketplace.getAddress();
    const newContractBalance = await ethers.provider.getBalance(marketplaceAddress);
    console.log("合约剩余ETH余额:", ethers.formatEther(newContractBalance), "ETH");
    
    const newDeployerBalance = await ethers.provider.getBalance(deployer.address);
    console.log("操作账户新余额:", ethers.formatEther(newDeployerBalance), "ETH");

    if (newContractBalance === 0n) {
      console.log("✅ 所有ETH已成功提取！");
    } else {
      console.log("⚠️ 合约中仍有ETH余额，可能需要其他方法提取");
    }

  } catch (error) {
    console.error("❌ ETH提取失败:", error);
    throw error;
  }
}

// 错误处理
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ ETH提取脚本执行失败:", error);
    process.exit(1);
  }); 