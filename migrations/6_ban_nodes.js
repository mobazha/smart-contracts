var StringArray = artifacts.require("StringArray");
var BanNodes = artifacts.require("BanNodes");

module.exports = async(deployer) => {
  await deployer.deploy(StringArray);
  await deployer.link(StringArray, BanNodes);

  deployer.deploy(BanNodes);
};

// Result:
// Amoy, StringArray tx: 0xfd5e3d92cfd77629794c26dac585f062316503a94a3117f1f1a81424da87f856 , BanNodes tx: 0x5c5218f49a3e054daaf28e5d3ad37524bc119d09c53d0eb6b8f9ac89b27b5657
// Mainnet, StringArray tx: 0x7d7d66fd1e560687565d58520366edcac1f135683f92e865ed8202de7d7bdf0a , BanNodes tx: 0x688666cc5d7d41e607294d9c3d067f9cbc57c4f13053a6dfef4390995d0578ef