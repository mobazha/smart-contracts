# Rewards Contract Specification

## Introduction

Mobazha will occasionally hold 'promotions' where users who buy goods from "promoted sellers" become eligible for reward tokens (MGLT). Each of these promotions will be managed by an instance of the MGLRewards contract.

## Time Limits

When a buyer purchases from a promoted seller they become eligible to receive up to 50 MGLT from the rewards contract. The buyer has a fixed amount of time (`timeWindow` seconds) after the completion of the sale to claim their reward tokens from the contract.

The promotion as a whole has an `endDate`, which is set (and changeable) by the owner. After the promotion's `endDate` has come to pass, buyers can no longer claim any rewards.

## Claiming Rewards

The buyer can claim tokens for which she is eligible in on of two ways:

1.  By calling `claimRewards`, the buyer can pass a list of MGL transactions (identified by their `scriptHash`). The contract ensures that each of the passed transactions do indeed make the buyer eligible for some reward tokens, computes the total amount of tokens the buyer is eligible to receive, and sends that amount of reward tokens to the buyer. (If the contract does not have enough reward tokens remaining, it will send the buyer all of the tokens it has. Then, if Mobazha sends more reward tokens to the contract, the buyer should be able to claim whatever remaining tokens they are owed -- assuming they are still eligible to receive the tokens.)

2.  By calling `executeAndClaim` the buyer can complete their trade with the seller and claim any rewards with a single transaction.

## Limits on Reward Amounts

Each buyer may be rewarded up to 50 reward tokens for purchasing from a given promoted seller. That is, if buyer Bob buys from promoted seller Sally, he'll be eligible for up to 50 reward tokens, but if he buys from her again during the same promotion, he will not be eligible for an additional 50 reward tokens. If Bob wants to earn more tokens during the same promotion, he'd have to complete a purchase from some other promoted seller.

Additionally, the owner of the contract sets a maximum total number of tokens that can be rewarded for purchasing from any given promoted seller (`maxRewardPerSeller`). For example, suppose `maxRewardPerSeller` is 500 MGLT and that each buyer is eligible to receive up to 50 MGLT for purchasing from a given promoted seller. Then if Alice is a promoted seller, at most 10 buyers can receive rewards from purchasing from Alice.

## Additional Notes

- Any reward tokens remaining in the contract can be withdrawn from the contract by the owner.

- This approach to promoting sellers is subject to trivially-executable sybil attacks, as well as buyer collusion with promoted sellers. The limits on the rewards and the restriction to Mobazha-promoted sellers are intended to mitigate this risk.

- If a buyer is eligible to receive more MGLT than remains in the contract balance, then when the buyer attempts to claim their reward, the contract should pay out as much MGLT as it can to the buyer. If the contract later receives more MGLT, then the buyer should be able to claim the remainder of their reward.
