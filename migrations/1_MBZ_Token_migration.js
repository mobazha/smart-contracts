var MBZToken = artifacts.require("MBZToken");

module.exports = function(deployer) {
  deployer.deploy(MBZToken, "Mobazha", "MBZ", 18, 1000000000);
};