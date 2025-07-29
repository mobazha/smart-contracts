const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("RWAMarketplace", function () {
  let marketplace;
  let rwaToken;
  let buyer;
  let seller;
  let tokenIssuer;
  let owner;

  const INITIAL_SUPPLY = ethers.parseEther("1000000"); // 100万代币
  const ORDER_AMOUNT = ethers.parseEther("100"); // 100代币
  const PAYMENT_AMOUNT = ethers.parseEther("1"); // 1 ETH

  beforeEach(async function () {
    [owner, buyer, seller, tokenIssuer] = await ethers.getSigners();

    // 部署合约
    const RWAMarketplace = await ethers.getContractFactory("RWAMarketplace");
    marketplace = await RWAMarketplace.deploy();

    const ExampleRWAToken = await ethers.getContractFactory("ExampleRWAToken");
    rwaToken = await ExampleRWAToken.deploy(
      "Real Estate Token",
      "RET",
      INITIAL_SUPPLY,
      tokenIssuer.address
    );
  });

  describe("基本功能", function () {
    it("应该正确部署合约", async function () {
      expect(await marketplace.orderCounter()).to.equal(0);
      expect(await rwaToken.name()).to.equal("Real Estate Token");
      expect(await rwaToken.symbol()).to.equal("RET");
    });

    it("应该获取正确的RWA Token信息", async function () {
      expect(await rwaToken.getUnderlyingAssetType()).to.equal("Carbon Credit");
      expect(await rwaToken.getUnderlyingAssetId()).to.equal("CARBON_CREDIT_001");
      expect(await rwaToken.getComplianceStatus()).to.equal(true);
      expect(await rwaToken.getIssuer()).to.equal(tokenIssuer.address);
    });
  });

  describe("KYC功能", function () {
    it("应该能够设置KYC状态", async function () {
      await rwaToken.connect(owner).setKYCStatus(buyer.address, true);
      expect(await rwaToken.isKYCVerified(buyer.address)).to.equal(true);

      await rwaToken.connect(owner).setKYCStatus(buyer.address, false);
      expect(await rwaToken.isKYCVerified(buyer.address)).to.equal(false);
    });

    it("未通过KYC的用户不能转移代币", async function () {
      // 给买家铸造代币
      await rwaToken.connect(tokenIssuer).mint(buyer.address, ORDER_AMOUNT);

      // 未通过KYC的用户尝试转移代币应该失败
      await expect(
        rwaToken.connect(buyer).transfer(seller.address, ORDER_AMOUNT)
      ).to.be.revertedWith("KYC verification required");

      // 设置KYC状态后应该成功
      await rwaToken.connect(owner).setKYCStatus(buyer.address, true);
      await expect(
        rwaToken.connect(buyer).transfer(seller.address, ORDER_AMOUNT)
      ).to.not.be.reverted;
    });
  });

  describe("管理员功能", function () {
    it("应该能够设置平台费用", async function () {
      await marketplace.connect(owner).setPlatformFee(50); // 0.5%
      expect(await marketplace.platformFee()).to.equal(50);
    });

    it("应该能够暂停和恢复合约", async function () {
      await marketplace.connect(owner).pause();
      expect(await marketplace.paused()).to.equal(true);

      await marketplace.connect(owner).unpause();
      expect(await marketplace.paused()).to.equal(false);
    });

    it("应该能够设置授权操作员", async function () {
      await marketplace.connect(owner).setAuthorizedOperator(seller.address, true);
      expect(await marketplace.authorizedOperators(seller.address)).to.equal(true);

      await marketplace.connect(owner).setAuthorizedOperator(seller.address, false);
      expect(await marketplace.authorizedOperators(seller.address)).to.equal(false);
    });
  });

  describe("稳定币付款功能", function () {
    let mockUSDT;
    const USDT_AMOUNT = 1000 * 10**6; // 1000 USDT (6位小数)

    beforeEach(async function () {
      // 部署模拟USDT合约
      const MockUSDT = await ethers.getContractFactory("MockUSDT");
      mockUSDT = await MockUSDT.deploy();

      // 给买家铸造USDT
      await mockUSDT.connect(owner).mint(buyer.address, USDT_AMOUNT);
    });

    it("应该能够使用USDT创建订单并付款", async function () {
      const buyerAddress = await buyer.getAddress();
      const sellerAddress = await seller.getAddress();
      const marketplaceAddress = await marketplace.getAddress();
      const rwaTokenAddress = await rwaToken.getAddress();
      const mockUSDTAddress = await mockUSDT.getAddress();

      // 买家授权Marketplace使用USDT
      await mockUSDT.connect(buyer).approve(marketplaceAddress, USDT_AMOUNT);

      // 买家创建订单并付款
      await expect(
        marketplace.connect(buyer).createOrderAndPay(
          sellerAddress,
          rwaTokenAddress,
          mockUSDTAddress, // USDT支付
          buyerAddress,
          ORDER_AMOUNT,
          USDT_AMOUNT,
          { value: 0 } // 不使用ETH
        )
      ).to.emit(marketplace, "OrderCreated");

      // 验证订单创建
      const order = await marketplace.getOrder(1);
      expect(order.buyer).to.equal(buyerAddress);
      expect(order.seller).to.equal(sellerAddress);
      expect(order.rwaTokenAddress).to.equal(rwaTokenAddress);
      expect(order.paymentTokenAddress).to.equal(mockUSDTAddress);
      expect(order.paymentAmount).to.equal(USDT_AMOUNT);
      expect(order.status).to.equal(0); // PAID

      // 验证USDT已转移到Marketplace
      expect(await mockUSDT.balanceOf(marketplaceAddress)).to.equal(USDT_AMOUNT);
    });

    it("应该能够使用USDT完成订单", async function () {
      const buyerAddress = await buyer.getAddress();
      const sellerAddress = await seller.getAddress();
      const marketplaceAddress = await marketplace.getAddress();
      const rwaTokenAddress = await rwaToken.getAddress();
      const mockUSDTAddress = await mockUSDT.getAddress();

      // 设置卖家KYC验证
      await rwaToken.connect(owner).setKYCStatus(sellerAddress, true);

      // 设置Marketplace合约KYC验证
      await rwaToken.connect(owner).setKYCStatus(marketplaceAddress, true);

      // 买家授权Marketplace使用USDT
      await mockUSDT.connect(buyer).approve(marketplaceAddress, USDT_AMOUNT);

      // 买家创建订单并付款
      await marketplace.connect(buyer).createOrderAndPay(
        sellerAddress,
        rwaTokenAddress,
        mockUSDTAddress,
        buyerAddress,
        ORDER_AMOUNT,
        USDT_AMOUNT,
        { value: 0 }
      );

      // 给卖家铸造RWA Token
      await rwaToken.connect(tokenIssuer).mint(sellerAddress, ORDER_AMOUNT);

      // 卖家授权Marketplace使用RWA Token
      await rwaToken.connect(seller).approve(marketplaceAddress, ORDER_AMOUNT);

      // 卖家发货并完成交易
      await expect(marketplace.connect(seller).shipAndComplete(1))
        .to.emit(marketplace, "OrderCompleted");

      // 验证订单状态
      const order = await marketplace.getOrder(1);
      expect(order.status).to.equal(1); // COMPLETED

      // 验证买家收到RWA Token
      expect(await rwaToken.balanceOf(buyerAddress)).to.equal(ORDER_AMOUNT);

      // 验证卖家收到USDT（扣除平台费用）
      const platformFee = await marketplace.platformFee();
      const feeDenominator = 10000n;
      const usdtAmount = BigInt(USDT_AMOUNT);
      const platformFeeAmount = (usdtAmount * BigInt(platformFee)) / feeDenominator;
      const sellerAmount = usdtAmount - platformFeeAmount;
      expect(await mockUSDT.balanceOf(sellerAddress)).to.equal(sellerAmount);
    });

    it("使用USDT付款时不应该接受ETH", async function () {
      const buyerAddress = await buyer.getAddress();
      const sellerAddress = await seller.getAddress();
      const rwaTokenAddress = await rwaToken.getAddress();
      const mockUSDTAddress = await mockUSDT.getAddress();
      const marketplaceAddress = await marketplace.getAddress();

      // 买家授权Marketplace使用USDT
      await mockUSDT.connect(buyer).approve(marketplaceAddress, USDT_AMOUNT);

      // 尝试同时使用USDT和ETH付款应该失败
      await expect(
        marketplace.connect(buyer).createOrderAndPay(
          sellerAddress,
          rwaTokenAddress,
          mockUSDTAddress,
          buyerAddress,
          ORDER_AMOUNT,
          USDT_AMOUNT,
          { value: ethers.parseEther("1") } // 同时发送ETH
        )
      ).to.be.revertedWith("ETH not accepted for token payment");
    });

    it("使用ETH付款时不应该发送代币地址", async function () {
      const buyerAddress = await buyer.getAddress();
      const sellerAddress = await seller.getAddress();
      const rwaTokenAddress = await rwaToken.getAddress();
      const mockUSDTAddress = await mockUSDT.getAddress();

      // 尝试使用ETH但指定代币地址应该失败
      await expect(
        marketplace.connect(buyer).createOrderAndPay(
          sellerAddress,
          rwaTokenAddress,
          mockUSDTAddress, // 指定代币地址
          buyerAddress,
          ORDER_AMOUNT,
          PAYMENT_AMOUNT,
          { value: PAYMENT_AMOUNT } // 同时发送ETH
        )
      ).to.be.revertedWith("ETH not accepted for token payment");
    });
  });
}); 