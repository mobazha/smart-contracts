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
cargo build-sbf --manifest-path=./Cargo.toml --sbf-out-dir=./target/deploy

# 部署程序
PROGRAM_ID=$(solana program deploy ./target/deploy/time_locked_multisig.so)
echo "程序已部署，程序ID: $PROGRAM_ID"

# 保存程序ID到文件
echo $PROGRAM_ID > ./target/deploy/program-id.txt 