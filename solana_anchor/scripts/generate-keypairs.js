import { Keypair } from "@solana/web3.js";
import fs from 'fs';
import path from 'path';

// 创建 keypairs 目录
const keypairsDir = path.join(process.cwd(), 'keypairs');
if (!fs.existsSync(keypairsDir)) {
  fs.mkdirSync(keypairsDir);
}

// 生成测试账户
const accounts = ['buyer', 'seller', 'moderator'];
accounts.forEach(account => {
  const keypair = Keypair.generate();
  const filePath = path.join(keypairsDir, `${account}.json`);
  fs.writeFileSync(filePath, JSON.stringify(Array.from(keypair.secretKey)));
  console.log(`已生成 ${account} 账户: ${keypair.publicKey.toString()}`);
}); 