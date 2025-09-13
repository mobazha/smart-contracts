# Solana Contract Manager

这是一个类似以太坊ContractManager的Solana版本管理系统，用于管理合约版本和地址，让Go后台可以动态获取最新的合约地址，避免硬编码。

## 功能特性

- ✅ 合约版本管理
- ✅ 推荐版本设置
- ✅ 版本状态跟踪（Beta, RC, Production, Deprecated）
- ✅ Bug级别跟踪（None, Low, Medium, High, Critical）
- ✅ Go客户端动态获取合约地址
- ✅ 完全去中心化，无需硬编码

## 架构设计

### 程序结构
```
contract-manager/
├── src/
│   ├── lib.rs              # 主程序入口
│   ├── state.rs            # 状态定义
│   ├── error.rs            # 错误定义
│   └── instructions/       # 指令实现
│       ├── initialize.rs
│       ├── add_version.rs
│       ├── update_version.rs
│       ├── mark_recommended.rs
│       └── remove_recommended.rs
└── Cargo.toml
```

### 数据结构

#### ContractManager
```rust
pub struct ContractManager {
    pub authority: Pubkey,        // 管理员地址
    pub contracts: Vec<Contract>, // 合约列表
    pub bump: u8,                // PDA bump
}
```

#### Contract
```rust
pub struct Contract {
    pub contract_name: String,           // 合约名称
    pub versions: Vec<Version>,          // 版本列表
    pub recommended_version: Option<String>, // 推荐版本
}
```

#### Version
```rust
pub struct Version {
    pub version_name: String,    // 版本名称
    pub status: ContractStatus,  // 状态
    pub bug_level: BugLevel,     // Bug级别
    pub program_id: Pubkey,      // 程序ID
    pub date_added: i64,         // 添加时间
}
```

## 部署和使用

### 1. 部署程序

```bash
# 部署到devnet
./scripts/deploy-contract-manager.sh devnet

# 部署到mainnet
./scripts/deploy-contract-manager.sh mainnet
```

### 2. 初始化合约管理器

```bash
anchor run initialize-contract-manager
```

### 3. 添加合约版本

```bash
# 添加escrow程序v1.0版本
anchor run add-version escrow_program v1.0 production 25ecY9sGUkFyy78aYaSbdWGMgySSKZvPjQunf6Uk23qk
```

### 4. 设置推荐版本

```bash
anchor run mark-recommended escrow_program v1.0
```

## Go客户端使用

### 基本用法

```go
package main

import (
    "context"
    "log"
    "github.com/portto/solana-go-sdk/common"
)

func main() {
    // 初始化客户端
    rpcEndpoint := "https://api.devnet.solana.com"
    programID := common.PublicKeyFromString("ContractManager111111111111111111111111111111111")
    client := NewContractManagerClient(rpcEndpoint, programID)

    ctx := context.Background()

    // 获取推荐版本
    recommendedVersion, err := client.GetRecommendedVersion(ctx, "escrow_program")
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("推荐版本: %s\n", recommendedVersion.VersionName)
    fmt.Printf("程序ID: %s\n", recommendedVersion.ProgramID.ToBase58())
    fmt.Printf("状态: %s\n", recommendedVersion.Status.String())
}
```

### 高级用法

```go
// 获取所有版本
versions, err := client.GetContractVersions(ctx, "escrow_program")
if err != nil {
    log.Fatal(err)
}

// 获取特定版本的程序ID
programID, err := client.GetProgramID(ctx, "escrow_program", "v1.0")
if err != nil {
    log.Fatal(err)
}

// 获取完整的合约管理器状态
manager, err := client.GetContractManager(ctx)
if err != nil {
    log.Fatal(err)
}
```

## 测试

运行测试套件：

```bash
# 运行所有测试
anchor test

# 只运行合约管理器测试
anchor test --skip-local-validator
```

## 程序ID

- **Contract Manager Program ID**: `ContractManager111111111111111111111111111111111`
- **Escrow Program ID**: `25ecY9sGUkFyy78aYaSbdWGMgySSKZvPjQunf6Uk23qk`

## 网络配置

### Devnet
- RPC: `https://api.devnet.solana.com`
- 用于开发和测试

### Mainnet
- RPC: `https://api.mainnet-beta.solana.com`
- 用于生产环境

## 安全考虑

1. **权限控制**: 只有authority可以管理合约版本
2. **数据验证**: 所有输入都经过严格验证
3. **PDA安全**: 使用程序派生地址确保安全性
4. **版本控制**: 防止重复版本和无效数据

## 错误处理

程序定义了完整的错误类型：

```rust
pub enum ContractManagerError {
    EmptyContractName,
    EmptyVersionName,
    ContractAlreadyExists,
    VersionAlreadyExists,
    ContractNotFound,
    VersionNotFound,
    NoRecommendedVersion,
    InvalidProgramId,
    Unauthorized,
    NotInitialized,
}
```

## 最佳实践

1. **版本命名**: 使用语义化版本号（如v1.0, v1.1, v2.0）
2. **状态管理**: 新版本从Beta开始，逐步升级到Production
3. **推荐版本**: 只推荐经过充分测试的Production版本
4. **监控**: 定期检查合约状态和Bug级别
5. **备份**: 保留重要版本的备份

## 故障排除

### 常见问题

1. **程序未初始化**
   - 确保先调用initialize指令

2. **权限不足**
   - 确保使用正确的authority账户

3. **版本不存在**
   - 检查版本名称是否正确

4. **网络连接问题**
   - 检查RPC端点是否可用

### 调试技巧

1. 使用Solana CLI检查账户状态
2. 查看程序日志获取详细信息
3. 使用Anchor测试框架进行本地测试

## 贡献

欢迎提交Issue和Pull Request来改进这个系统。

## 许可证

本项目采用MIT许可证。
