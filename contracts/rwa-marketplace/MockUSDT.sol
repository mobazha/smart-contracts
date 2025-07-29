// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title MockUSDT
 * @notice 模拟USDT稳定币合约，用于测试
 * @dev 使用6位小数，符合USDT标准
 */
contract MockUSDT is ERC20, Ownable {
    uint8 private _decimals;

    constructor() ERC20("Tether USD", "USDT") Ownable() {
        _decimals = 6; // USDT使用6位小数
    }

    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }

    /**
     * @notice 铸造代币
     * @param to 接收地址
     * @param amount 数量
     */
    function mint(address to, uint256 amount) external onlyOwner {
        _mint(to, amount);
    }

    /**
     * @notice 销毁代币
     * @param from 发送地址
     * @param amount 数量
     */
    function burn(address from, uint256 amount) external onlyOwner {
        _burn(from, amount);
    }
} 