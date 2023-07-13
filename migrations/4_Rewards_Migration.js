var MGLRewards = artifacts.require("MGLRewards");
var Escrow = artifacts.require("Escrow");
var MBZToken = artifacts.require("MBZToken");

module.exports = function(deployer) {
  //This is dummy data
  deployer.deploy(
    MGLRewards,
    "50000000000000000000",
    432000,
    Escrow.address,
    MBZToken.address
  );
};
