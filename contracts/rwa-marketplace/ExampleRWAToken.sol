// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.22;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./IRWAToken.sol";

/**
 * @title ExampleRWAToken
 * @notice 森林碳汇信用代币合约，实现IRWAToken接口
 * @dev 基于可持续森林管理的碳汇信用代币，每个代币代表1吨CO2当量的碳汇
 */
contract ExampleRWAToken is ERC20, Ownable, Pausable, IRWAToken {
    
    // RWA Token基本属性
    string private _underlyingAssetType;
    string private _underlyingAssetId;
    bool private _complianceStatus;
    address private _issuer;

    // KYC验证映射
    mapping(address => bool) private _kycVerified;

    // 修饰符
    modifier onlyKYCVerified() {
        require(_kycVerified[msg.sender], "KYC verification required");
        _;
    }

    modifier onlyIssuer() {
        require(msg.sender == _issuer, "Only issuer can call this function");
        _;
    }

    /**
     * @notice 构造函数
     * @param name_ 代币名称 (例如: "Forest Carbon Credit Token")
     * @param symbol_ 代币符号 (例如: "FCC")
     * @param initialSupply 初始供应量 (500,000 tokens)
     * @param issuer_ 发行人地址 (绿色森林基金)
     */
    constructor(
        string memory name_,
        string memory symbol_,
        uint256 initialSupply,
        address issuer_
    ) ERC20(name_, symbol_) {
        require(issuer_ != address(0), "Invalid issuer address");
        
        _issuer = issuer_;
        _underlyingAssetType = "Carbon Credit";
        _underlyingAssetId = "CARBON_CREDIT_001";
        _complianceStatus = true;

        // 给发行人铸造初始代币
        _mint(issuer_, initialSupply);
    }

    /**
     * @notice 铸造代币
     * @param to 接收地址
     * @param amount 数量
     */
    function mint(address to, uint256 amount) external onlyIssuer {
        _mint(to, amount);
    }

    /**
     * @notice 销毁代币
     * @param from 发送地址
     * @param amount 数量
     */
    function burn(address from, uint256 amount) external onlyIssuer {
        _burn(from, amount);
    }

    /**
     * @notice 设置KYC验证状态
     * @param account 账户地址
     * @param verified 验证状态
     */
    function setKYCStatus(address account, bool verified) external onlyOwner {
        _kycVerified[account] = verified;
        emit KYCStatusUpdated(account, verified);
    }

    /**
     * @notice 设置合规状态
     * @param status 合规状态
     */
    function setComplianceStatus(bool status) external onlyOwner {
        _complianceStatus = status;
        emit RWAComplianceUpdated(status);
    }

    /**
     * @notice 暂停代币转移
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @notice 恢复代币转移
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    // 重写ERC20的transfer函数，添加KYC检查
    function transfer(address to, uint256 amount) 
        public 
        override(ERC20, IERC20) 
        whenNotPaused 
        onlyKYCVerified 
        returns (bool) 
    {
        return super.transfer(to, amount);
    }

    // 重写ERC20的transferFrom函数，添加KYC检查
    function transferFrom(address from, address to, uint256 amount) 
        public 
        override(ERC20, IERC20) 
        whenNotPaused 
        onlyKYCVerified 
        returns (bool) 
    {
        return super.transferFrom(from, to, amount);
    }

    // IRWAToken接口实现

    function getUnderlyingAssetType() external view override returns (string memory) {
        return _underlyingAssetType;
    }

    function getUnderlyingAssetId() external view override returns (string memory) {
        return _underlyingAssetId;
    }

    function getIssuer() external view override returns (address) {
        return _issuer;
    }

    function getComplianceStatus() external view override returns (bool) {
        return _complianceStatus;
    }

    function isKYCVerified(address account) external view override returns (bool) {
        return _kycVerified[account];
    }
} 