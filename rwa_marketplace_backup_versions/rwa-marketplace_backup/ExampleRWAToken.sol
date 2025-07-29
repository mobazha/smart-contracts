// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.22;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./IRWAToken.sol";

/**
 * @title ExampleRWAToken
 * @notice 示例RWA Token合约，实现IRWAToken接口
 * @dev 用于演示RWA Token的功能和特性
 */
contract ExampleRWAToken is ERC20, Ownable, Pausable, IRWAToken {
    
    // RWA Token特有属性
    string private _underlyingAssetType;
    string private _underlyingAssetId;
    bool private _complianceStatus;
    bool private _kycRequired;
    bool private _isRedeemable;
    uint256 private _minimumTradeUnit;
    uint256 private _maximumTradeUnit;
    address private _issuer;
    string private _regulatoryInfo;
    string private _valuationMethod;
    bool private _auditStatus;
    bool private _insuranceStatus;
    string private _custodian;
    bool private _liquidityStatus;
    uint8 private _riskLevel;
    uint256 private _yieldRate;
    uint256 private _maturityDate;
    string private _geographicLocation;
    string private _industryCategory;
    bytes32 private _documentHash;
    string private _metadataURI;

    // KYC验证映射
    mapping(address => bool) private _kycVerified;

    // 修饰符
    modifier onlyKYCVerified() {
        if (_kycRequired) {
            require(_kycVerified[msg.sender], "KYC verification required");
        }
        _;
    }

    modifier onlyIssuer() {
        require(msg.sender == _issuer, "Only issuer can call this function");
        _;
    }

    /**
     * @notice 构造函数
     * @param name_ 代币名称
     * @param symbol_ 代币符号
     * @param initialSupply 初始供应量
     * @param issuer_ 发行人地址
     */
    constructor(
        string memory name_,
        string memory symbol_,
        uint256 initialSupply,
        address issuer_
    ) ERC20(name_, symbol_) {
        require(issuer_ != address(0), "Invalid issuer address");
        
        _issuer = issuer_;
        _underlyingAssetType = "Real Estate";
        _underlyingAssetId = "RE001";
        _complianceStatus = true;
        _kycRequired = true;
        _isRedeemable = true;
        _minimumTradeUnit = 1 * 10**decimals();
        _maximumTradeUnit = 1000000 * 10**decimals();
        _regulatoryInfo = "SEC Registered";
        _valuationMethod = "Appraisal Based";
        _auditStatus = true;
        _insuranceStatus = true;
        _custodian = "Trust Company XYZ";
        _liquidityStatus = true;
        _riskLevel = 3; // 中等风险
        _yieldRate = 500; // 5% 年化收益率
        _maturityDate = block.timestamp + 365 days;
        _geographicLocation = "New York, USA";
        _industryCategory = "Commercial Real Estate";
        _documentHash = keccak256(abi.encodePacked("RWA_DOCUMENT_001"));
        _metadataURI = "https://api.example.com/rwa/metadata/RE001";

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
     * @notice 设置审计状态
     * @param status 审计状态
     */
    function setAuditStatus(bool status) external onlyOwner {
        _auditStatus = status;
        emit AuditCompleted(status);
    }

    /**
     * @notice 设置保险状态
     * @param status 保险状态
     */
    function setInsuranceStatus(bool status) external onlyOwner {
        _insuranceStatus = status;
        emit InsuranceUpdated(status);
    }

    /**
     * @notice 设置流动性状态
     * @param status 流动性状态
     */
    function setLiquidityStatus(bool status) external onlyOwner {
        _liquidityStatus = status;
        emit LiquidityUpdated(status);
    }

    /**
     * @notice 设置风险等级
     * @param level 风险等级 (1-5)
     */
    function setRiskLevel(uint8 level) external onlyOwner {
        require(level >= 1 && level <= 5, "Invalid risk level");
        _riskLevel = level;
        emit RiskLevelUpdated(level);
    }

    /**
     * @notice 设置收益率
     * @param rate 年化收益率 (基点)
     */
    function setYieldRate(uint256 rate) external onlyOwner {
        _yieldRate = rate;
        emit YieldRateUpdated(rate);
    }

    /**
     * @notice 设置到期时间
     * @param maturityDate 到期时间戳
     */
    function setMaturityDate(uint256 maturityDate) external onlyOwner {
        _maturityDate = maturityDate;
        emit MaturityDateUpdated(maturityDate);
    }

    /**
     * @notice 设置估值方法
     * @param method 估值方法
     */
    function setValuationMethod(string memory method) external onlyOwner {
        _valuationMethod = method;
        emit ValuationUpdated(method);
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
        override 
        whenNotPaused 
        onlyKYCVerified 
        returns (bool) 
    {
        return super.transfer(to, amount);
    }

    // 重写ERC20的transferFrom函数，添加KYC检查
    function transferFrom(address from, address to, uint256 amount) 
        public 
        override 
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

    function getComplianceStatus() external view override returns (bool) {
        return _complianceStatus;
    }

    function getKYCRequired() external view override returns (bool) {
        return _kycRequired;
    }

    function isKYCVerified(address account) external view override returns (bool) {
        return _kycVerified[account];
    }

    function isRedeemable() external view override returns (bool) {
        return _isRedeemable;
    }

    function getMinimumTradeUnit() external view override returns (uint256) {
        return _minimumTradeUnit;
    }

    function getMaximumTradeUnit() external view override returns (uint256) {
        return _maximumTradeUnit;
    }

    function getIssuer() external view override returns (address) {
        return _issuer;
    }

    function getRegulatoryInfo() external view override returns (string memory) {
        return _regulatoryInfo;
    }

    function getValuationMethod() external view override returns (string memory) {
        return _valuationMethod;
    }

    function getAuditStatus() external view override returns (bool) {
        return _auditStatus;
    }

    function getInsuranceStatus() external view override returns (bool) {
        return _insuranceStatus;
    }

    function getCustodian() external view override returns (string memory) {
        return _custodian;
    }

    function getLiquidityStatus() external view override returns (bool) {
        return _liquidityStatus;
    }

    function getRiskLevel() external view override returns (uint8) {
        return _riskLevel;
    }

    function getYieldRate() external view override returns (uint256) {
        return _yieldRate;
    }

    function getMaturityDate() external view override returns (uint256) {
        return _maturityDate;
    }

    function getGeographicLocation() external view override returns (string memory) {
        return _geographicLocation;
    }

    function getIndustryCategory() external view override returns (string memory) {
        return _industryCategory;
    }

    function getDocumentHash() external view override returns (bytes32) {
        return _documentHash;
    }

    function getMetadataURI() external view override returns (string memory) {
        return _metadataURI;
    }

    /**
     * @notice 获取RWA Token的详细信息
     * @return 包含所有RWA Token信息的结构
     */
    function getRWAInfo() external view returns (
        string memory underlyingAssetType,
        string memory underlyingAssetId,
        bool complianceStatus,
        bool kycRequired,
        bool isRedeemable,
        uint256 minimumTradeUnit,
        uint256 maximumTradeUnit,
        address issuer,
        string memory regulatoryInfo,
        string memory valuationMethod,
        bool auditStatus,
        bool insuranceStatus,
        string memory custodian,
        bool liquidityStatus,
        uint8 riskLevel,
        uint256 yieldRate,
        uint256 maturityDate,
        string memory geographicLocation,
        string memory industryCategory,
        bytes32 documentHash,
        string memory metadataURI
    ) {
        return (
            _underlyingAssetType,
            _underlyingAssetId,
            _complianceStatus,
            _kycRequired,
            _isRedeemable,
            _minimumTradeUnit,
            _maximumTradeUnit,
            _issuer,
            _regulatoryInfo,
            _valuationMethod,
            _auditStatus,
            _insuranceStatus,
            _custodian,
            _liquidityStatus,
            _riskLevel,
            _yieldRate,
            _maturityDate,
            _geographicLocation,
            _industryCategory,
            _documentHash,
            _metadataURI
        );
    }
} 