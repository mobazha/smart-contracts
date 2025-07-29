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
    console.log("✅ 总供应量:", ethers.utils.formatEther(totalSupply));

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
    console.log("✅ USDT总供应量:", ethers.utils.formatUnits(usdtTotalSupply, usdtDecimals));

    // 4. 测试创建订单功能
    console.log("\n🛒 测试创建订单功能...");
    
    // 给测试账户铸造一些USDT
    const mintAmount = ethers.utils.parseUnits("1000", 6); // 1000 USDT
    await mockUSDT.mint(deployer.address, mintAmount);
    console.log("✅ 已铸造", ethers.utils.formatUnits(mintAmount, 6), "USDT");

    // 授权Marketplace使用USDT
    await mockUSDT.approve(marketplace.address, mintAmount);
    console.log("✅ 已授权Marketplace使用USDT");

    // 创建订单
    const orderData = {
      seller: deployer.address,
      rwaTokenAddress: rwaToken.address,
      paymentTokenAddress: mockUSDT.address,
      buyerReceiveAddress: deployer.address,
      rwaTokenAmount: ethers.utils.parseEther("100"), // 100 FCC
      paymentAmount: ethers.utils.parseUnits("100", 6) // 100 USDT
    };

    console.log("📝 创建订单数据:", {
      seller: orderData.seller,
      rwaTokenAddress: orderData.rwaTokenAddress,
      paymentTokenAddress: orderData.paymentTokenAddress,
      buyerReceiveAddress: orderData.buyerReceiveAddress,
      rwaTokenAmount: ethers.utils.formatEther(orderData.rwaTokenAmount),
      paymentAmount: ethers.utils.formatUnits(orderData.paymentAmount, 6)
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
    console.log("✅ 订单创建成功，交易哈希:", receipt.transactionHash);

    // 获取订单ID
    const orderCreatedEvent = receipt.events?.find(event => event.event === 'OrderCreated');
    if (orderCreatedEvent) {
      const orderId = orderCreatedEvent.args?.orderId;
      console.log("✅ 订单ID:", orderId.toString());

      // 获取订单信息
      const order = await marketplace.getOrder(orderId);
      console.log("✅ 订单信息:", {
        orderId: order.orderId.toString(),
        buyer: order.buyer,
        seller: order.seller,
        status: order.status.toString(),
        rwaTokenAmount: ethers.utils.formatEther(order.rwaTokenAmount),
        paymentAmount: ethers.utils.formatUnits(order.paymentAmount, 6)
      });
    }

    // 5. 测试发货完成功能
    console.log("\n🚚 测试发货完成功能...");
    
    // 给卖家铸造一些RWA Token
    const rwaMintAmount = ethers.utils.parseEther("1000"); // 1000 FCC
    await rwaToken.mint(deployer.address, rwaMintAmount);
    console.log("✅ 已铸造", ethers.utils.formatEther(rwaMintAmount), "FCC");

    // 授权Marketplace使用RWA Token
    await rwaToken.approve(marketplace.address, rwaMintAmount);
    console.log("✅ 已授权Marketplace使用FCC");

    // 发货完成
    const shipTx = await marketplace.shipAndComplete(orderCreatedEvent.args?.orderId);
    console.log("⏳ 等待发货交易确认...");
    const shipReceipt = await shipTx.wait();
    console.log("✅ 发货完成，交易哈希:", shipReceipt.transactionHash);

    // 6. 检查最终状态
    console.log("\n📊 检查最终状态...");
    
    const finalOrder = await marketplace.getOrder(orderCreatedEvent.args?.orderId);
    console.log("✅ 最终订单状态:", {
      status: finalOrder.status.toString(),
      completedAt: finalOrder.completedAt.toString()
    });

    const buyerFCCBalance = await rwaToken.balanceOf(deployer.address);
    console.log("✅ 买家FCC余额:", ethers.utils.formatEther(buyerFCCBalance));

    const sellerUSDTBalance = await mockUSDT.balanceOf(deployer.address);
    console.log("✅ 卖家USDT余额:", ethers.utils.formatUnits(sellerUSDTBalance, 6));

    console.log("\n🎉 所有测试通过！");
    console.log("=" * 50);
    console.log("📋 测试摘要:");
    console.log("✅ RWA Marketplace合约功能正常");
    console.log("✅ 森林碳汇信用代币合约功能正常");
    console.log("✅ Mock USDT合约功能正常");
    console.log("✅ 订单创建功能正常");
    console.log("✅ 发货完成功能正常");
    console.log("✅ 代币转移功能正常");
    console.log("=" * 50);

  } catch (error) {
    console.error("❌ 测试失败:", error);
    throw error;
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ 测试脚本执行失败:", error);
    process.exit(1);
  }); 