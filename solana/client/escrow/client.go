package escrow

import (
	"context"
	"encoding/binary"
	"fmt"
	"time"

	"github.com/gagliardetto/binary"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/programs/system"
	"github.com/gagliardetto/solana-go/rpc"
	"github.com/gagliardetto/solana-go/rpc/ws"
)

type EscrowClient struct {
	ProgramID solana.PublicKey
	RpcClient *rpc.Client
	WsClient  *ws.Client
}

type CreateEscrowParams struct {
	Buyer             solana.PublicKey
	Seller            solana.PublicKey
	Moderator         *solana.PublicKey
	UnlockHours       uint64 // 0表示不设置时间锁
	RequiredSignature uint8
	TokenType         TokenType
}

// 托管账户数据结构
type Escrow struct {
	State           EscrowState
	Buyer           solana.PublicKey
	Seller          solana.PublicKey
	Moderator       *solana.PublicKey
	TokenType       TokenType
	Amount          uint64
	UnlockTime      uint64
	RequiredSigners uint8
	Signatures      []solana.PublicKey
}

func NewEscrowClient(endpoint string, programID solana.PublicKey) (*EscrowClient, error) {
	rpcClient := rpc.New(endpoint)
	wsClient, err := ws.Connect(context.Background(), endpoint)
	if err != nil {
		return nil, fmt.Errorf("连接WebSocket失败: %v", err)
	}

	return &EscrowClient{
		ProgramID: programID,
		RpcClient: rpcClient,
		WsClient:  wsClient,
	}, nil
}

// 获取托管账户地址
func (ec *EscrowClient) GetEscrowAddress(params CreateEscrowParams) (solana.PublicKey, uint8, error) {
	seed := make([]byte, 8)
	binary.LittleEndian.PutUint64(seed, uint64(time.Now().Unix()))

	seeds := [][]byte{
		[]byte("escrow"),
		params.Buyer.Bytes(),
		params.Seller.Bytes(),
		seed,
	}

	if params.Moderator != nil {
		seeds = append(seeds, params.Moderator.Bytes())
	}

	addr, bump, err := solana.FindProgramAddress(seeds, ec.ProgramID)
	if err != nil {
		return solana.PublicKey{}, 0, fmt.Errorf("计算PDA失败: %v", err)
	}

	return addr, bump, nil
}

// 计算所需费用
func (ec *EscrowClient) GetTotalCost(amount uint64) (uint64, error) {
	// 获取租金
	rent, err := ec.RpcClient.GetMinimumBalanceForRentExemption(
		context.Background(),
		ACCOUNT_LEN, // 使用同步的常量
		rpc.CommitmentFinalized,
	)
	if err != nil {
		return 0, fmt.Errorf("获取租金失败: %v", err)
	}

	// 获取最近的区块哈希以计算交易费
	recent, err := ec.RpcClient.GetRecentBlockhash(context.Background(), rpc.CommitmentFinalized)
	if err != nil {
		return 0, fmt.Errorf("获取最近区块哈希失败: %v", err)
	}

	// 计算总费用
	fee := recent.Value.FeeCalculator.LamportsPerSignature * 3 // 3条指令
	total := rent + amount + fee

	return total, nil
}

// GetEscrowState 获取托管状态
func (ec *EscrowClient) GetEscrowState(addr solana.PublicKey) (*Escrow, error) {
	account, err := ec.RpcClient.GetAccountInfo(context.Background(), addr)
	if err != nil {
		return nil, fmt.Errorf("获取账户信息失败: %v", err)
	}

	escrow := &Escrow{}
	if err := bin.NewBorshDecoder(account.GetBinary()).Decode(escrow); err != nil {
		return nil, fmt.Errorf("解码账户数据失败: %v", err)
	}
	return escrow, nil
}

