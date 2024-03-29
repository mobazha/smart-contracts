var ScriptHashCalculator = artifacts.require("ScriptHashCalculator");
var Escrow = artifacts.require("Escrow");

async function doDeploy(deployer) {
  // await deployer.deploy(ScriptHashCalculator);
  // await deployer.link(ScriptHashCalculator, Escrow);

  const calculatorInstance = await ScriptHashCalculator.at('0x393b2FEfA82aB9ddFd7AF920C24A9dB0B27388c7');
  await deployer.link(calculatorInstance, Escrow);

  // The second param is the MBZ token address
  // BSC: 0xBAD8470f50575Ac41d4FE1C31039554112d31E89
  // Mumbai: 0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991
  // ConfluxTestnet/Mainnet: 0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991
  await deployer.deploy(Escrow, "0x4c1A1b21c4471CA57145EE08404Cbaf9C8B83991");
}

module.exports = async(deployer) =>{
  await doDeploy(deployer);
};

// module.exports = async(deployer) =>{
//   await deployer.deploy(Escrow, "0xBAD8470f50575Ac41d4FE1C31039554112d31E89");
// };