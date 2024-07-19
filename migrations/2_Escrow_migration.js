var ScriptHashCalculator = artifacts.require("ScriptHashCalculator");
var Escrow = artifacts.require("Escrow");

async function doDeploy(deployer) {
  await deployer.deploy(ScriptHashCalculator);
  await deployer.link(ScriptHashCalculator, Escrow);

  // const calculatorInstance = await ScriptHashCalculator.at('0x3DAe8BD5972D7D83A9661E13becd0C2dA9177F3B');
  // await deployer.link(calculatorInstance, Escrow);

  // The second param is the MBZ token address
  // BSC: 0xBAD8470f50575Ac41d4FE1C31039554112d31E89
  // Mumbai/Mainnet: 0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991
  // ConfluxTestnet/Mainnet: 0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991
  await deployer.deploy(Escrow, "0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991");
}

module.exports = async(deployer) =>{
  await doDeploy(deployer);
};

// module.exports = async(deployer) =>{
//   await deployer.deploy(Escrow, "0xBAD8470f50575Ac41d4FE1C31039554112d31E89");
// };