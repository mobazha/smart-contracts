const MyToken = artifacts.require("USDT");

module.exports = (deployer) => {
    deployer.deploy(MyToken, "MyUSDT", "USDT", 1000000000);
};