var MGLRewards = artifacts.require("MGLRewards");
var Escrow = artifacts.require("Escrow");
var MGLToken = artifacts.require("MGLToken");

module.exports = function(deployer) {
  //This is dummy data
  deployer.deploy(
    MGLRewards,
    "50000000000000000000",
    432000,
    Escrow.address,
    MGLToken.address
  );
};
