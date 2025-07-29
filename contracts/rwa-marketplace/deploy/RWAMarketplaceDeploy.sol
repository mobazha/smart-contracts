// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.22;

import "../RWAMarketplace.sol";
import "../ExampleRWAToken.sol";

/**
 * @title RWAMarketplaceDeploy
 * @notice RWA Marketplace合约的部署脚本
 * @dev 用于部署和初始化RWA Marketplace相关合约
 */
contract RWAMarketplaceDeploy {
    
    RWAMarketplace public rwaMarketplace;
    ExampleRWAToken public exampleRWAToken;
    
    address public deployer;
    
    event MarketplaceDeployed(address marketplace);
    event ExampleTokenDeployed(address token);
    
    constructor() {
        deployer = msg.sender;
    }
    
    /**
     * @notice 部署RWA Marketplace合约
     * @return 部署的合约地址
     */
    function deployMarketplace() external returns (address) {
        require(msg.sender == deployer, "Only deployer can deploy");
        
        rwaMarketplace = new RWAMarketplace();
        
        emit MarketplaceDeployed(address(rwaMarketplace));
        
        return address(rwaMarketplace);
    }
    
    /**
     * @notice 部署示例RWA Token合约
     * @param name 代币名称
     * @param symbol 代币符号
     * @param initialSupply 初始供应量
     * @param issuer 发行人地址
     * @return 部署的合约地址
     */
    function deployExampleToken(
        string memory name,
        string memory symbol,
        uint256 initialSupply,
        address issuer
    ) external returns (address) {
        require(msg.sender == deployer, "Only deployer can deploy");
        
        exampleRWAToken = new ExampleRWAToken(
            name,
            symbol,
            initialSupply,
            issuer
        );
        
        emit ExampleTokenDeployed(address(exampleRWAToken));
        
        return address(exampleRWAToken);
    }
    
    /**
     * @notice 批量部署RWA Token合约
     * @param names 代币名称数组
     * @param symbols 代币符号数组
     * @param initialSupplies 初始供应量数组
     * @param issuers 发行人地址数组
     * @return 部署的合约地址数组
     */
    function deployMultipleTokens(
        string[] memory names,
        string[] memory symbols,
        uint256[] memory initialSupplies,
        address[] memory issuers
    ) external returns (address[] memory) {
        require(msg.sender == deployer, "Only deployer can deploy");
        require(
            names.length == symbols.length &&
            symbols.length == initialSupplies.length &&
            initialSupplies.length == issuers.length,
            "Array lengths must match"
        );
        
        address[] memory deployedTokens = new address[](names.length);
        
        for (uint256 i = 0; i < names.length; i++) {
            ExampleRWAToken token = new ExampleRWAToken(
                names[i],
                symbols[i],
                initialSupplies[i],
                issuers[i]
            );
            
            deployedTokens[i] = address(token);
            
            emit ExampleTokenDeployed(address(token));
        }
        
        return deployedTokens;
    }
    
    /**
     * @notice 获取已部署的合约地址
     * @return marketplace RWA Marketplace合约地址
     * @return exampleToken 示例RWA Token合约地址
     */
    function getDeployedContracts() external view returns (address marketplace, address exampleToken) {
        return (address(rwaMarketplace), address(exampleRWAToken));
    }
    
    /**
     * @notice 设置RWA Marketplace的授权操作员
     * @param operator 操作员地址
     * @param authorized 是否授权
     */
    function setMarketplaceOperator(address operator, bool authorized) external {
        require(msg.sender == deployer, "Only deployer can set operator");
        require(address(rwaMarketplace) != address(0), "Marketplace not deployed");
        
        rwaMarketplace.setAuthorizedOperator(operator, authorized);
    }
    
    /**
     * @notice 设置RWA Marketplace的平台费用
     * @param newFee 新费用（基点）
     */
    function setMarketplaceFee(uint256 newFee) external {
        require(msg.sender == deployer, "Only deployer can set fee");
        require(address(rwaMarketplace) != address(0), "Marketplace not deployed");
        
        rwaMarketplace.setPlatformFee(newFee);
    }
    
    /**
     * @notice 转移RWA Marketplace的所有权
     * @param newOwner 新所有者地址
     */
    function transferMarketplaceOwnership(address newOwner) external {
        require(msg.sender == deployer, "Only deployer can transfer ownership");
        require(address(rwaMarketplace) != address(0), "Marketplace not deployed");
        
        rwaMarketplace.transferOwnership(newOwner);
    }
} 