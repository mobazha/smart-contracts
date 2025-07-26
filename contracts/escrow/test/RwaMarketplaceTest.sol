// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "../RwaMarketplace.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

// 模拟预言机合约
contract MockPriceOracle {
    mapping(address => mapping(address => uint256)) public prices;
    
    function setPrice(address rwaToken, address payToken, uint256 price) external {
        prices[rwaToken][payToken] = price;
    }
    
    function getPrice(address rwaToken, address payToken) external view returns (uint256) {
        return prices[rwaToken][payToken];
    }
}

// 模拟RWA Token
contract MockRwaToken is ERC20 {
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {
        _mint(msg.sender, 1000000 * 10**decimals());
    }
}

// 模拟支付币种
contract MockPayToken is ERC20 {
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {
        _mint(msg.sender, 1000000 * 10**decimals());
    }
}

contract RwaMarketplaceTest is Test {
    RwaMarketplace public marketplace;
    MockPriceOracle public priceOracle;
    MockRwaToken public rwaToken;
    MockPayToken public payToken;
    
    address public platform = address(0x123);
    address public seller = address(0x456);
    address public buyer = address(0x789);
    
    function setUp() public {
        // 部署预言机
        priceOracle = new MockPriceOracle();
        
        // 部署Marketplace
        marketplace = new RwaMarketplace(address(priceOracle), platform);
        
        // 部署Token
        rwaToken = new MockRwaToken("RWA Token", "RWA");
        payToken = new MockPayToken("USDT", "USDT");
        
        // 设置价格
        priceOracle.setPrice(address(rwaToken), address(payToken), 100 * 10**18); // 1 RWA = 100 USDT
        
        // 设置白名单
        marketplace.setSellerWhitelist(seller, true);
        marketplace.setRwaTokenWhitelist(address(rwaToken), true);
        marketplace.setPayTokenWhitelist(address(payToken), true);
        
        // 给测试账户分配Token
        rwaToken.transfer(seller, 1000 * 10**18);
        payToken.transfer(buyer, 10000 * 10**18);
    }
    
    function testDeploy() public {
        assertEq(address(marketplace.priceOracle()), address(priceOracle));
        assertEq(marketplace.platform(), platform);
        assertEq(marketplace.feeBps(), 30);
        assertEq(marketplace.kycRequired(), true);
        assertEq(marketplace.paused(), false);
    }
    
    function testListRwaToken() public {
        vm.startPrank(seller);
        
        address[] memory payTokens = new address[](1);
        payTokens[0] = address(payToken);
        
        marketplace.listRwaToken(address(rwaToken), payTokens);
        
        assertTrue(marketplace.sales(seller, address(rwaToken)).allowedPayTokens(address(payToken)));
        vm.stopPrank();
    }
    
    function testDepositRwaToken() public {
        vm.startPrank(seller);
        
        uint256 amount = 100 * 10**18;
        rwaToken.approve(address(marketplace), amount);
        marketplace.depositRwaToken(address(rwaToken), amount);
        
        assertEq(marketplace.sales(seller, address(rwaToken)).available, amount);
        vm.stopPrank();
    }
    
    function testBuyRwaToken() public {
        // 卖家上架和充值
        vm.startPrank(seller);
        address[] memory payTokens = new address[](1);
        payTokens[0] = address(payToken);
        marketplace.listRwaToken(address(rwaToken), payTokens);
        
        uint256 depositAmount = 100 * 10**18;
        rwaToken.approve(address(marketplace), depositAmount);
        marketplace.depositRwaToken(address(rwaToken), depositAmount);
        vm.stopPrank();
        
        // 设置买家KYC
        vm.prank(platform);
        marketplace.setKycStatus(buyer, true);
        
        // 买家购买
        vm.startPrank(buyer);
        uint256 buyAmount = 10 * 10**18; // 10 RWA
        uint256 expectedPayAmount = buyAmount * 100 * 10**18 / 10**18; // 1000 USDT
        uint256 expectedFee = expectedPayAmount * 30 / 10000; // 3 USDT
        
        payToken.approve(address(marketplace), expectedPayAmount);
        marketplace.buyRwaToken(seller, address(rwaToken), buyAmount, address(payToken));
        
        // 验证余额变化
        assertEq(rwaToken.balanceOf(buyer), buyAmount);
        assertEq(marketplace.sellerPayTokenBalances(seller, address(payToken)), expectedPayAmount - expectedFee);
        assertEq(marketplace.sellerPayTokenBalances(platform, address(payToken)), expectedFee);
        vm.stopPrank();
    }
    
    function testWithdrawPayToken() public {
        // 先完成一笔交易
        testBuyRwaToken();
        
        // 卖家提现
        vm.startPrank(seller);
        uint256 withdrawAmount = 500 * 10**18;
        uint256 balanceBefore = payToken.balanceOf(seller);
        marketplace.withdrawPayToken(address(payToken), withdrawAmount);
        uint256 balanceAfter = payToken.balanceOf(seller);
        
        assertEq(balanceAfter - balanceBefore, withdrawAmount);
        vm.stopPrank();
    }
    
    function testWithdrawUnsoldRwaToken() public {
        // 卖家充值
        vm.startPrank(seller);
        uint256 depositAmount = 100 * 10**18;
        rwaToken.approve(address(marketplace), depositAmount);
        marketplace.depositRwaToken(address(rwaToken), depositAmount);
        
        // 撤回未售Token
        uint256 withdrawAmount = 50 * 10**18;
        uint256 balanceBefore = rwaToken.balanceOf(seller);
        marketplace.withdrawUnsoldRwaToken(address(rwaToken), withdrawAmount);
        uint256 balanceAfter = rwaToken.balanceOf(seller);
        
        assertEq(balanceAfter - balanceBefore, withdrawAmount);
        assertEq(marketplace.sales(seller, address(rwaToken)).available, depositAmount - withdrawAmount);
        vm.stopPrank();
    }
    
    function testKycRequired() public {
        // 关闭KYC要求
        vm.prank(platform);
        marketplace.setKycRequired(false);
        
        // 卖家上架和充值
        vm.startPrank(seller);
        address[] memory payTokens = new address[](1);
        payTokens[0] = address(payToken);
        marketplace.listRwaToken(address(rwaToken), payTokens);
        
        uint256 depositAmount = 100 * 10**18;
        rwaToken.approve(address(marketplace), depositAmount);
        marketplace.depositRwaToken(address(rwaToken), depositAmount);
        vm.stopPrank();
        
        // 买家无需KYC即可购买
        vm.startPrank(buyer);
        uint256 buyAmount = 10 * 10**18;
        uint256 expectedPayAmount = buyAmount * 100 * 10**18 / 10**18;
        
        payToken.approve(address(marketplace), expectedPayAmount);
        marketplace.buyRwaToken(seller, address(rwaToken), buyAmount, address(payToken));
        
        assertEq(rwaToken.balanceOf(buyer), buyAmount);
        vm.stopPrank();
    }
    
    function testPause() public {
        // 暂停合约
        vm.prank(platform);
        marketplace.setPaused(true);
        
        // 验证无法执行操作
        vm.startPrank(seller);
        address[] memory payTokens = new address[](1);
        payTokens[0] = address(payToken);
        
        vm.expectRevert("Paused");
        marketplace.listRwaToken(address(rwaToken), payTokens);
        vm.stopPrank();
    }
    
    function testFeeChange() public {
        // 修改手续费
        vm.prank(platform);
        marketplace.setFeeBps(50); // 0.5%
        
        assertEq(marketplace.feeBps(), 50);
    }
    
    function testFailUnauthorized() public {
        // 测试未授权操作
        vm.expectRevert("Not platform");
        marketplace.setFeeBps(100);
    }
    
    function testFailNotWhitelisted() public {
        // 测试未在白名单的操作
        address unauthorizedSeller = address(0x999);
        
        vm.startPrank(unauthorizedSeller);
        address[] memory payTokens = new address[](1);
        payTokens[0] = address(payToken);
        
        vm.expectRevert("Seller not whitelisted");
        marketplace.listRwaToken(address(rwaToken), payTokens);
        vm.stopPrank();
    }
} 