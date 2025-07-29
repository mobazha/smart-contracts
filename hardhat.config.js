require("@nomicfoundation/hardhat-toolbox");
require("@nomicfoundation/hardhat-verify");
require("dotenv").config();

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.22",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
  paths: {
    sources: "./contracts/rwa-marketplace",
    tests: "./contracts/rwa-marketplace/test",
    cache: "./cache",
    artifacts: "./artifacts"
  },
  networks: {
    hardhat: {
      chainId: 1337,
    },
    // 兼容Truffle配置的网络
    development: {
      url: "http://127.0.0.1:8545",
      host: "127.0.0.1",
      port: 8545,
      network_id: "*",
    },
    ethereumTestnet: {
      url: `https://eth-sepolia.g.alchemy.com/v2/${process.env.alchemy_PROJECT_ID}`,
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 11155111,
      confirmations: 2,
      timeoutBlocks: 200,
      skipDryRun: true,
    },
    sepolia: {
      url: `https://eth-sepolia.g.alchemy.com/v2/${process.env.alchemy_PROJECT_ID}`,
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 11155111,
      confirmations: 2,
      timeoutBlocks: 200,
      skipDryRun: true,
      gasPrice: 20000000000, // 20 gwei
    },
    bscTestnet: {
      url: "https://data-seed-prebsc-1-s1.binance.org:8545/",
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 97,
      confirmations: 10,
      timeoutBlocks: 200,
      skipDryRun: true,
    },
    bscMainnet: {
      url: "https://bsc-dataseed.binance.org/",
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 56,
      confirmations: 10,
      timeoutBlocks: 200,
      skipDryRun: true,
    },
    polygonTestnet: {
      url: "https://rpc-amoy.polygon.technology",
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 80002,
      confirmations: 10,
      timeoutBlocks: 200,
      pollingInterval: 1800000,
      disableConfirmationListener: true,
      skipDryRun: true,
    },
    polygonMainnet: {
      url: "https://polygon-bor-rpc.publicnode.com",
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 137,
      gas: 16721975,
      confirmations: 10,
      timeoutBlocks: 200,
      skipDryRun: true,
    },
    confluxTestnet: {
      url: "https://evmtestnet.confluxrpc.com",
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 71,
      confirmations: 3,
      timeoutBlocks: 200,
      pollingInterval: 1800000,
      disableConfirmationListener: true,
      skipDryRun: true,
    },
    confluxMainnet: {
      url: "https://evm.confluxrpc.com",
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 1030,
      confirmations: 2,
      timeoutBlocks: 200,
      pollingInterval: 1800000,
      disableConfirmationListener: true,
      skipDryRun: true,
    },
    // 其他网络配置
    goerli: {
      url: process.env.GOERLI_RPC_URL || "https://goerli.infura.io/v3/YOUR_PROJECT_ID",
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 5,
    },
    mainnet: {
      url: process.env.MAINNET_RPC_URL || "https://mainnet.infura.io/v3/YOUR_PROJECT_ID",
      accounts: process.env.MNEMONIC ? { mnemonic: process.env.MNEMONIC } : [],
      network_id: 1,
    },
  },
  sourcify: {
    enabled: true,
  },
  etherscan: {
    apiKey: {
      sepolia: process.env.ETHERSCAN_API_KEY || "",
      goerli: process.env.ETHERSCAN_API_KEY || "",
      mainnet: process.env.ETHERSCAN_API_KEY || "",
      bscTestnet: process.env.BSCSCAN_API_KEY || "",
      bsc: process.env.BSCSCAN_API_KEY || "",
      polygon: process.env.POLYGONSCAN_API_KEY || "",
      polygonMumbai: process.env.POLYGONSCAN_API_KEY || "",
    },
  },
  gasReporter: {
    enabled: process.env.REPORT_GAS !== undefined,
    currency: "USD",
  },
}; 