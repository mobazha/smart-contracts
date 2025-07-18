// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.22;

library StringArray {
  struct StringValueArray {
    string[] values;
  }

  function add(StringValueArray storage self, string memory value) public {
    // Check for duplicates
    for (uint i = 0; i < self.values.length; i++) {
      if (keccak256(abi.encodePacked(self.values[i])) == keccak256(abi.encodePacked(value))) {
        return; // Already exists
      }
    }
    self.values.push(value);
  }

  function remove(StringValueArray storage self, string memory value) public {
    uint i = 0;
    for (; i < self.values.length; i++) {
      if (keccak256(abi.encodePacked(self.values[i])) == keccak256(abi.encodePacked(value))) {
        break;
      }
    }

    // Shift elements if value found
    if (i < self.values.length) {
      for (; i < self.values.length - 1; i++) {
        self.values[i] = self.values[i + 1];
      }
      self.values.pop();
    }
  }

  // Due to this API is called frequently, we don't use map for the values, as it iterates over the entire map to get keys,
  // which might be less gas-efficient especially for very large datasets
  function get(StringValueArray storage self) public view returns (string[] memory) {
    return self.values;
  }
}

contract BanNodes {
  using StringArray for StringArray.StringValueArray;
  StringArray.StringValueArray internal blockedIds;

  function addBlockedID(string memory value) public {
    blockedIds.add(value);
  }

  function removeBlockedID(string memory value) public {
    blockedIds.remove(value);
  }

  function getBlockedIds() public view returns (string[] memory) {
    return blockedIds.get();
  }
}
