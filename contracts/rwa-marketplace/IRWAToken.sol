// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.22;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title IRWAToken
 * @notice RWA Token合约的简化接口
 * @dev 用于demo目的的简化RWA Token接口
 */
interface IRWAToken is IERC20 {

    /**
     * @notice 获取RWA Token的底层资产类型
     * @return 底层资产类型
     */
    function getUnderlyingAssetType() external view returns (string memory);

    /**
     * @notice 获取RWA Token的底层资产标识符
     * @return 底层资产标识符
     */
    function getUnderlyingAssetId() external view returns (string memory);

    /**
     * @notice 获取RWA Token的发行人地址
     * @return 发行人地址
     */
    function getIssuer() external view returns (address);

    /**
     * @notice 获取RWA Token的合规状态
     * @return 合规状态
     */
    function getComplianceStatus() external view returns (bool);

    /**
     * @notice 检查地址是否通过KYC验证
     * @param account 账户地址
     * @return 是否通过KYC
     */
    function isKYCVerified(address account) external view returns (bool);

    // RWA特定事件定义
    event RWAComplianceUpdated(bool complianceStatus);
    event KYCStatusUpdated(address indexed account, bool kycStatus);
} 