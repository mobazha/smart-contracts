// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.22;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/utils/Address.sol";
import "../token/ITokenContract.sol";

/**
 * @title RWA Token Marketplace
 * @notice 专门用于RWA Token交易的智能合约
 * @dev 支持买家下单、卖家发货、RWA Token转移等功能
 */
contract RWAMarketplace is Ownable, ReentrancyGuard, Pausable {
    using Address for address payable;

    // 订单状态枚举
    enum OrderStatus {
        CREATED,        // 订单已创建
        PAID,           // 买家已付款
        SHIPPED,        // 卖家已发货
        COMPLETED,      // 交易完成
        CANCELLED,      // 订单取消
        DISPUTED        // 争议中
    }

    // 订单结构
    struct Order {
        address buyer;                    // 买家地址
        address seller;                   // 卖家地址
        address rwaTokenAddress;          // RWA Token合约地址
        address paymentTokenAddress;      // 支付代币地址（0表示ETH）
        address buyerReceiveAddress;      // 买家接收RWA Token的地址
        uint256 rwaTokenAmount;           // RWA Token数量
        uint256 paymentAmount;            // 支付金额
        uint256 orderId;                  // 订单ID
        uint256 createdAt;                // 创建时间
        uint256 paidAt;                   // 付款时间
        uint256 shippedAt;                // 发货时间
        uint256 completedAt;              // 完成时间
        OrderStatus status;               // 订单状态
        bool isActive;                    // 订单是否激活
    }

    // 事件定义
    event OrderCreated(
        uint256 indexed orderId,
        address indexed buyer,
        address indexed seller,
        address rwaTokenAddress,
        address paymentTokenAddress,
        uint256 rwaTokenAmount,
        uint256 paymentAmount
    );

    event OrderPaid(
        uint256 indexed orderId,
        address indexed buyer,
        uint256 paymentAmount,
        uint256 paidAt
    );

    event OrderShipped(
        uint256 indexed orderId,
        address indexed seller,
        address rwaTokenAddress,
        uint256 rwaTokenAmount,
        uint256 shippedAt
    );

    event OrderCompleted(
        uint256 indexed orderId,
        address indexed buyer,
        address indexed seller,
        uint256 completedAt
    );

    event OrderCancelled(
        uint256 indexed orderId,
        address indexed cancelledBy,
        uint256 cancelledAt
    );

    event OrderDisputed(
        uint256 indexed orderId,
        address indexed disputedBy,
        uint256 disputedAt
    );

    event FundsWithdrawn(
        address indexed recipient,
        uint256 amount,
        address tokenAddress
    );

    // 状态变量
    uint256 public orderCounter = 0;
    uint256 public platformFee = 25; // 0.25% 平台费用 (25 = 0.25%)
    uint256 public constant FEE_DENOMINATOR = 10000;
    
    // 映射
    mapping(uint256 => Order) public orders;
    mapping(address => uint256[]) public buyerOrders;
    mapping(address => uint256[]) public sellerOrders;
    mapping(address => bool) public authorizedOperators;

    // 修饰符
    modifier onlyAuthorizedOperator() {
        require(
            authorizedOperators[msg.sender] || msg.sender == owner(),
            "Not authorized operator"
        );
        _;
    }

    modifier orderExists(uint256 orderId) {
        require(orders[orderId].isActive, "Order does not exist");
        _;
    }

    modifier onlyBuyer(uint256 orderId) {
        require(orders[orderId].buyer == msg.sender, "Not the buyer");
        _;
    }

    modifier onlySeller(uint256 orderId) {
        require(orders[orderId].seller == msg.sender, "Not the seller");
        _;
    }

    modifier validOrderStatus(uint256 orderId, OrderStatus expectedStatus) {
        require(orders[orderId].status == expectedStatus, "Invalid order status");
        _;
    }

    modifier nonZeroAddress(address _address) {
        require(_address != address(0), "Zero address not allowed");
        _;
    }

    modifier nonZeroAmount(uint256 amount) {
        require(amount > 0, "Amount must be greater than 0");
        _;
    }

    /**
     * @notice 构造函数
     */
    constructor() {
        authorizedOperators[msg.sender] = true;
    }

    /**
     * @notice 创建RWA Token订单
     * @param seller 卖家地址
     * @param rwaTokenAddress RWA Token合约地址
     * @param paymentTokenAddress 支付代币地址（0表示ETH）
     * @param buyerReceiveAddress 买家接收RWA Token的地址
     * @param rwaTokenAmount RWA Token数量
     * @param paymentAmount 支付金额
     */
    function createOrder(
        address seller,
        address rwaTokenAddress,
        address paymentTokenAddress,
        address buyerReceiveAddress,
        uint256 rwaTokenAmount,
        uint256 paymentAmount
    )
        external
        nonReentrant
        whenNotPaused
        nonZeroAddress(seller)
        nonZeroAddress(rwaTokenAddress)
        nonZeroAddress(buyerReceiveAddress)
        nonZeroAmount(rwaTokenAmount)
        nonZeroAmount(paymentAmount)
        returns (uint256 orderId)
    {
        orderId = ++orderCounter;

        Order storage order = orders[orderId];
        order.buyer = msg.sender;
        order.seller = seller;
        order.rwaTokenAddress = rwaTokenAddress;
        order.paymentTokenAddress = paymentTokenAddress;
        order.buyerReceiveAddress = buyerReceiveAddress;
        order.rwaTokenAmount = rwaTokenAmount;
        order.paymentAmount = paymentAmount;
        order.orderId = orderId;
        order.createdAt = block.timestamp;
        order.status = OrderStatus.CREATED;
        order.isActive = true;

        buyerOrders[msg.sender].push(orderId);
        sellerOrders[seller].push(orderId);

        emit OrderCreated(
            orderId,
            msg.sender,
            seller,
            rwaTokenAddress,
            paymentTokenAddress,
            rwaTokenAmount,
            paymentAmount
        );

        return orderId;
    }

    /**
     * @notice 买家付款
     * @param orderId 订单ID
     */
    function payOrder(uint256 orderId)
        external
        payable
        nonReentrant
        whenNotPaused
        orderExists(orderId)
        onlyBuyer(orderId)
        validOrderStatus(orderId, OrderStatus.CREATED)
    {
        Order storage order = orders[orderId];
        
        if (order.paymentTokenAddress == address(0)) {
            // ETH支付
            require(msg.value == order.paymentAmount, "Incorrect payment amount");
        } else {
            // ERC20代币支付
            require(msg.value == 0, "ETH not accepted for token payment");
            ITokenContract paymentToken = ITokenContract(order.paymentTokenAddress);
            require(
                paymentToken.transferFrom(msg.sender, address(this), order.paymentAmount),
                "Token transfer failed"
            );
        }

        order.status = OrderStatus.PAID;
        order.paidAt = block.timestamp;

        emit OrderPaid(orderId, msg.sender, order.paymentAmount, block.timestamp);
    }

    /**
     * @notice 卖家发货（转移RWA Token给买家）
     * @param orderId 订单ID
     */
    function shipOrder(uint256 orderId)
        external
        nonReentrant
        whenNotPaused
        orderExists(orderId)
        onlySeller(orderId)
        validOrderStatus(orderId, OrderStatus.PAID)
    {
        Order storage order = orders[orderId];
        
        // 转移RWA Token给买家
        ITokenContract rwaToken = ITokenContract(order.rwaTokenAddress);
        require(
            rwaToken.transferFrom(msg.sender, order.buyerReceiveAddress, order.rwaTokenAmount),
            "RWA Token transfer failed"
        );

        order.status = OrderStatus.SHIPPED;
        order.shippedAt = block.timestamp;

        emit OrderShipped(
            orderId,
            msg.sender,
            order.rwaTokenAddress,
            order.rwaTokenAmount,
            block.timestamp
        );
    }

    /**
     * @notice 买家确认收货，完成交易
     * @param orderId 订单ID
     */
    function completeOrder(uint256 orderId)
        external
        nonReentrant
        whenNotPaused
        orderExists(orderId)
        onlyBuyer(orderId)
        validOrderStatus(orderId, OrderStatus.SHIPPED)
    {
        Order storage order = orders[orderId];
        
        // 计算平台费用
        uint256 platformFeeAmount = (order.paymentAmount * platformFee) / FEE_DENOMINATOR;
        uint256 sellerAmount = order.paymentAmount - platformFeeAmount;

        // 转移付款给卖家
        if (order.paymentTokenAddress == address(0)) {
            // ETH支付
            payable(order.seller).transfer(sellerAmount);
        } else {
            // ERC20代币支付
            ITokenContract paymentToken = ITokenContract(order.paymentTokenAddress);
            require(
                paymentToken.transfer(order.seller, sellerAmount),
                "Seller payment failed"
            );
        }

        order.status = OrderStatus.COMPLETED;
        order.completedAt = block.timestamp;

        emit OrderCompleted(orderId, msg.sender, order.seller, block.timestamp);
    }

    /**
     * @notice 取消订单
     * @param orderId 订单ID
     */
    function cancelOrder(uint256 orderId)
        external
        nonReentrant
        whenNotPaused
        orderExists(orderId)
    {
        Order storage order = orders[orderId];
        
        require(
            msg.sender == order.buyer || msg.sender == order.seller,
            "Not authorized to cancel"
        );
        
        require(
            order.status == OrderStatus.CREATED || order.status == OrderStatus.PAID,
            "Cannot cancel order in current status"
        );

        // 如果已付款，退还给买家
        if (order.status == OrderStatus.PAID) {
            if (order.paymentTokenAddress == address(0)) {
                payable(order.buyer).transfer(order.paymentAmount);
            } else {
                ITokenContract paymentToken = ITokenContract(order.paymentTokenAddress);
                require(
                    paymentToken.transfer(order.buyer, order.paymentAmount),
                    "Refund failed"
                );
            }
        }

        order.status = OrderStatus.CANCELLED;
        order.isActive = false;

        emit OrderCancelled(orderId, msg.sender, block.timestamp);
    }

    /**
     * @notice 争议订单
     * @param orderId 订单ID
     */
    function disputeOrder(uint256 orderId)
        external
        nonReentrant
        whenNotPaused
        orderExists(orderId)
    {
        Order storage order = orders[orderId];
        
        require(
            msg.sender == order.buyer || msg.sender == order.seller,
            "Not authorized to dispute"
        );
        
        require(
            order.status == OrderStatus.PAID || order.status == OrderStatus.SHIPPED,
            "Cannot dispute order in current status"
        );

        order.status = OrderStatus.DISPUTED;

        emit OrderDisputed(orderId, msg.sender, block.timestamp);
    }

    /**
     * @notice 管理员处理争议订单
     * @param orderId 订单ID
     * @param refundBuyer 是否退款给买家
     */
    function resolveDispute(uint256 orderId, bool refundBuyer)
        external
        onlyAuthorizedOperator
        orderExists(orderId)
        validOrderStatus(orderId, OrderStatus.DISPUTED)
    {
        Order storage order = orders[orderId];
        
        if (refundBuyer) {
            // 退款给买家
            if (order.paymentTokenAddress == address(0)) {
                payable(order.buyer).transfer(order.paymentAmount);
            } else {
                ITokenContract paymentToken = ITokenContract(order.paymentTokenAddress);
                require(
                    paymentToken.transfer(order.buyer, order.paymentAmount),
                    "Refund failed"
                );
            }
        } else {
            // 付款给卖家
            uint256 platformFeeAmount = (order.paymentAmount * platformFee) / FEE_DENOMINATOR;
            uint256 sellerAmount = order.paymentAmount - platformFeeAmount;
            
            if (order.paymentTokenAddress == address(0)) {
                payable(order.seller).transfer(sellerAmount);
            } else {
                ITokenContract paymentToken = ITokenContract(order.paymentTokenAddress);
                require(
                    paymentToken.transfer(order.seller, sellerAmount),
                    "Seller payment failed"
                );
            }
        }

        order.status = OrderStatus.CANCELLED;
        order.isActive = false;
    }

    /**
     * @notice 提取合约中的资金
     * @param recipient 接收地址
     * @param amount 金额
     * @param tokenAddress 代币地址（0表示ETH）
     */
    function withdrawFunds(
        address recipient,
        uint256 amount,
        address tokenAddress
    )
        external
        onlyOwner
        nonZeroAddress(recipient)
        nonZeroAmount(amount)
    {
        if (tokenAddress == address(0)) {
            require(address(this).balance >= amount, "Insufficient ETH balance");
            payable(recipient).transfer(amount);
        } else {
            ITokenContract token = ITokenContract(tokenAddress);
            require(
                token.transfer(recipient, amount),
                "Token transfer failed"
            );
        }

        emit FundsWithdrawn(recipient, amount, tokenAddress);
    }

    /**
     * @notice 设置平台费用
     * @param newFee 新费用（基点，如25表示0.25%）
     */
    function setPlatformFee(uint256 newFee) external onlyOwner {
        require(newFee <= 1000, "Fee too high"); // 最大10%
        platformFee = newFee;
    }

    /**
     * @notice 设置授权操作员
     * @param operator 操作员地址
     * @param authorized 是否授权
     */
    function setAuthorizedOperator(address operator, bool authorized)
        external
        onlyOwner
        nonZeroAddress(operator)
    {
        authorizedOperators[operator] = authorized;
    }

    /**
     * @notice 暂停合约
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @notice 恢复合约
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    // 查询函数

    /**
     * @notice 获取订单信息
     * @param orderId 订单ID
     * @return 订单结构
     */
    function getOrder(uint256 orderId) external view returns (Order memory) {
        return orders[orderId];
    }

    /**
     * @notice 获取买家的订单列表
     * @param buyer 买家地址
     * @return 订单ID数组
     */
    function getBuyerOrders(address buyer) external view returns (uint256[] memory) {
        return buyerOrders[buyer];
    }

    /**
     * @notice 获取卖家的订单列表
     * @param seller 卖家地址
     * @return 订单ID数组
     */
    function getSellerOrders(address seller) external view returns (uint256[] memory) {
        return sellerOrders[seller];
    }

    /**
     * @notice 获取合约ETH余额
     */
    function getETHBalance() external view returns (uint256) {
        return address(this).balance;
    }

    /**
     * @notice 获取合约代币余额
     * @param tokenAddress 代币地址
     */
    function getTokenBalance(address tokenAddress) external view returns (uint256) {
        ITokenContract token = ITokenContract(tokenAddress);
        return token.balanceOf(address(this));
    }

    // 接收ETH
    receive() external payable {
        // 允许接收ETH
    }
} 