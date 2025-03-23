package escrow

import (
	"bytes"

	"github.com/gagliardetto/binary"
	"github.com/gagliardetto/solana-go"
)

// 指令类型枚举
type InstructionType uint8

const (
	InstructionInitialize InstructionType = iota
	InstructionDeposit
	InstructionSign
	InstructionRelease
)

// TokenType 代表代币类型
type TokenType struct {
	IsSpl bool
	Mint  solana.PublicKey // 如果是SPL代币，这里是mint地址
}

// 支付目标结构
type PaymentTarget struct {
	Recipient solana.PublicKey
	Amount    uint64
}

// 初始化指令数据
type InitializeInstructionData struct {
	Instruction     InstructionType
	UnlockHours     uint64
	RequiredSigners uint8
	HasModerator    bool
	TokenType       TokenType
}

// 存款指令数据
type DepositInstructionData struct {
	Instruction InstructionType
	Amount      uint64
}

// 释放指令数据
type ReleaseInstructionData struct {
	Instruction    InstructionType
	PaymentTargets []PaymentTarget
}

// 账户状态枚举
type EscrowState uint8

const (
	EscrowStateUninitialized EscrowState = iota
	EscrowStateActive
	EscrowStateCompleted
)

// 账户大小常量 - 与 Rust 程序中的 Escrow::LEN 保持一致
const ACCOUNT_LEN = 256

// 账户数据结构大小
const (
	PUBKEY_SIZE        = 32 // Solana公钥大小
	OPTION_PUBKEY_SIZE = 33 // 可选公钥大小 (1字节标志 + 32字节公钥)
	U64_SIZE           = 8  // uint64大小
	U8_SIZE            = 1  // uint8/bool大小
)

// 创建初始化指令
func NewInitializeInstruction(
	programID solana.PublicKey,
	buyer solana.PublicKey,
	escrowAccount solana.PublicKey,
	seller solana.PublicKey,
	moderator *solana.PublicKey,
	unlockHours uint64,
	requiredSigners uint8,
	tokenType TokenType,
) solana.Instruction {
	accounts := []*solana.AccountMeta{
		solana.NewAccountMeta(buyer, true, true),                      // 买家账户
		solana.NewAccountMeta(escrowAccount, false, true),             // 托管账户
		solana.NewAccountMeta(seller, false, false),                   // 卖家账户
		solana.NewAccountMeta(solana.SystemProgramID, false, false),   // 系统程序
		solana.NewAccountMeta(solana.SysVarRentPubkey, false, false),  // 租金账户
		solana.NewAccountMeta(solana.SysVarClockPubkey, false, false), // 时钟账户
	}

	// 如果有仲裁人，添加仲裁人账户
	if moderator != nil {
		accounts = append(accounts, solana.NewAccountMeta(*moderator, false, false))
	}

	data := new(bytes.Buffer)
	if err := bin.NewBorshEncoder(data).Encode(InitializeInstructionData{
		Instruction:     InstructionInitialize,
		UnlockHours:     unlockHours,
		RequiredSigners: requiredSigners,
		HasModerator:    moderator != nil,
		TokenType:       tokenType,
	}); err != nil {
		panic(err)
	}

	return solana.NewInstruction(programID, accounts, data.Bytes())
}

// 创建存款指令
func NewDepositInstruction(
	programID solana.PublicKey,
	depositor solana.PublicKey,
	escrowAccount solana.PublicKey,
	amount uint64,
) solana.Instruction {
	accounts := []*solana.AccountMeta{
		solana.NewAccountMeta(depositor, true, true),                // 存款人账户
		solana.NewAccountMeta(escrowAccount, false, true),           // 托管账户
		solana.NewAccountMeta(solana.SystemProgramID, false, false), // 系统程序
	}

	data := new(bytes.Buffer)
	if err := bin.NewBorshEncoder(data).Encode(DepositInstructionData{
		Instruction: InstructionDeposit,
		Amount:      amount,
	}); err != nil {
		panic(err)
	}

	return solana.NewInstruction(programID, accounts, data.Bytes())
}

// 创建签名指令
func NewSignInstruction(
	programID solana.PublicKey,
	signer solana.PublicKey,
	escrowAccount solana.PublicKey,
) solana.Instruction {
	accounts := []*solana.AccountMeta{
		solana.NewAccountMeta(signer, true, false),        // 签名者账户
		solana.NewAccountMeta(escrowAccount, false, true), // 托管账户
	}

	data := new(bytes.Buffer)
	data.WriteByte(byte(InstructionSign))

	return solana.NewInstruction(programID, accounts, data.Bytes())
}

// 创建释放指令
func NewReleaseInstruction(
	programID solana.PublicKey,
	initiator solana.PublicKey,
	escrowAccount solana.PublicKey,
	paymentTargets []PaymentTarget,
) solana.Instruction {
	accounts := []*solana.AccountMeta{
		solana.NewAccountMeta(initiator, true, false),                 // 发起者账户
		solana.NewAccountMeta(escrowAccount, false, true),             // 托管账户
		solana.NewAccountMeta(solana.SysVarClockPubkey, false, false), // 时钟账户
	}

	// 添加所有收款账户
	for _, target := range paymentTargets {
		accounts = append(accounts, solana.NewAccountMeta(target.Recipient, false, true))
	}

	data := new(bytes.Buffer)
	if err := bin.NewBorshEncoder(data).Encode(ReleaseInstructionData{
		Instruction:    InstructionRelease,
		PaymentTargets: paymentTargets,
	}); err != nil {
		panic(err)
	}

	return solana.NewInstruction(programID, accounts, data.Bytes())
}
