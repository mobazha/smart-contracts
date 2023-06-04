var MGLToken = artifacts.require("MGLToken");

module.exports = function(deployer) {
  deployer.deploy(MGLToken, "Mogaolei", "MGLT", 18, 1000000000);
};