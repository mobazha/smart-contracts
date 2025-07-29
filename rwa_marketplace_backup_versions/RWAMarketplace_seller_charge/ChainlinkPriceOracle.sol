// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title ChainlinkPriceOracle
 * @notice 基于Chainlink的价格预言机，支持Polygon链上的多种Token价格查询
 * @dev 支持USDT、USDC、DAI等主流稳定币的价格查询
 */
contract ChainlinkPriceOracle is Ownable {
    
    // 价格精度（Chainlink通常使用8位小数）
    uint8 public constant PRICE_DECIMALS = 8;
    // 输出精度（18位小数）
    uint8 public constant OUTPUT_DECIMALS = 18;
    
    // Token地址到Chainlink价格源的映射
    mapping(address => address) public priceFeeds;
    
    // 支持的Token列表
    address[] public supportedTokens;
    
    // 事件
    event PriceFeedUpdated(address indexed token, address indexed priceFeed);
    event PriceFeedRemoved(address indexed token);
    
    // 错误定义
    error TokenNotSupported();
    error InvalidPriceFeed();
    error StalePrice();
    error ZeroPrice();
    
    constructor() {
        // 初始化Polygon主网上的常用价格源
        _initializePolygonPriceFeeds();
    }
    
    /**
     * @notice 获取Token的USD价格
     * @param token Token地址
     * @return price 价格（18位小数）
     */
    function getPrice(address token) external view returns (uint256) {
        address priceFeed = priceFeeds[token];
        if (priceFeed == address(0)) {
            revert TokenNotSupported();
        }
        
        AggregatorV3Interface aggregator = AggregatorV3Interface(priceFeed);
        
        // 获取最新价格数据
        (
            /* uint80 roundId */,
            int256 price,
            /*uint startedAt*/,
            uint256 updatedAt,
            /*uint80 answeredInRound*/
        ) = aggregator.latestRoundData();
        
        // 检查价格有效性
        if (price <= 0) {
            revert ZeroPrice();
        }
        
        // 检查价格是否过期（5分钟）
        if (block.timestamp - updatedAt > 300) {
            revert StalePrice();
        }
        
        // 转换精度：从8位小数转换为18位小数
        return uint256(price) * 10**(OUTPUT_DECIMALS - PRICE_DECIMALS);
    }
    
    /**
     * @notice 获取两个Token之间的价格比率
     * @param baseToken 基础Token
     * @param quoteToken 报价Token
     * @return price 价格比率（18位小数）
     */
    function getPrice(address baseToken, address quoteToken) external view returns (uint256) {
        if (baseToken == quoteToken) {
            return 1e18; // 相同Token，比率为1
        }
        
        uint256 basePrice = this.getPrice(baseToken);
        uint256 quotePrice = this.getPrice(quoteToken);
        
        // 计算比率：basePrice / quotePrice
        return (basePrice * 1e18) / quotePrice;
    }
    
    /**
     * @notice 添加或更新价格源
     * @param token Token地址
     * @param priceFeed Chainlink价格源地址
     */
    function setPriceFeed(address token, address priceFeed) external onlyOwner {
        if (priceFeed == address(0)) {
            revert InvalidPriceFeed();
        }
        
        // 验证价格源合约
        AggregatorV3Interface aggregator = AggregatorV3Interface(priceFeed);
        try aggregator.latestRoundData() returns (
            uint80,
            int256,
            uint256,
            uint256,
            uint80
        ) {
            // 价格源有效
        } catch {
            revert InvalidPriceFeed();
        }
        
        // 如果是新Token，添加到支持列表
        if (priceFeeds[token] == address(0)) {
            supportedTokens.push(token);
        }
        
        priceFeeds[token] = priceFeed;
        emit PriceFeedUpdated(token, priceFeed);
    }
    
    /**
     * @notice 移除价格源
     * @param token Token地址
     */
    function removePriceFeed(address token) external onlyOwner {
        if (priceFeeds[token] == address(0)) {
            revert TokenNotSupported();
        }
        
        // 从支持列表中移除
        for (uint i = 0; i < supportedTokens.length; i++) {
            if (supportedTokens[i] == token) {
                supportedTokens[i] = supportedTokens[supportedTokens.length - 1];
                supportedTokens.pop();
                break;
            }
        }
        
        delete priceFeeds[token];
        emit PriceFeedRemoved(token);
    }
    
    /**
     * @notice 获取所有支持的Token
     * @return tokens 支持的Token地址数组
     */
    function getSupportedTokens() external view returns (address[] memory) {
        return supportedTokens;
    }
    
    /**
     * @notice 检查Token是否支持
     * @param token Token地址
     * @return supported 是否支持
     */
    function isTokenSupported(address token) external view returns (bool) {
        return priceFeeds[token] != address(0);
    }
    
    /**
     * @notice 获取价格源信息
     * @param token Token地址
     * @return priceFeed 价格源地址
     * @return decimals 价格源精度
     */
    function getPriceFeedInfo(address token) external view returns (address priceFeed, uint8 decimals) {
        priceFeed = priceFeeds[token];
        if (priceFeed == address(0)) {
            revert TokenNotSupported();
        }
        
        AggregatorV3Interface aggregator = AggregatorV3Interface(priceFeed);
        decimals = aggregator.decimals();
    }
    
    /**
     * @notice 初始化Polygon主网的价格源
     * @dev 这些是Polygon主网上的Chainlink价格源地址
     */
    function _initializePolygonPriceFeeds() internal {
        // Polygon主网价格源地址
        // 注意：这些地址需要根据实际部署情况调整
        
        // USDT/USD
        _setPriceFeedInternal(
            0xc2132D05D31c914a87C6611C10748AEb04B58e8F, // USDT on Polygon
            0x0A6513e40db6EB1b165753AD52E80663aeA50545  // USDT/USD price feed
        );
        
        // USDC/USD
        _setPriceFeedInternal(
            0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174, // USDC on Polygon
            0xfE4A8cc5b5B2366C1B58Bea3858e81843581b2F7  // USDC/USD price feed
        );
        
        // DAI/USD
        _setPriceFeedInternal(
            0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063, // DAI on Polygon
            0x4746DeC9e833A82EC7C2C1356372CcF2cfcD2F3D  // DAI/USD price feed
        );
        
        // WETH/USD
        _setPriceFeedInternal(
            0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619, // WETH on Polygon
            0xF9680D99D6C9589e2a93a78A04A279e509205945  // WETH/USD price feed
        );
        
        // WMATIC/USD
        _setPriceFeedInternal(
            0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270, // WMATIC on Polygon
            0xAB594600376Ec9fD91F8e885dADF0CE036862dE0  // WMATIC/USD price feed
        );
    }
    
    /**
     * @notice 内部设置价格源（不触发事件）
     * @param token Token地址
     * @param priceFeed 价格源地址
     */
    function _setPriceFeedInternal(address token, address priceFeed) internal {
        if (priceFeed != address(0)) {
            priceFeeds[token] = priceFeed;
            supportedTokens.push(token);
        }
    }
} 