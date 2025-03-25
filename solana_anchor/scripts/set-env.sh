#!/bin/bash

# 设置本地环境
function set_local() {
  export SOLANA_CLUSTER=localnet
  export ANCHOR_PROVIDER_URL=http://localhost:8899
  echo "已设置为本地环境"
}

# 设置 Devnet 环境
function set_devnet() {
  export SOLANA_CLUSTER=devnet
  export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
  echo "已设置为 Devnet 环境"
}

# 根据参数设置环境
if [ "$1" == "local" ]; then
  set_local
elif [ "$1" == "devnet" ]; then
  set_devnet
else
  echo "用法: source scripts/set-env.sh [local|devnet]"
  echo "当前环境: $SOLANA_CLUSTER"
fi 