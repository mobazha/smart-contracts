// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.22;

/**
 * @title IRWAToken
 * @notice RWA Token合约的标准接口
 * @dev 所有RWA Token合约都应该实现这个接口
 */
interface IRWAToken {
    /**
     * @notice 获取代币名称
     * @return 代币名称
     */
    function name() external view returns (string memory);

    /**
     * @notice 获取代币符号
     * @return 代币符号
     */
    function symbol() external view returns (string memory);

    /**
     * @notice 获取代币精度
     * @return 代币精度
     */
    function decimals() external view returns (uint8);

    /**
     * @notice 获取代币总供应量
     * @return 总供应量
     */
    function totalSupply() external view returns (uint256);

    /**
     * @notice 获取指定地址的代币余额
     * @param account 账户地址
     * @return 代币余额
     */
    function balanceOf(address account) external view returns (uint256);

    /**
     * @notice 转移代币
     * @param to 接收地址
     * @param amount 转移数量
     * @return 是否成功
     */
    function transfer(address to, uint256 amount) external returns (bool);

    /**
     * @notice 从指定地址转移代币
     * @param from 发送地址
     * @param to 接收地址
     * @param amount 转移数量
     * @return 是否成功
     */
    function transferFrom(address from, address to, uint256 amount) external returns (bool);

    /**
     * @notice 授权指定地址使用代币
     * @param spender 被授权地址
     * @param amount 授权数量
     * @return 是否成功
     */
    function approve(address spender, uint256 amount) external returns (bool);

    /**
     * @notice 获取授权额度
     * @param owner 所有者地址
     * @param spender 被授权地址
     * @return 授权额度
     */
    function allowance(address owner, address spender) external view returns (uint256);

    /**
     * @notice 获取RWA Token的底层资产信息
     * @return 底层资产类型
     */
    function getUnderlyingAssetType() external view returns (string memory);

    /**
     * @notice 获取RWA Token的底层资产标识符
     * @return 底层资产标识符
     */
    function getUnderlyingAssetId() external view returns (string memory);

    /**
     * @notice 获取RWA Token的合规信息
     * @return 合规状态
     */
    function getComplianceStatus() external view returns (bool);

    /**
     * @notice 获取RWA Token的KYC要求
     * @return KYC要求状态
     */
    function getKYCRequired() external view returns (bool);

    /**
     * @notice 检查地址是否通过KYC验证
     * @param account 账户地址
     * @return 是否通过KYC
     */
    function isKYCVerified(address account) external view returns (bool);

    /**
     * @notice 获取RWA Token的赎回信息
     * @return 是否可赎回
     */
    function isRedeemable() external view returns (bool);

    /**
     * @notice 获取RWA Token的最小交易单位
     * @return 最小交易单位
     */
    function getMinimumTradeUnit() external view returns (uint256);

    /**
     * @notice 获取RWA Token的最大交易单位
     * @return 最大交易单位
     */
    function getMaximumTradeUnit() external view returns (uint256);

    /**
     * @notice 获取RWA Token的发行人地址
     * @return 发行人地址
     */
    function getIssuer() external view returns (address);

    /**
     * @notice 获取RWA Token的监管机构信息
     * @return 监管机构信息
     */
    function getRegulatoryInfo() external view returns (string memory);

    /**
     * @notice 获取RWA Token的估值信息
     * @return 估值方法
     */
    function getValuationMethod() external view returns (string memory);

    /**
     * @notice 获取RWA Token的审计信息
     * @return 审计状态
     */
    function getAuditStatus() external view returns (bool);

    /**
     * @notice 获取RWA Token的保险信息
     * @return 保险状态
     */
    function getInsuranceStatus() external view returns (bool);

    /**
     * @notice 获取RWA Token的托管信息
     * @return 托管机构
     */
    function getCustodian() external view returns (string memory);

    /**
     * @notice 获取RWA Token的流动性信息
     * @return 流动性状态
     */
    function getLiquidityStatus() external view returns (bool);

    /**
     * @notice 获取RWA Token的风险等级
     * @return 风险等级
     */
    function getRiskLevel() external view returns (uint8);

    /**
     * @notice 获取RWA Token的收益率信息
     * @return 年化收益率
     */
    function getYieldRate() external view returns (uint256);

    /**
     * @notice 获取RWA Token的到期时间
     * @return 到期时间戳
     */
    function getMaturityDate() external view returns (uint256);

    /**
     * @notice 获取RWA Token的地理位置信息
     * @return 地理位置
     */
    function getGeographicLocation() external view returns (string memory);

    /**
     * @notice 获取RWA Token的行业分类
     * @return 行业分类
     */
    function getIndustryCategory() external view returns (string memory);

    /**
     * @notice 获取RWA Token的文档哈希
     * @return 文档哈希
     */
    function getDocumentHash() external view returns (bytes32);

    /**
     * @notice 获取RWA Token的元数据URI
     * @return 元数据URI
     */
    function getMetadataURI() external view returns (string memory);

    // 事件定义
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event RWAComplianceUpdated(bool complianceStatus);
    event KYCStatusUpdated(address indexed account, bool kycStatus);
    event ValuationUpdated(string valuationMethod);
    event AuditCompleted(bool auditStatus);
    event InsuranceUpdated(bool insuranceStatus);
    event LiquidityUpdated(bool liquidityStatus);
    event RiskLevelUpdated(uint8 riskLevel);
    event YieldRateUpdated(uint256 yieldRate);
    event MaturityDateUpdated(uint256 maturityDate);
} 