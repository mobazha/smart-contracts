// SPDX-License-Identifier: MBZ
pragma solidity ^0.8.22;

import "./Common.sol";
import "../token/ITokenContract.sol";


/**
* @title Mobazha Escrow
* @notice Holds ETH and ERC20 tokens for moderated trades on the Mobazha
* platform. See the specification here:
* https://github.com/mobazha/bsc-smart-contracts/blob/master/contracts/escrow/EscrowSpec.md
* @dev Do not use this contract with tokens that do not strictly adhere to the
* ERC20 token standard. In particular, all successful calls to `transfer` and
* `transferFrom` on the token contract MUST return true. Non-compliant tokens
* may get trapped in this contract forever. See the specification for more
* details.
*/
contract Escrow {

    enum Status {FUNDED, RELEASED}

    enum TransactionType {ETH, TOKEN}

    event Executed(
        bytes32 indexed scriptHash,
        address payable[] destinations,
        uint256[] amounts
    );

    event Funded(
        bytes32 indexed scriptHash,
        address indexed from,
        uint256 value
    );

    struct Transaction {
        uint256 value;
        uint256 lastModified;
        Status status;
        TransactionType transactionType;
        uint8 threshold;
        uint32 timeoutHours;
        address buyer;
        address seller;
        address tokenAddress; //address of ERC20 token if applicable
        address moderator;
        uint256 released;
        uint256 noOfReleases; //number of times funds have been released
        mapping(address => bool) isOwner;
        //tracks who has authorized release of funds from escrow
        mapping(bytes32 => bool) voted;
        //tracks who has received funds released from escrow
        mapping(address => bool) beneficiaries;
    }

    mapping(bytes32 => Transaction) public transactions;

    uint256 public transactionCount = 0;

    //maps address to array of scriptHashes of all Mobazha transacations for
    //which they are either the buyer or the seller
    mapping(address => bytes32[]) private partyVsTransaction;

    address private _owner;

    modifier transactionExists(bytes32 scriptHash) {
        require(
            transactions[scriptHash].value != 0, "Transaction does not exist"
        );
        _;
    }

    modifier transactionDoesNotExist(bytes32 scriptHash) {
        require(transactions[scriptHash].value == 0, "Transaction exists");
        _;
    }

    modifier inFundedState(bytes32 scriptHash) {
        require(
            transactions[scriptHash].status == Status.FUNDED,
            "Not in FUNDED state"
        );
        _;
    }

    modifier fundsExist(bytes32 scriptHash) {
        require(
            transactions[scriptHash].value - transactions[scriptHash].released > 0,
            "All funds has been released"
        );
        _;
    }

    modifier nonZeroAddress(address addressToCheck) {
        require(addressToCheck != address(0), "Zero address passed");
        _;
    }

    modifier checkTransactionType(
        bytes32 scriptHash,
        TransactionType transactionType
    )
    {
        require(
            transactions[scriptHash].transactionType == transactionType,
            "Transaction type does not match"
        );
        _;
    }

    modifier onlyBuyer(bytes32 scriptHash) {
        require(
            msg.sender == transactions[scriptHash].buyer,
            "Not buyer"
        );
        _;
    }

    //Address of the reward Token to be distributed to the users
    ITokenContract public mbzToken;

    /**
    * @dev Add reward token contract address at the time of deployment
    * @param mbzTokenAddress Address of the reward token contract
    */
    constructor(
        address mbzTokenAddress
    )
        nonZeroAddress(mbzTokenAddress)
    {
        _owner = msg.sender;

        mbzToken = ITokenContract(mbzTokenAddress);
    }

    /**
    * @dev Allows the owner of the contract to transfer all remaining MBZ tokens to
    * an address of their choosing.
    * @param receiver The receiver's address
    */
    function transferRemainingMBZTokens(
        address receiver
    )
        external
        nonZeroAddress(receiver)
    {
        require(msg.sender == _owner, "Not the owner");

        mbzToken.transfer(receiver, mbzToken.balanceOf(address(this)));
    }

    /**
    * @dev Allows the owner of the contract to transfer funds to an address of their choosing.
    // If it is main net coin, tokenAddress can be 0
    * @param receiver The receiver's address
    */
    function transferLockedFunds(
        address receiver,
        uint256 value,
        TransactionType transactionType,
        address tokenAddress
    )
        external
        nonZeroAddress(receiver)
    {
        require(msg.sender == _owner, "Not the owner");

        require(value > 0, "Value 0");

        if (transactionType == TransactionType.ETH) {
            payable(receiver).transfer(value);
        } else {
            ITokenContract token = ITokenContract(tokenAddress);
            require(token.transfer(receiver, value), "Token transfer failed.");
        }
    }

    /**
    * @notice Registers a new Mobazha transaction to the contract
    * @dev To be used for moderated ETH transactions
    * @param buyer The buyer associated with the Mobazha transaction
    * @param seller The seller associated with the Mobazha transaction
    * @param moderator The moderator (if any) associated with the Mobazha
    * transaction
    * @param threshold The minimum number of signatures required to release
    * funds from escrow before the timeout.
    * @param timeoutHours The number hours after which the seller can
    * unilaterally release funds from escrow. When timeoutHours is set to 0
    * it means the seller can never unilaterally release funds from escrow
    * @param scriptHash The keccak256 hash of the redeem script. See
    * specification for more details
    * @param uniqueId A nonce chosen by the buyer
    * @dev This call is intended to be made by the buyer and should send the
    * amount of ETH to be put in escrow
    * @dev You MUST NOT pass a contract address for buyer, seller, or moderator
    * or else funds could be locked in this contract permanently. Releasing
    * funds from this contract require signatures that cannot be created by
    * contract addresses
    */
    function addTransaction(
        address buyer,
        address seller,
        address moderator,
        uint8 threshold,
        uint32 timeoutHours,
        bytes32 scriptHash,
        bytes20 uniqueId
    )
        external
        payable
        transactionDoesNotExist(scriptHash)
        nonZeroAddress(buyer)
        nonZeroAddress(seller)
    {
        _addTransaction(
            buyer,
            seller,
            moderator,
            threshold,
            timeoutHours,
            scriptHash,
            msg.value,
            uniqueId,
            TransactionType.ETH,
            address(0)
        );

        emit Funded(scriptHash, msg.sender, msg.value);
    }

    /**
    * @notice Registers a new Mobazha transaction to the contract
    * @dev To be used for moderated ERC20 transactions
    * @param buyer The buyer associated with the Mobazha transaction
    * @param seller The seller associated with the Mobazha transaction
    * @param moderator The moderator (if any) associated with the Mobazha
    * transaction
    * @param threshold The minimum number of signatures required to release
    * funds from escrow before the timeout.
    * @param timeoutHours The number hours after which the seller can
    * unilaterally release funds from escrow. When timeoutHours is set to 0
    * it means the seller can never unilaterally release funds from escrow
    * @param scriptHash The keccak256 hash of the redeem script. See
    * specification for more details
    * @param value The number of tokens to be held in escrow
    * @param uniqueId A nonce chosen by the buyer
    * @param tokenAddress The address of the ERC20 token contract
    * @dev Be sure the buyer approves this contract to spend at least `value`
    * on the buyer's behalf
    * @dev You MUST NOT pass a contract address for buyer, seller, or moderator
    * or else funds could be locked in this contract permanently. Releasing
    * funds from this contract require signatures that cannot be created by
    * contract addresses
    */
    function addTokenTransaction(
        address buyer,
        address seller,
        address moderator,
        uint8 threshold,
        uint32 timeoutHours,
        bytes32 scriptHash,
        uint256 value,
        bytes20 uniqueId,
        address tokenAddress
    )
        external
        transactionDoesNotExist(scriptHash)
        nonZeroAddress(buyer)
        nonZeroAddress(seller)
        nonZeroAddress(tokenAddress)
    {
        _addTransaction(
            buyer,
            seller,
            moderator,
            threshold,
            timeoutHours,
            scriptHash,
            value,
            uniqueId,
            TransactionType.TOKEN,
            tokenAddress
        );

        ITokenContract token = ITokenContract(tokenAddress);

        emit Funded(scriptHash, msg.sender, value);

        require(
            token.transferFrom(msg.sender, address(this), value),
            "Token transfer failed, maybe you did not approve escrow contract to spend on behalf of sender"
        );
    }

    /**
    * @notice Determines whether a given address was a beneficiary of any
    * payout from the escrow associated with an Mobazha transaction that is
    * associated with a given scriptHash
    * @param scriptHash scriptHash associated with the Mobazha transaction
    * of interest
    * @param beneficiary Address to be checked
    * @return true if and only if the passed address was a beneficiary of some
    * payout from the escrow associated with `scriptHash`
    */
    function checkBeneficiary(
        bytes32 scriptHash,
        address beneficiary
    )
        external
        view
        returns (bool)
    {
        return transactions[scriptHash].beneficiaries[beneficiary];
    }

    /**
    * @notice Check whether given party has signed for funds to be released
    * from the escrow associated with a scriptHash.
    * @param scriptHash Hash identifying the Mobazha transaction in question
    * @param party The address we are checking
    * @return true if and only if `party` received any funds from the escrow
    * associated with `scripHash`
    */
    function checkVote(
        bytes32 scriptHash,
        address party
    )
        external
        view
        returns (bool)
    {
        for (uint256 i = 0; i < transactions[scriptHash].noOfReleases; i++){

            bytes32 addressHash = keccak256(abi.encodePacked(party, i));

            if (transactions[scriptHash].voted[addressHash]){
                return true;
            }
        }

        return false;
    }

    /**
    * @notice Returns an array of scriptHashes associated with trades in which
    * a given address was listed as a buyer or a seller
    * @param partyAddress The address to look up
    * @return an array of scriptHashes
    */
    function getAllTransactionsForParty(
        address partyAddress
    )
        external
        view
        returns (bytes32[] memory)
    {
        return partyVsTransaction[partyAddress];
    }

    /**
    * @notice This method will be used to release funds from the escrow
    * associated with an existing Mobazha transaction.
    * @dev please see the contract specification for more details
    * @param sigV Array containing V component of all the signatures
    * @param sigR Array containing R component of all the signatures
    * @param sigS Array containing S component of all the signatures
    * @param scriptHash ScriptHash of the transaction
    * @param payData Struct containing target destinations and amounts
    */
    function execute(
        uint8[] calldata sigV,
        bytes32[] calldata sigR,
        bytes32[] calldata sigS,
        bytes32 scriptHash,
        PayData calldata payData
    )
        external
        transactionExists(scriptHash)
        fundsExist(scriptHash)
    {
        require(
            payData.destinations.length > 0,
            "No destination"
        );
        require(
            payData.destinations.length == payData.amounts.length,
            "Number of destinations must match number of values sent"
        );

        for (uint256 i = 0; i < payData.destinations.length; i++) {
            require(
                payData.destinations[i] != address(0),
                "zero address is not allowed"
            );

            require(
                payData.amounts[i] > 0,
                "Amount to be sent should be greater than 0"
            );
        }

        _verifyTransaction(
            sigV,
            sigR,
            sigS,
            scriptHash,
            payData.destinations,
            payData.amounts
        );

        transactions[scriptHash].status = Status.RELEASED;

        //solium-disable-next-line security/no-block-members
        transactions[scriptHash].lastModified = block.timestamp;

        transactions[scriptHash].noOfReleases += 1;

        transactions[scriptHash].released += _transferFunds(
            scriptHash,
            payData
        );

        emit Executed(scriptHash, payData.destinations, payData.amounts);

        require(
            transactions[scriptHash].value >= transactions[scriptHash].released,
            "Insufficient balance"
        );
    }

    /**
    * @notice This methods checks validity of a set of signatures AND whether
    * they are sufficient to release funds from escrow
    * @param sigV Array containing V component of all the signatures
    * @param sigR Array containing R component of all the signatures
    * @param sigS Array containing S component of all the signatures
    * @param scriptHash ScriptHash of the transaction
    * @param destinations List of addresses who will receive funds
    * @param amounts List of amounts to be released to the destinations
    * @dev This will revert if the set of signatures is not valid or the
    * attempted payout is not valid. It will succeed silently otherwise
    */
    function _verifyTransaction(
        uint8[] memory sigV,
        bytes32[] memory sigR,
        bytes32[] memory sigS,
        bytes32 scriptHash,
        address payable[] memory destinations,
        uint256[] memory amounts
    )
        private
    {
        _verifySignatures(
            sigV,
            sigR,
            sigS,
            scriptHash,
            destinations,
            amounts
        );

        bool timeLockExpired = _isTimeLockExpired(
            transactions[scriptHash].timeoutHours,
            transactions[scriptHash].lastModified
        );

        //if the minimum number (`threshold`) of signatures are not present and
        //either the timelock has not expired or the release was not signed by
        //the seller then revert
        if (sigV.length < transactions[scriptHash].threshold) {
            if (!timeLockExpired) {
                revert("Min number of sigs not present and timelock not expired");
            }
            else if (
                !transactions[scriptHash].voted[keccak256(
                    abi.encodePacked(
                        transactions[scriptHash].seller,
                        transactions[scriptHash].noOfReleases
                    )
                )]
            )
            {
                revert("Min number of sigs not present and seller did not sign");
            }
        }
    }

    /**
    * @notice Method to transfer funds to a set of destinations
    * @param scriptHash Hash identifying the Mobazha transaction
    * @param payData Struct containing target destinations and amounts
    * @return the total amount of funds that were paid out
    */
    function _transferFunds(
        bytes32 scriptHash,
        PayData memory payData
    )
        private
        returns (uint256)
    {
        Transaction storage t = transactions[scriptHash];

        uint256 valueTransferred = 0;
        if (t.transactionType == TransactionType.ETH) {
            for (uint256 i = 0; i < payData.destinations.length; i++) {
                //add receiver as beneficiary
                t.beneficiaries[payData.destinations[i]] = true;

                payData.destinations[i].transfer(payData.amounts[i]);

                valueTransferred += payData.amounts[i];
            }
        } else if (t.transactionType == TransactionType.TOKEN) {
            ITokenContract token = ITokenContract(t.tokenAddress);
            for (uint256 j = 0; j < payData.destinations.length; j++) {
                //add receiver as beneficiary
                t.beneficiaries[payData.destinations[j]] = true;

                require(
                    token.transfer(payData.destinations[j], payData.amounts[j]),
                    "Token transfer failed."
                );

                valueTransferred += payData.amounts[j];
            }
        }
        return valueTransferred;
    }

    /**
    * @notice Checks whether a given set of signatures are valid
    * @param sigV Array containing V component of all the signatures
    * @param sigR Array containing R component of all the signatures
    * @param sigS Array containing S component of all the signatures
    * @param scriptHash ScriptHash of the transaction
    * @param destinations List of addresses who will receive funds
    * @param amounts List of amounts to be released to the destinations
    * @dev This also records which addresses have successfully signed
    * @dev This function SHOULD NOT be called by ANY function other than
    * `_verifyTransaction`
    */
    function _verifySignatures(
        uint8[] memory sigV,
        bytes32[] memory sigR,
        bytes32[] memory sigS,
        bytes32 scriptHash,
        address payable[] memory destinations,
        uint256[] memory amounts
    )
        private
    {
        require(sigR.length == sigS.length, "R,S length mismatch");
        require(sigR.length == sigV.length, "R,V length mismatch");

        bytes32 txHash = getTransactionHash(
            scriptHash,
            destinations,
            amounts
        );

        for (uint256 i = 0; i < sigR.length; i++) {

            address recovered = ecrecover(
                txHash,
                sigV[i],
                sigR[i],
                sigS[i]
            );

            bytes32 addressHash = keccak256(
                abi.encodePacked(
                    recovered,
                    transactions[scriptHash].noOfReleases
                )
            );

            require(
                transactions[scriptHash].isOwner[recovered],
                "Invalid signature"
            );
            require(
                !transactions[scriptHash].voted[addressHash],
                "Same signature sent twice"
            );
            transactions[scriptHash].voted[addressHash] = true;
        }
    }

    /**
    * @notice Checks whether a timeout has occured
    * @param timeoutHours The number hours after which the seller can
    * unilaterally release funds from escrow. When `timeoutHours` is set to 0
    * it means the seller can never unilaterally release funds from escrow
    * @param lastModified The timestamp of the last modification of escrow for
    * a particular Mobazha transaction
    * @return true if and only if `timeoutHours` hours have passed since
    * `lastModified`
    */
    function _isTimeLockExpired(
        uint32 timeoutHours,
        uint256 lastModified
    )
        private
        view
        returns (bool)
    {
        //solium-disable-next-line security/no-block-members
        uint256 timeSince = block.timestamp - lastModified;
        return (
            timeoutHours == 0 ? false : timeSince > uint256(timeoutHours) * (1 hours)
        );
    }

    /**
    * @dev Private method for adding a new Mobazha transaction to the
    * contract. Used to reduce code redundancy
    * @param buyer The buyer associated with the Mobazha transaction
    * @param seller The seller associated with the Mobazha transaction
    * @param moderator The moderator (if any) associated with the Mobazha
    * transaction
    * @param threshold The minimum number of signatures required to release
    * funds from escrow before the timeout.
    * @param timeoutHours The number hours after which the seller can
    * unilaterally release funds from escrow. When timeoutHours is set to 0
    * it means the seller can never unilaterally release funds from escrow
    * @param scriptHash The keccak256 hash of the redeem script. See
    * specification for more details
    * @param value The amount of currency to add to escrow
    * @param uniqueId A nonce chosen by the buyer
    * @param transactionType Indicates whether the Mobazha trade is using
    * ETH or ERC20 tokens for payment
    * @param tokenAddress The address of the ERC20 token being used for
    * payment. Set to 0 if the Mobazha transaction is settling in ETH
    */
    function _addTransaction(
        address buyer,
        address seller,
        address moderator,
        uint8 threshold,
        uint32 timeoutHours,
        bytes32 scriptHash,
        uint256 value,
        bytes20 uniqueId,
        TransactionType transactionType,
        address tokenAddress
    )
        private
    {
        require(buyer != seller, "Buyer and seller are same");
        require(value > 0, "Value 0");
        require(threshold > 0, "Threshold 0");
        require(threshold <= 3, "Threshold greater than 3");

        //when threshold is 1 that indicates the Mobazha transaction is not
        //being moderated, so `moderator` can be any address
        //if `threadhold > 1` then `moderator` should be nonzero address
        require(
            threshold == 1 || moderator != address(0),
            "Either threshold should be 1 or valid moderator address should be passed"
        );

        require(
            scriptHash == calculateRedeemScriptHash(
                uniqueId,
                threshold,
                timeoutHours,
                buyer,
                seller,
                moderator,
                tokenAddress
            ),
            "Script hash does not match."
        );

        Transaction storage transaction = transactions[scriptHash];
        transaction.buyer = buyer;
        transaction.seller = seller;
        transaction.moderator = moderator;
        transaction.value = value;
        transaction.status = Status.FUNDED;
        //solium-disable-next-line security/no-block-members
        transaction.lastModified = block.timestamp;
        transaction.threshold = threshold;
        transaction.timeoutHours = timeoutHours;
        transaction.transactionType = transactionType;
        transaction.tokenAddress = tokenAddress;
        transaction.released = uint256(0);
        transaction.noOfReleases = uint256(0);

        transaction.isOwner[seller] = true;
        transaction.isOwner[buyer] = true;

        //check if buyer or seller are not passed as moderator
        require(
            !transaction.isOwner[moderator],
            "Either buyer or seller is passed as moderator"
        );

        //the moderator should be an owner only if `threshold > 1`
        if (threshold > 1) {
            transaction.isOwner[moderator] = true;
        }

        transactionCount++;

        partyVsTransaction[buyer].push(scriptHash);
        partyVsTransaction[seller].push(scriptHash);
    }

    /**
    * @notice Gives the hash that the parties need to sign in order to
    * release funds from the escrow of a given Mobazha transactions given
    * a set of destinations and amounts
    * @param scriptHash Script hash of the Mobazha transaction
    * @param destinations List of addresses who will receive funds
    * @param amounts List of amounts for each destination
    * @return a bytes32 hash
    */
    function getTransactionHash(
        bytes32 scriptHash,
        address payable[] memory destinations,
        uint256[] memory amounts
    )
        public
        view
        returns (bytes32)
    {
        //follows ERC191 signature scheme: https://github.com/ethereum/EIPs/issues/191
        bytes32 txHash = keccak256(
            abi.encodePacked(
                "\x19Ethereum Signed Message:\n32",
                keccak256(
                    abi.encodePacked(
                        bytes1(0x19),
                        bytes1(0),
                        address(this),
                        abi.encodePacked(destinations),
                        abi.encodePacked(amounts),
                        scriptHash
                    )
                )
            )
        );
        return txHash;
    }

    /**
    * @notice Calculating scriptHash for a given Mobazha transaction
    * @param uniqueId A nonce chosen by the buyer
    * @param threshold The minimum number of signatures required to release
    * funds from escrow before the timeout.
    * @param timeoutHours The number hours after which the seller can
    * unilaterally release funds from escrow. When timeoutHours is set to 0
    * it means the seller can never unilaterally release funds from escrow
    * @param buyer The buyer associated with the Mobazha transaction
    * @param seller The seller associated with the Mobazha transaction
    * @param moderator The moderator (if any) associated with the Mobazha
    * transaction
    * @param tokenAddress The address of the ERC20 token contract
    * @return a bytes32 hash
    */
    function calculateRedeemScriptHash(
        bytes20 uniqueId,
        uint8 threshold,
        uint32 timeoutHours,
        address buyer,
        address seller,
        address moderator,
        address tokenAddress
    )
        public
        view
        returns (bytes32)
    {
        if (tokenAddress == address(0)) {
            return keccak256(
                abi.encodePacked(
                    uniqueId,
                    threshold,
                    timeoutHours,
                    buyer,
                    seller,
                    moderator,
                    address(this)
                )
            );
        } else {
            return keccak256(
                abi.encodePacked(
                    uniqueId,
                    threshold,
                    timeoutHours,
                    buyer,
                    seller,
                    moderator,
                    address(this),
                    tokenAddress
                )
            );
        }
    }
}
