// SPDX-License-Identifier: MBZ
pragma solidity 0.8.4;

import "../token/ITokenContract.sol";

library ScriptHashCalculator {
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
                        // transactions[scriptHash].noOfReleases,
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

    function calculatePlatformFee(
        uint256 amount,
        bool isToken,
        address tokenAddress
    )
        public
        view
        returns (uint256)
    {
        // 1) Since currently we cannot get exchange rate in contract, we cannot evaluate
        //    min and max fee for mainnet coin (ETH). We simply use 1% for the fee.
        // 2) For USDT and USDC tokens, pay 1% of vendor funds to the platform, 
        //    with 0.5 USD in minimum and 100 USD in maximum.

        uint256 valuePlatform = amount * 1 / 100;
        if (!isToken) {
            return valuePlatform;
        }

        ITokenContract token = ITokenContract(tokenAddress);

        // Pay 1% of vendor funds to the platform, 0.5 USD in minimum and 100 USD in maximum.
        uint256 minFee = 1 * 10**(token.decimals()) / 2;
        uint256 maxFee = 100 * 10**(token.decimals());

        // If amount is less than minFee, use 1%
        
        if (amount >= minFee) {
            if (valuePlatform < minFee) {
                valuePlatform = minFee;
            } else if (valuePlatform > maxFee) {
                valuePlatform = maxFee;
            }
        }
        return valuePlatform;
    }
}
