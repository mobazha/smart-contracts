// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "../RwaMarketplace.sol";
import "../../registry/ContractManager.sol";

/**
 * @title RwaMarketplaceDeploy
 * @notice RWA Marketplace合约部署脚本
 * @dev 部署后自动注册到ContractManager
 */
contract RwaMarketplaceDeploy {
    
    event MarketplaceDeployed(address indexed marketplace, address indexed priceOracle, address indexed platform);
    
    /**
     * @notice 部署RWA Marketplace合约
     * @param priceOracle 预言机合约地址
     * @param platform 平台管理地址（建议为多签钱包）
     * @param contractManager ContractManager合约地址
     * @return marketplace 部署的Marketplace合约地址
     */
    function deployMarketplace(
        address priceOracle,
        address platform,
        address contractManager
    ) external returns (address marketplace) {
        require(priceOracle != address(0), "Invalid price oracle");
        require(platform != address(0), "Invalid platform address");
        require(contractManager != address(0), "Invalid contract manager");
        
        // 部署Marketplace合约
        marketplace = address(new RwaMarketplace(priceOracle, platform));
        
        // 注册到ContractManager
        ContractManager(contractManager).setContract("RwaMarketplace", marketplace);
        
        emit MarketplaceDeployed(marketplace, priceOracle, platform);
        
        return marketplace;
    }
    
    /**
     * @notice 批量设置初始白名单
     * @param marketplace Marketplace合约地址
     * @param sellers 卖家地址数组
     * @param rwaTokens RWA Token地址数组
     * @param payTokens 支付币种地址数组
     */
    function setInitialWhitelist(
        address marketplace,
        address[] calldata sellers,
        address[] calldata rwaTokens,
        address[] calldata payTokens
    ) external {
        RwaMarketplace market = RwaMarketplace(marketplace);
        
        // 设置卖家白名单
        for (uint i = 0; i < sellers.length; i++) {
            market.setSellerWhitelist(sellers[i], true);
        }
        
        // 设置RWA Token白名单
        for (uint i = 0; i < rwaTokens.length; i++) {
            market.setRwaTokenWhitelist(rwaTokens[i], true);
        }
        
        // 设置支付币种白名单
        for (uint i = 0; i < payTokens.length; i++) {
            market.setPayTokenWhitelist(payTokens[i], true);
        }
    }
} 