// CreateAndDeposit 创建托管并存款
func (ec *EscrowClient) CreateAndDeposit(
	params CreateEscrowParams,
	amount uint64,
	payer *solana.PrivateKey,
) (string, error) {
	// 1. 获取托管账户地址
	escrowAddr, _, err := ec.GetEscrowAddress(params)
	if err != nil {
		return "", err
	}

	// 2. 创建交易指令
	recent, err := ec.RpcClient.GetRecentBlockhash(context.Background(), rpc.CommitmentFinalized)
	if err != nil {
		return "", fmt.Errorf("获取区块哈希失败: %v", err)
	}

	rent, err := ec.RpcClient.GetMinimumBalanceForRentExemption(
		context.Background(),
		ACCOUNT_LEN, // 使用同步的常量
		rpc.CommitmentFinalized,
	)
	if err != nil {
		return "", fmt.Errorf("获取租金失败: %v", err)
	}

	tx, err := solana.NewTransaction(
		[]solana.Instruction{
			// 创建账户指令
			system.NewCreateAccountInstruction(
				rent,
				ACCOUNT_LEN,
				ec.ProgramID,
				payer.PublicKey(),
				escrowAddr,
			).Build(),
			NewInitializeInstruction(
				ec.ProgramID,
				payer.PublicKey(),
				escrowAddr,
				params.Seller,
				params.Moderator,
				params.UnlockHours,
				params.RequiredSignature,
				params.TokenType,
			),
			NewDepositInstruction(
				ec.ProgramID,
				payer.PublicKey(),
				escrowAddr,
				amount,
			),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(payer.PublicKey()),
	)

	// 3. 签名并发送交易
	_, err = tx.Sign(func(key solana.PublicKey) *solana.PrivateKey {
		if key.Equals(payer.PublicKey()) {
			return payer
		}
		return nil
	})
	if err != nil {
		return "", fmt.Errorf("签名失败: %v", err)
	}

	sig, err := ec.RpcClient.SendTransaction(context.Background(), tx)
	if err != nil {
		return "", fmt.Errorf("发送交易失败: %v", err)
	}

	return sig.String(), nil
}

// Sign 签名
func (ec *EscrowClient) Sign(
	escrowAddr solana.PublicKey,
	signer *solana.PrivateKey,
) (string, error) {
	recent, err := ec.RpcClient.GetRecentBlockhash(
		context.Background(),
		rpc.CommitmentFinalized,
	)
	if err != nil {
		return "", fmt.Errorf("获取区块哈希失败: %v", err)
	}

	tx, err := solana.NewTransaction(
		[]solana.Instruction{
			NewSignInstruction(
				ec.ProgramID,
				signer.PublicKey(),
				escrowAddr,
			),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(signer.PublicKey()),
	)

	_, err = tx.Sign(func(key solana.PublicKey) *solana.PrivateKey {
		if key.Equals(signer.PublicKey()) {
			return signer
		}
		return nil
	})
	if err != nil {
		return "", fmt.Errorf("签名失败: %v", err)
	}

	sig, err := ec.RpcClient.SendTransaction(context.Background(), tx)
	if err != nil {
		return "", fmt.Errorf("发送交易失败: %v", err)
	}

	return sig.String(), nil
}

// Release 释放资金
func (ec *EscrowClient) Release(
	escrowAddr solana.PublicKey,
	initiator *solana.PrivateKey,
	paymentTargets []PaymentTarget,
) (string, error) {
	recent, err := ec.RpcClient.GetRecentBlockhash(
		context.Background(),
		rpc.CommitmentFinalized,
	)
	if err != nil {
		return "", fmt.Errorf("获取区块哈希失败: %v", err)
	}

	tx, err := solana.NewTransaction(
		[]solana.Instruction{
			NewReleaseInstruction(
				ec.ProgramID,
				initiator.PublicKey(),
				escrowAddr,
				paymentTargets,
			),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(initiator.PublicKey()),
	)

	_, err = tx.Sign(func(key solana.PublicKey) *solana.PrivateKey {
		if key.Equals(initiator.PublicKey()) {
			return initiator
		}
		return nil
	})
	if err != nil {
		return "", fmt.Errorf("签名失败: %v", err)
	}

	sig, err := ec.RpcClient.SendTransaction(context.Background(), tx)
	if err != nil {
		return "", fmt.Errorf("发送交易失败: %v", err)
	}

	return sig.String(), nil
}

// 释放SPL代币
func (ec *EscrowClient) ReleaseSpl(
	escrowAddr solana.PublicKey,
	tokenAccount solana.PublicKey,
	initiator *solana.PrivateKey,
	paymentTargets []PaymentTarget,
) (string, error) {
	recent, err := ec.RpcClient.GetRecentBlockhash(context.Background(), rpc.CommitmentFinalized)
	if err != nil {
		return "", fmt.Errorf("获取区块哈希失败: %v", err)
	}

	tx, err := solana.NewTransaction(
		[]solana.Instruction{
			NewReleaseInstruction(
				ec.ProgramID,
				initiator.PublicKey(),
				escrowAddr,
				paymentTargets,
			),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(initiator.PublicKey()),
	)

	_, err = tx.Sign(func(key solana.PublicKey) *solana.PrivateKey {
		if key.Equals(initiator.PublicKey()) {
			return initiator
		}
		return nil
	})
	if err != nil {
		return "", fmt.Errorf("签名失败: %v", err)
	}

	sig, err := ec.RpcClient.SendTransaction(context.Background(), tx)
	if err != nil {
		return "", fmt.Errorf("发送交易失败: %v", err)
	}

	return sig.String(), nil
}

// 等待交易确认
func (ec *EscrowClient) WaitForConfirmation(sig string) error {
	for {
		signature := solana.MustSignatureFromBase58(sig)
		status, err := ec.RpcClient.GetSignatureStatuses(
			context.Background(),
			false,
			signature,
		)
		if err != nil {
			return err
		}
		if status.Value[0] != nil && status.Value[0].Confirmations != nil && *status.Value[0].Confirmations > 0 {
			return nil
		}
		time.Sleep(time.Second)
	}
}
