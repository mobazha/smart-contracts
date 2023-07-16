# Escrow Contract Specification

## Introduction

Mobazha facilitates trades between arbitrary third parties on the internet. Currently, only UTXO-based cryptocurrencies can be used as a medium of exchange on Mobazha. The escrow contract is intended to be used as a way to shoehorn BSC(Binance Smart Chain) functionality into Mobazha's existing framework so that users can trade using ETH and ERC20 tokens as their medium of exchange.

**IMPORTANT:** This contract supports only ETH and _compliant_ ERC20 tokens. Use of the Escrow contract with non-compliant ERC20 tokens may result in permanent loss of tokens. In particular, if the token does not return `true` upon a successful call to `token.transfer` or `token.transferFrom` you should not use the token with this escrow contract. See [this article](https://medium.com/coinmonks/missing-return-value-bug-at-least-130-tokens-affected-d67bf08521ca) for a deeper explanation. We will never present non-complaint tokens as a payment option in the Mobazha UI, but it is still possible to send (and permanently lose) such tokens by interacting with the Escrow contract through a third-party UI.

### How Mobazha Trades Currently Work (in UTXO land)

#### Moderated Payments

When a buyer and seller have agreed on a product and a price, the buyer sends their funds to an escrow address, which is a 2-of-3 multisig address with one key controlled by the buyer, one key controlled by the seller, and one key controlled by a moderator that has been agreed upon by both the buyer and the seller.

**IMPORTANT:** This contract requires _signatures_ in order to release funds from escrow. Contracts cannot create signatures corresponding to their own addresses. Therefore, you SHOULD NOT pass a contract address for the `buyer`, `seller`, or `moderator`. Doing so could make it impossible for your funds to be released from escrow.

On the "happy path", the seller delivers the goods, then the buyer releases the funds to the seller (with the buyer and seller signing the payout txn from the escrow address).

In the event that the seller does not deliver the goods as promised, the buyer pleads their case to the moderator, and the buyer & moderator can send the funds from escrow back to the buyer.

In the (very common) case where the buyer receives their goods but doesn't release the funds to the seller, the seller pleads their case to the moderator, and the seller & moderator sign the funds from escrow to the seller.

The seller can also unilaterally release funds from escrow after a previously agreed upon amount of time has passed. This allows the seller to release the funds from escrow without the moderator in the event that the buyer disappears. With UTXO-based coins, this is achieved by requiring that the buyer sign an nLockTime transaction releasing funds to the seller, and then passing that txn to the seller (off-chain) before the seller delivers the product or service.

#### Direct Payments

Buyers have the option of _not_ using a moderator when making an Mobazha trade. While this isn't recommended, it may be an acceptable risk for the buyer if the buyer trusts the seller. Direct/unmoderated payments come in two forms: online payments and offline payments.

Online direct payments occur when the buyer knows the seller is online. For online payments, the buyer simply sends the funds directly to the sellers wallet. These are simple, classic transfers of value from one account to another.

Offline payments occur when the buyer sees that the seller is offline and is _uncertain_ whether the seller will ever come back online. In this case the buyer sends the funds to a 1-of-2 multisig address with one key held by the buyer and the other held by the seller. If the seller comes back online, they can accept the funds. If the seller doesn't come back online, the buyer can reclaim the funds.

### Limitations Imposed by Mobazha's Wallet Interface

Mobazha interacts with all supported coins through its [wallet interface](https://github.com/Mobazha/wallet-interface/blob/master/wallet.go#L77). This means that Mobazha's BSC smart contracts must be designed in such a way as to be compatible with that interface. Mobazha is a live/launched product, so making big changes to the wallet interface in order to support BSC is non-trivial. Instead, we've decided to keep the wallet interface fixed (for now), and design the smart contract to be compatible with it.

## Intended Use of the Escrow contract

The Escrow contract will store the escrowed funds and state information for _every_ Mobazha trade that is using BSC (or ERC20 tokens) as the medium of exchange. (We could have, instead, opted to deploy a new escrow contract for each BSC-based trade -- thereby siloing escrowed funds from each trade in their own smart contract. However, we think the gas requirements for doing so are cost prohibitive, and we fear that would introduce too much friction into BSC-based trades). Mobazha trades that use ETH/ERC20 as the medium of exchange are intended to follow the same protocol as those that use a UTXO-based coin as the medium of exchange -- and the escrow smart contract is intended to facilitate that.

### Funding the Trade

Buyers initiate a trade by creating/storing a _Transaction_ struct in the Escrow contract and (simultaneously) funding the transaction by sending ETH (or ERC20 tokens) to the Escrow contract. At this point the transaction is in the _FUNDED_ state. While in the _FUNDED_ state, the buyer may add more ETH (or ERC20 tokens) to escrow if necessary. Adding more funds to escrow _does not_ result in any changes to _timeoutHours_ (see next section).

### Releasing Funds from Escrow

While the transaction is in the _FUNDED_ state, the escrowed funds can be released only if: (1) Two of the three participants (buyer, seller, and moderator) agree on how the escrowed funds are to be distributed, or (2) an amount of time (_timeoutHours_) has passed since the last time the buyer added funds to escrow. (Note: when _timeoutHours_ is set to 0, this indicates an infinite timeout, not an instantaneous timeout. In other words, if _timeoutHours_ is set to 0 then the seller can never unilaterally release funds from escrow.)

The reasoning behind (2) is that it is very common for buyers to not release funds after they've received their goods (this is due more to buyer laziness than malice). In that event, we want to make it easy for the seller to claim the escrowed funds without having to coordinate with a moderator.

Funds released from escrow can be split up and sent to various addresses. However, the receiving addresses _must be_ the addresses of the trade's buyer, seller, or moderator. To reiterate, funds cannot be sent to an address that is not affiliated with the trade in question, but the escrowed funds can be divided up among the participants in any way -- so long as 2-of-3 of the parties agree.

Upon release of funds from escrow, the trade is put into the _RELEASED_ state. Once in the _RELEASED_ state, trades can no longer be altered. All participants who received some of the escrowed funds are noted in the trade's _Transaction_ struct (via the _beneficiaries_ mapping).

(The _beneficiaries_ information will be used later, by other contracts, to determine whether or not a given trade was disputed, refunded, etc).

If there are ever any funds left in escrow (even if the trade is in the _RELEASED_ state) the party's can call _execute_ to release the funds.

### Offline Direct Payments

The escrow contract can mirror the behavior of UTXO-based offline payments by calling `addTransaction` (or `addTokenTransaction` if it is an ERC20 transaction), setting the `threshold` value to 1, and setting the moderator address to a known, non-zero burn address. The effect is the equivalent of a 1-of-2 multisig address where the buyer holds one key and the seller holds the other.

## Known Issues / Misc

It is assumed that the moderator is trusted by both the buyer and the seller before the trade begins. The obvious threat of collusion between a buyer and moderator -- or seller and moderator -- is beyond the scope of this contract.

The _transferFunds_ function uses push payments (rather than the pull model) due to limitations imposed by Mobazha's wallet interface. Hence any of the beneficiaries of a payout from escrow can cause the payout to fail (for example, by putting a _revert()_ in their fallback function). Game theoretically speaking, such a DoS attack is irrational for any of the participants capable of causing such an issue, because the honest parties can always benefit by removing the offending party as a beneficiary and taking her share of the payout.

(For example, suppose the three parties agreed that the moderator would received 5% of the funds, and that the buyer and seller would split the remaining funds. The seller, being unhappy with the result, could cause the payout to fail until she could negotiate a more favorable agreement. However, the buyer & moderator -- upon seeing the seller's misbehavior -- could simply agree to remove the seller as a beneficiary -- thus removing the seller's ability to DoS the payout.)

For this reason, we consider the DoS possibility caused by use of push payments in the _transferFunds_ function to be low risk.
