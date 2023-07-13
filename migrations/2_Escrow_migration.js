var ScriptHashCalculator = artifacts.require("ScriptHashCalculator");
var Escrow = artifacts.require("Escrow");

async function doDeploy(deployer) {
  await deployer.deploy(ScriptHashCalculator);
  await deployer.link(ScriptHashCalculator, Escrow);
  await deployer.deploy(Escrow, "0xBAD8470f50575Ac41d4FE1C31039554112d31E89");
}


module.exports = async(deployer) =>{
  await doDeploy(deployer);
};

// module.exports = async(deployer) =>{
//   await deployer.deploy(Escrow, "0xBAD8470f50575Ac41d4FE1C31039554112d31E89");
// };