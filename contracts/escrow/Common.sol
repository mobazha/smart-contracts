// SPDX-License-Identifier: MBZ
pragma solidity ^0.8.22;

struct PayData {
    // List of addresses who will receive funds
    address payable[] destinations;
    // List of amounts to be released to the destinations
    uint256[] amounts;
}