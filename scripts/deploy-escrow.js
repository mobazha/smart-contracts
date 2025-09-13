const { ethers } = require("hardhat");

async function main() {
  console.log("开始部署 Escrow 合约...");

  // 获取部署者账户
  const [deployer] = await ethers.getSigners();
  console.log("部署者地址:", deployer.address);
  console.log("部署者余额:", ethers.formatEther(await deployer.provider.getBalance(deployer.address)), "ETH");

  // 部署 Escrow 合约
  const Escrow = await ethers.getContractFactory("Escrow");
  const escrow = await Escrow.deploy();
  
  await escrow.waitForDeployment();
  const escrowAddress = await escrow.getAddress();

  console.log("Escrow 合约已部署到:", escrowAddress);
  console.log("部署交易哈希:", escrow.deploymentTransaction().hash);

  // 保存部署信息
  const deploymentInfo = {
    network: "sepolia",
    deployer: deployer.address,
    deploymentTime: new Date().toISOString(),
    contracts: {
      escrow: {
        address: escrowAddress,
        name: "Escrow",
        description: "Mobazha Escrow合约 - 用于托管ETH和ERC20代币"
      }
    }
  };

  console.log("\n部署信息:");
  console.log(JSON.stringify(deploymentInfo, null, 2));

  // 验证合约部署
  console.log("\n验证合约部署...");
  const transactionCount = await escrow.transactionCount();
  console.log("当前交易数量:", transactionCount.toString());
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("部署失败:", error);
    process.exit(1);
  });
