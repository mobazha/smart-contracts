// SPDX-License-Identifier: MBZ
pragma solidity 0.8.4;

enum Role {BUYER, VENDOR, MODERATOR}

struct PayData {
    // List of addresses who will receive funds
    address payable[] destinations;
    // List of amounts to be released to the destinations
    uint256[] amounts;
    // List of user roles of the destinations
    Role[] roles;
}

enum OrderFinishType {
    // Buyer has completed the order
    COMPLETE,
    // Vendor cancel the order
    CANCEL,
    // Vendor refunded the order
    REFUND,
    // The winning party has accepted the dispute and it is now complete
    RESOLVED,
    // For executeAndClaim in MGLRewards
    OTHER
}