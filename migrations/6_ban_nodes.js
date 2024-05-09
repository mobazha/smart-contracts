var StringArray = artifacts.require("StringArray");
var BanNodes = artifacts.require("BanNodes");

module.exports = async(deployer) => {
  await deployer.deploy(StringArray);
  await deployer.link(StringArray, BanNodes);

  deployer.deploy(BanNodes);
};

// Result:
// Amoy, StringArray tx: 0xfd5e3d92cfd77629794c26dac585f062316503a94a3117f1f1a81424da87f856 , BanNodes tx: 0x5c5218f49a3e054daaf28e5d3ad37524bc119d09c53d0eb6b8f9ac89b27b5657