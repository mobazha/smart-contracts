var ScriptHashCalculator = artifacts.require("ScriptHashCalculator");
var Escrow = artifacts.require("Escrow");

async function doDeploy(deployer) {
  await deployer.deploy(ScriptHashCalculator);
  await deployer.link(ScriptHashCalculator, Escrow);
  // The second param is the MBZ token address
  // BSC: 0xBAD8470f50575Ac41d4FE1C31039554112d31E89
  // Mumbai: 0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991
  // ConfluxTestnet: 0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991
  await deployer.deploy(Escrow, "0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991");
}

module.exports = async(deployer) =>{
  await doDeploy(deployer);
};

// module.exports = async(deployer) =>{
//   await deployer.deploy(Escrow, "0xBAD8470f50575Ac41d4FE1C31039554112d31E89");
// };