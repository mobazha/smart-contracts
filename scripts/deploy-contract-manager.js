const { ethers } = require("hardhat");

async function main() {
  console.log("开始部署ContractManager合约到Base主网...");

  // 获取部署者账户
  const [deployer] = await ethers.getSigners();
  console.log("部署者地址:", deployer.address);
  
  // 检查账户余额
  const balance = await deployer.provider.getBalance(deployer.address);
  console.log("账户余额:", ethers.formatEther(balance), "ETH");

  if (balance === 0n) {
    throw new Error("账户余额不足，无法支付gas费用");
  }

  // 获取合约工厂
  const ContractManager = await ethers.getContractFactory("ContractManager");
  
  console.log("正在部署ContractManager合约...");
  
  // 部署合约
  const contractManager = await ContractManager.deploy();
  
  console.log("等待合约部署确认...");
  await contractManager.waitForDeployment();
  
  const contractAddress = await contractManager.getAddress();
  console.log("ContractManager合约已部署到地址:", contractAddress);
  
  // 获取部署交易信息
  const deploymentTx = contractManager.deploymentTransaction();
  if (deploymentTx) {
    console.log("部署交易哈希:", deploymentTx.hash);
    console.log("Gas使用量:", deploymentTx.gasLimit?.toString());
  }

  // 验证合约部署
  console.log("验证合约部署...");
  const owner = await contractManager.owner();
  console.log("合约所有者:", owner);
  
  if (owner.toLowerCase() === deployer.address.toLowerCase()) {
    console.log("✅ 合约部署成功！");
  } else {
    console.log("❌ 合约部署验证失败");
  }

  // 保存部署信息
  const deploymentInfo = {
    network: "baseMainnet",
    contractName: "ContractManager",
    contractAddress: contractAddress,
    deployer: deployer.address,
    deploymentTx: deploymentTx?.hash,
    timestamp: new Date().toISOString(),
    chainId: 8453
  };

  console.log("\n部署信息:");
  console.log(JSON.stringify(deploymentInfo, null, 2));

  // 提示下一步操作
  console.log("\n下一步操作:");
  console.log("1. 在BaseScan上验证合约: https://basescan.org/address/" + contractAddress);
  console.log("2. 使用以下命令验证合约源码:");
  console.log(`npx hardhat verify --network baseMainnet ${contractAddress}`);
  
  return contractAddress;
}

// 如果直接运行此脚本
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("部署失败:", error);
      process.exit(1);
    });
}

module.exports = main;
