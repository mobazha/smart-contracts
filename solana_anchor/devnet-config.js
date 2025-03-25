import os from 'os';

// 检测当前环境
const isDevnet = process.env.ANCHOR_PROVIDER_URL?.includes('devnet') || 
                 process.env.SOLANA_CLUSTER === 'devnet';

// 本地环境配置
const localConfig = {
  connection: "http://localhost:8899",
  useRandomAccounts: true,
  testAmounts: {
    escrow: 1 * 1000000000,    // 1 SOL
    token: 2 * 1000000000,     // 2 SOL
    multi: 0.8 * 1000000000,   // 0.8 SOL
    simple: 0.5 * 1000000000   // 0.5 SOL
  }
};

// Devnet 环境配置
const devnetConfig = {
  connection: "https://api.devnet.solana.com",
  useRandomAccounts: false,
  keypairPaths: {
    main: `${os.homedir()}/.config/solana/id.json`,
    buyer: "./keypairs/buyer.json",
    seller: "./keypairs/seller.json",
    moderator: "./keypairs/moderator.json"
  },
  testAmounts: {
    escrow: 0.1 * 1000000000,  // 0.1 SOL
    token: 0.2 * 1000000000,   // 0.2 SOL
    multi: 0.08 * 1000000000,  // 0.08 SOL
    simple: 0.05 * 1000000000  // 0.05 SOL
  }
};

// 导出当前环境的配置
export default isDevnet ? devnetConfig : localConfig; 