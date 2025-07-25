// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title RwaMarketplace
 * @notice 支持多卖家多Token多币种的RWA分销平台，平台管理权限建议由多签钱包（如Gnosis Safe）持有，所有管理操作需多签共同签名。
 * @dev KYC校验可动态开关，平台方可根据合规需求决定是否强制校验KYC。
 */
interface IPriceOracle {
    function getPrice(address rwaToken, address payToken) external view returns (uint256);
}

contract RwaMarketplace {
    struct RwaSale {
        uint256 available;
        mapping(address => bool) allowedPayTokens;
    }

    mapping(address => mapping(address => RwaSale)) public sales;
    mapping(address => mapping(address => uint256)) public sellerPayTokenBalances;
    IPriceOracle public priceOracle;

    // 平台相关
    address public platform;
    uint256 public feeBps = 30; // 0.3%
    bool public paused;
    bool public kycRequired; // KYC校验开关

    // 白名单
    mapping(address => bool) public allowedSellers;
    mapping(address => bool) public allowedRwaTokens;
    mapping(address => bool) public allowedPayTokens;
    // KYC
    mapping(address => bool) public kycPassed;

    // 事件
    event Listed(address indexed seller, address indexed rwaToken, address[] payTokens);
    event Deposit(address indexed seller, address indexed rwaToken, uint256 amount);
    event Purchase(address indexed buyer, address indexed seller, address indexed rwaToken, address payToken, uint256 rwaAmount, uint256 payAmount, uint256 fee);
    event WithdrawPayToken(address indexed seller, address indexed payToken, uint256 amount);
    event WithdrawUnsoldRwaToken(address indexed seller, address indexed rwaToken, uint256 amount);
    event FeeChanged(uint256 newFeeBps);
    event PlatformChanged(address newPlatform);
    event SellerWhitelisted(address seller, bool allowed);
    event RwaTokenWhitelisted(address rwaToken, bool allowed);
    event PayTokenWhitelisted(address payToken, bool allowed);
    event KycStatusChanged(address user, bool passed);
    event KycRequiredChanged(bool required);
    event PriceOracleChanged(address newOracle);
    event Paused(bool status);

    modifier onlyPlatform() {
        require(msg.sender == platform, "Not platform");
        _;
    }
    modifier notPaused() {
        require(!paused, "Paused");
        _;
    }

    constructor(address _priceOracle, address _platform) {
        priceOracle = IPriceOracle(_priceOracle);
        platform = _platform; // 建议为多签钱包地址
        kycRequired = true; // 默认开启KYC校验
    }

    // 平台管理
    function setFeeBps(uint256 newFeeBps) external onlyPlatform {
        require(newFeeBps <= 1000, "Fee too high");
        feeBps = newFeeBps;
        emit FeeChanged(newFeeBps);
    }
    function setPlatform(address newPlatform) external onlyPlatform {
        require(newPlatform != address(0), "Zero address");
        platform = newPlatform;
        emit PlatformChanged(newPlatform);
    }
    function setPriceOracle(address newOracle) external onlyPlatform {
        require(newOracle != address(0), "Zero address");
        priceOracle = IPriceOracle(newOracle);
        emit PriceOracleChanged(newOracle);
    }
    function setPaused(bool _paused) external onlyPlatform {
        paused = _paused;
        emit Paused(_paused);
    }
    // KYC校验开关
    function setKycRequired(bool required) external onlyPlatform {
        kycRequired = required;
        emit KycRequiredChanged(required);
    }
    // 白名单管理
    function setSellerWhitelist(address seller, bool allowed) external onlyPlatform {
        allowedSellers[seller] = allowed;
        emit SellerWhitelisted(seller, allowed);
    }
    function setRwaTokenWhitelist(address rwaToken, bool allowed) external onlyPlatform {
        allowedRwaTokens[rwaToken] = allowed;
        emit RwaTokenWhitelisted(rwaToken, allowed);
    }
    function setPayTokenWhitelist(address payToken, bool allowed) external onlyPlatform {
        allowedPayTokens[payToken] = allowed;
        emit PayTokenWhitelisted(payToken, allowed);
    }
    // KYC管理
    function setKycStatus(address user, bool passed) external onlyPlatform {
        kycPassed[user] = passed;
        emit KycStatusChanged(user, passed);
    }

    // 卖家上架RWA Token，设置支持的支付币种
    function listRwaToken(address rwaToken, address[] calldata payTokens) external notPaused {
        require(allowedSellers[msg.sender], "Seller not whitelisted");
        require(allowedRwaTokens[rwaToken], "RWA Token not whitelisted");
        RwaSale storage sale = sales[msg.sender][rwaToken];
        for (uint i = 0; i < payTokens.length; i++) {
            require(allowedPayTokens[payTokens[i]], "Pay token not whitelisted");
            sale.allowedPayTokens[payTokens[i]] = true;
        }
        emit Listed(msg.sender, rwaToken, payTokens);
    }

    // 卖家充值RWA Token
    function depositRwaToken(address rwaToken, uint256 amount) external notPaused {
        require(allowedSellers[msg.sender], "Seller not whitelisted");
        require(allowedRwaTokens[rwaToken], "RWA Token not whitelisted");
        require(amount > 0, "Amount must be > 0");
        IERC20(rwaToken).transferFrom(msg.sender, address(this), amount);
        sales[msg.sender][rwaToken].available += amount;
        emit Deposit(msg.sender, rwaToken, amount);
    }

    // 买家购买
    function buyRwaToken(address seller, address rwaToken, uint256 rwaAmount, address payToken) external notPaused {
        require(allowedSellers[seller], "Seller not whitelisted");
        require(allowedRwaTokens[rwaToken], "RWA Token not whitelisted");
        require(allowedPayTokens[payToken], "Pay token not whitelisted");
        if (kycRequired) {
            require(kycPassed[msg.sender], "Buyer not KYC passed");
        }
        require(rwaAmount > 0, "Amount must be > 0");
        require(sales[seller][rwaToken].allowedPayTokens[payToken], "Pay token not allowed by seller");
        require(sales[seller][rwaToken].available >= rwaAmount, "Insufficient RWA token");
        uint256 price = priceOracle.getPrice(rwaToken, payToken);
        require(price > 0, "Price not set");
        uint256 payAmount = rwaAmount * price / 1e18;
        uint256 fee = payAmount * feeBps / 10000;
        uint256 sellerAmount = payAmount - fee;
        IERC20(payToken).transferFrom(msg.sender, address(this), payAmount);
        IERC20(rwaToken).transfer(msg.sender, rwaAmount);
        sales[seller][rwaToken].available -= rwaAmount;
        sellerPayTokenBalances[seller][payToken] += sellerAmount;
        sellerPayTokenBalances[platform][payToken] += fee;
        emit Purchase(msg.sender, seller, rwaToken, payToken, rwaAmount, payAmount, fee);
    }

    // 卖家提现支付币
    function withdrawPayToken(address payToken, uint256 amount) external notPaused {
        require(amount > 0, "Amount must be > 0");
        require(sellerPayTokenBalances[msg.sender][payToken] >= amount, "Insufficient balance");
        sellerPayTokenBalances[msg.sender][payToken] -= amount;
        IERC20(payToken).transfer(msg.sender, amount);
        emit WithdrawPayToken(msg.sender, payToken, amount);
    }

    // 卖家撤回未售RWA Token
    function withdrawUnsoldRwaToken(address rwaToken, uint256 amount) external notPaused {
        require(amount > 0, "Amount must be > 0");
        require(sales[msg.sender][rwaToken].available >= amount, "Insufficient RWA token");
        sales[msg.sender][rwaToken].available -= amount;
        IERC20(rwaToken).transfer(msg.sender, amount);
        emit WithdrawUnsoldRwaToken(msg.sender, rwaToken, amount);
    }
} 