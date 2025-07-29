// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "../../rwa-marketplace/RWAMarketplace.sol";
import "../../registry/ContractManager.sol";

/**
 * @title RwaMarketplaceDeploy
 * @notice RWA Marketplace合约部署脚本
 * @dev 部署后自动注册到ContractManager
 */
contract RwaMarketplaceDeploy {
    
    event MarketplaceDeployed(address indexed marketplace, address indexed contractManager);
    
    /**
     * @notice 部署RWA Marketplace合约
     * @param contractManager ContractManager合约地址
     * @return marketplace 部署的Marketplace合约地址
     */
    function deployMarketplace(
        address contractManager
    ) external returns (address marketplace) {
        require(contractManager != address(0), "Invalid contract manager");
        
        // 部署Marketplace合约
        marketplace = address(new RWAMarketplace());
        
        // 注册到ContractManager
        ContractManager(contractManager).addVersion("RwaMarketplace", "v1.0", ContractManager.Status.PRODUCTION, marketplace);
        
        emit MarketplaceDeployed(marketplace, contractManager);
        
        return marketplace;
    }
} 