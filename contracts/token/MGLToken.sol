pragma solidity 0.8.4;

import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";


contract MGLToken is ERC20Burnable {

    constructor(
        string memory name,
        string memory symbol,
        uint8 _decimals,
        uint256 _totalSupply
    )
        ERC20(name, symbol)
    {
        _mint(msg.sender, _totalSupply * (10 ** uint256(_decimals)));
    }

}
