// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.22;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "../RWAMarketplace.sol";
import "../ExampleRWAToken.sol";
import "../IRWAToken.sol";

/**
 * @title RWAMarketplaceTest
 * @notice 简化的RWA Marketplace合约测试文件
 * @dev 测试核心功能：创建订单并付款、发货完成、取消订单
 */
contract RWAMarketplaceTest {
    
    RWAMarketplace public marketplace;
    ExampleRWAToken public rwaToken;
    
    address public buyer;
    address public seller;
    address public tokenIssuer;
    
    uint256 public constant INITIAL_SUPPLY = 1000000 * 10**18; // 100万代币
    uint256 public constant ORDER_AMOUNT = 100 * 10**18; // 100代币
    uint256 public constant PAYMENT_AMOUNT = 1 ether; // 1 ETH
    
    function beforeEach() public {
        // 设置测试账户
        buyer = address(0x1);
        seller = address(0x2);
        tokenIssuer = address(0x3);
        
        // 部署合约
        marketplace = new RWAMarketplace();
        rwaToken = new ExampleRWAToken(
            "Real Estate Token",
            "RET",
            INITIAL_SUPPLY,
            tokenIssuer
        );
    }
    
    function testCreateOrderAndPay() public {
        // 生成唯一的订单ID
        bytes32 orderId = keccak256(abi.encodePacked("ORDER_001"));
        
        // 买家创建订单并付款
        marketplace.createOrderAndPay{value: PAYMENT_AMOUNT}(
            orderId,
            buyer,
            seller,
            address(rwaToken),
            address(0), // ETH支付
            buyer,
            ORDER_AMOUNT,
            PAYMENT_AMOUNT
        );
        
        // 获取订单信息
        RWAMarketplace.Order memory order = marketplace.getOrder(orderId);
        require(order.buyer == buyer, "Buyer should match");
        require(order.seller == seller, "Seller should match");
        require(order.rwaTokenAddress == address(rwaToken), "RWA Token address should match");
        require(order.paymentAmount == PAYMENT_AMOUNT, "Payment amount should match");
        require(uint256(order.status) == uint256(RWAMarketplace.OrderStatus.PAID), "Order status should be PAID");
    }
    
    function testShipAndComplete() public {
        // 生成唯一的订单ID
        bytes32 orderId = keccak256(abi.encodePacked("ORDER_002"));
        
        // 买家创建订单并付款
        marketplace.createOrderAndPay{value: PAYMENT_AMOUNT}(
            orderId,
            buyer, // 买家地址
            seller,
            address(rwaToken),
            address(0), // ETH支付
            buyer,
            ORDER_AMOUNT,
            PAYMENT_AMOUNT
        );
        
        // 给卖家铸造RWA Token
        rwaToken.mint(seller, ORDER_AMOUNT);
        
        // 卖家授权Marketplace使用代币
        rwaToken.approve(address(marketplace), ORDER_AMOUNT);
        
        // 卖家发货并完成交易
        marketplace.shipAndComplete(orderId, seller);
        
        // 验证订单状态
        RWAMarketplace.Order memory order = marketplace.getOrder(orderId);
        require(uint256(order.status) == uint256(RWAMarketplace.OrderStatus.COMPLETED), "Order status should be COMPLETED");
        
        // 验证买家收到代币
        require(rwaToken.balanceOf(buyer) == ORDER_AMOUNT, "Buyer should receive tokens");
    }
    
    function testCancelOrder() public {
        // 生成唯一的订单ID
        bytes32 orderId = keccak256(abi.encodePacked("ORDER_003"));
        
        // 买家创建订单并付款
        marketplace.createOrderAndPay{value: PAYMENT_AMOUNT}(
            orderId,
            buyer,
            seller,
            address(rwaToken),
            address(0), // ETH支付
            buyer,
            ORDER_AMOUNT,
            PAYMENT_AMOUNT
        );
        
        // 买家取消订单
        marketplace.cancelOrder(orderId);
        
        // 验证订单状态
        RWAMarketplace.Order memory order = marketplace.getOrder(orderId);
        require(uint256(order.status) == uint256(RWAMarketplace.OrderStatus.CANCELLED), "Order status should be CANCELLED");
    }
    

    
    function testGetBuyerOrders() public {
        // 生成唯一的订单ID
        bytes32 orderId1 = keccak256(abi.encodePacked("ORDER_004"));
        bytes32 orderId2 = keccak256(abi.encodePacked("ORDER_005"));
        
        // 买家创建多个订单
        marketplace.createOrderAndPay{value: PAYMENT_AMOUNT}(
            orderId1,
            buyer,
            seller,
            address(rwaToken),
            address(0),
            buyer,
            ORDER_AMOUNT,
            PAYMENT_AMOUNT
        );
        
        marketplace.createOrderAndPay{value: PAYMENT_AMOUNT}(
            orderId2,
            buyer,
            seller,
            address(rwaToken),
            address(0),
            buyer,
            ORDER_AMOUNT,
            PAYMENT_AMOUNT
        );
        
        // 获取买家订单
        bytes32[] memory buyerOrders = marketplace.getBuyerOrders(buyer);
        require(buyerOrders.length == 2, "Buyer should have 2 orders");
        require(buyerOrders[0] == orderId1, "First order ID should match");
        require(buyerOrders[1] == orderId2, "Second order ID should match");
    }
    
    function testGetSellerOrders() public {
        // 生成唯一的订单ID
        bytes32 orderId = keccak256(abi.encodePacked("ORDER_006"));
        
        // 买家创建订单
        marketplace.createOrderAndPay{value: PAYMENT_AMOUNT}(
            orderId,
            buyer,
            seller,
            address(rwaToken),
            address(0),
            buyer,
            ORDER_AMOUNT,
            PAYMENT_AMOUNT
        );
        
        // 获取卖家订单
        bytes32[] memory sellerOrders = marketplace.getSellerOrders(seller);
        require(sellerOrders.length == 1, "Seller should have 1 order");
        require(sellerOrders[0] == orderId, "Order ID should match");
    }
    
    function testPlatformFee() public {
        // 设置平台费用为0.5%
        marketplace.setPlatformFee(50); // 50 = 0.5%
        
        // 验证平台费用
        require(marketplace.platformFee() == 50, "Platform fee should be 50");
    }
    
    function testPauseUnpause() public {
        // 暂停合约
        marketplace.pause();
        require(marketplace.paused() == true, "Contract should be paused");
        
        // 恢复合约
        marketplace.unpause();
        require(marketplace.paused() == false, "Contract should not be paused");
    }
    
    function testOrderCounter() public {
        // 初始订单计数器
        require(marketplace.orderCounter() == 0, "Initial order counter should be 0");
        
        // 生成唯一的订单ID
        bytes32 orderId = keccak256(abi.encodePacked("ORDER_007"));
        
        // 创建订单
        marketplace.createOrderAndPay{value: PAYMENT_AMOUNT}(
            orderId,
            buyer,
            seller,
            address(rwaToken),
            address(0),
            buyer,
            ORDER_AMOUNT,
            PAYMENT_AMOUNT
        );
        
        // 验证订单计数器
        require(marketplace.orderCounter() == 1, "Order counter should be 1");
    }
    
    function testRWAInfo() public {
        // 获取RWA Token基本信息
        string memory assetType = rwaToken.getUnderlyingAssetType();
        string memory assetId = rwaToken.getUnderlyingAssetId();
        bool complianceStatus = rwaToken.getComplianceStatus();
        address issuer = rwaToken.getIssuer();
        
        // 验证基本信息
        require(keccak256(abi.encodePacked(assetType)) == keccak256(abi.encodePacked("Carbon Credit")), "Asset type should be Carbon Credit");
        require(keccak256(abi.encodePacked(assetId)) == keccak256(abi.encodePacked("CARBON_CREDIT_001")), "Asset ID should be CARBON_CREDIT_001");
        require(complianceStatus == true, "Compliance status should be true");
        require(issuer == tokenIssuer, "Issuer should match");
    }
} 