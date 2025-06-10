// SPDX-License-Identifier: MBZ
pragma solidity ^0.8.22;

enum Role {BUYER, VENDOR, MODERATOR, PLATFORM}

struct PayData {
    // List of addresses who will receive funds
    address payable[] destinations;
    // List of amounts to be released to the destinations
    uint256[] amounts;
    // List of user roles of the destinations
    Role[] roles;
}