#!/bin/bash

# 确保在 devnet
solana config set --url devnet

# 检查余额
BALANCE=$(solana balance)
if (( $(echo "$BALANCE < 1" | bc -l) )); then
    echo "余额不足，正在获取测试代币..."
    solana airdrop 2
fi

# 构建程序
echo "开始构建 Anchor 程序..."
anchor build

# 获取程序 ID
PROGRAM_ID=$(solana address -k ./target/deploy/escrow_program-keypair.json)
echo "程序 ID: $PROGRAM_ID"

# 更新 Anchor.toml 中的程序 ID
sed -i.bak "s/你的程序ID/$PROGRAM_ID/g" Anchor.toml
# 更新 lib.rs 中的程序 ID
sed -i.bak "s/你的程序ID/$PROGRAM_ID/g" programs/escrow-program/src/lib.rs

# 重新构建程序（更新了程序 ID 后）
echo "更新程序 ID 后重新构建..."
anchor build

# 部署程序
echo "部署程序到 devnet..."
anchor deploy

# 保存程序 ID 到文件
echo $PROGRAM_ID > ./target/deploy/program-id.txt
echo "程序已成功部署到 devnet，程序 ID 已保存至 ./target/deploy/program-id.txt" 