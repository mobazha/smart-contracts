#!/bin/bash

# 生成 Escrow 的 Go 绑定
abigen --abi=abigenBindings/abi/Escrow.abi \
       --pkg=Escrow \
       --out=abigenBindings/escrow.go 

# 生成 Escrow 的 Go 绑定
abigen --abi=abigenBindings/abi/ERC20.abi \
       --pkg=Token \
       --out=abigenBindings/erc20.go 

# 生成 ContractManager 的 Go 绑定
abigen --abi=abigenBindings/abi/ContractManager.abi \
       --pkg=Registry \
       --out=abigenBindings/contract_manager.go
