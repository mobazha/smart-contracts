const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("🧪 开始测试部署的合约...");

  // 读取部署信息
  const deploymentPath = path.join(__dirname, "deployment-sepolia.json");
  if (!fs.existsSync(deploymentPath)) {
    throw new Error("❌ 未找到部署信息文件，请先运行部署脚本");
  }

  const deploymentInfo = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));
  console.log("📋 部署信息:", deploymentInfo);

  const [deployer] = await ethers.getSigners();
  console.log("🧪 测试账户:", deployer.address);

  try {
    // 1. 测试RWA Marketplace合约
    console.log("\n📦 测试RWA Marketplace合约...");
    const marketplace = await ethers.getContractAt("RWAMarketplace", deploymentInfo.contracts.rwaMarketplace.address);
    
    // 检查平台费用
    const platformFee = await marketplace.platformFee();
    console.log("✅ 平台费用:", platformFee.toString());
    
    // 检查订单计数器
    const orderCounter = await marketplace.orderCounter();
    console.log("✅ 订单计数器:", orderCounter.toString());

    // 2. 测试RWA Token合约
    console.log("\n🌲 测试森林碳汇信用代币合约...");
    const rwaToken = await ethers.getContractAt("ExampleRWAToken", deploymentInfo.contracts.exampleRWAToken.address);
    
    // 检查代币信息
    const tokenName = await rwaToken.name();
    const tokenSymbol = await rwaToken.symbol();
    const totalSupply = await rwaToken.totalSupply();
    console.log("✅ 代币名称:", tokenName);
    console.log("✅ 代币符号:", tokenSymbol);
    console.log("✅ 总供应量:", ethers.formatEther(totalSupply));

    // 检查RWA特有信息
    const assetType = await rwaToken.getUnderlyingAssetType();
    const assetId = await rwaToken.getUnderlyingAssetId();
    const complianceStatus = await rwaToken.getComplianceStatus();
    const kycRequired = await rwaToken.getKYCRequired();
    console.log("✅ 资产类型:", assetType);
    console.log("✅ 资产ID:", assetId);
    console.log("✅ 合规状态:", complianceStatus);
    console.log("✅ KYC要求:", kycRequired);

    // 3. 测试Mock USDT合约
    console.log("\n💵 测试Mock USDT合约...");
    const mockUSDT = await ethers.getContractAt("MockUSDT", deploymentInfo.contracts.mockUSDT.address);
    
    const usdtName = await mockUSDT.name();
    const usdtSymbol = await mockUSDT.symbol();
    const usdtDecimals = await mockUSDT.decimals();
    const usdtTotalSupply = await mockUSDT.totalSupply();
    console.log("✅ USDT名称:", usdtName);
    console.log("✅ USDT符号:", usdtSymbol);
    console.log("✅ USDT小数位:", usdtDecimals);
    console.log("✅ USDT总供应量:", ethers.formatUnits(usdtTotalSupply, usdtDecimals));

    // 4. 测试创建订单功能
    console.log("\n🛒 测试创建订单功能...");
    
    // 给测试账户铸造一些USDT
    const mintAmount = ethers.parseUnits("1000", 6); // 1000 USDT
    await mockUSDT.mint(deployer.address, mintAmount);
    console.log("✅ 已铸造", ethers.formatUnits(mintAmount, 6), "USDT");

    // 授权Marketplace使用USDT
    const marketplaceAddress = await marketplace.getAddress();
    await mockUSDT.approve(marketplaceAddress, mintAmount);
    console.log("✅ 已授权Marketplace使用USDT");

    // 创建订单
    const orderData = {
      seller: deployer.address,
      rwaTokenAddress: await rwaToken.getAddress(),
      paymentTokenAddress: await mockUSDT.getAddress(),
      buyerReceiveAddress: deployer.address,
      rwaTokenAmount: ethers.parseEther("100"), // 100 FCC
      paymentAmount: ethers.parseUnits("100", 6) // 100 USDT
    };

    console.log("📝 创建订单数据:", {
      seller: orderData.seller,
      rwaTokenAddress: orderData.rwaTokenAddress,
      paymentTokenAddress: orderData.paymentTokenAddress,
      buyerReceiveAddress: orderData.buyerReceiveAddress,
      rwaTokenAmount: ethers.formatEther(orderData.rwaTokenAmount),
      paymentAmount: ethers.formatUnits(orderData.paymentAmount, 6)
    });

    const tx = await marketplace.createOrderAndPay(
      orderData.seller,
      orderData.rwaTokenAddress,
      orderData.paymentTokenAddress,
      orderData.buyerReceiveAddress,
      orderData.rwaTokenAmount,
      orderData.paymentAmount
    );

    console.log("⏳ 等待交易确认...");
    const receipt = await tx.wait();
    console.log("✅ 订单创建成功！交易哈希:", receipt.hash);

    // 5. 检查订单状态
    console.log("\n📋 检查订单状态...");
    const order = await marketplace.orders(0); // 第一个订单
    console.log("✅ 订单信息:", {
      seller: order.seller,
      rwaTokenAddress: order.rwaTokenAddress,
      paymentTokenAddress: order.paymentTokenAddress,
      buyerReceiveAddress: order.buyerReceiveAddress,
      rwaTokenAmount: ethers.formatEther(order.rwaTokenAmount),
      paymentAmount: ethers.formatUnits(order.paymentAmount, 6),
      status: order.status,
      createdAt: new Date(order.createdAt.toNumber() * 1000).toISOString()
    });

    // 6. 测试订单完成功能
    console.log("\n✅ 测试订单完成功能...");
    const completeTx = await marketplace.completeOrder(0);
    await completeTx.wait();
    console.log("✅ 订单完成成功！");

    // 7. 检查最终状态
    console.log("\n📊 最终状态检查...");
    const finalOrder = await marketplace.orders(0);
    console.log("✅ 订单最终状态:", finalOrder.status);
    
    const finalBalance = await mockUSDT.balanceOf(deployer.address);
    console.log("✅ 最终USDT余额:", ethers.formatUnits(finalBalance, 6));

    console.log("\n🎉 所有测试通过！");

  } catch (error) {
    console.error("❌ 测试失败:", error);
    throw error;
  }
}

// 错误处理
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ 测试脚本执行失败:", error);
    process.exit(1);
  }); 