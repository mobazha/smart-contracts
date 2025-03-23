package test

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"
	"github.com/mobazha/solana2/client/escrow"
)

var (
	client      *escrow.EscrowClient
	programID   solana.PublicKey
	buyer       *solana.Wallet
	seller      *solana.Wallet
	moderator   *solana.Wallet
	testContext context.Context
)

func TestMain(m *testing.M) {
	setup()
	code := m.Run()
	teardown()
	os.Exit(code)
}

func setup() {
	testContext = context.Background()

	// 读取程序ID
	programIDStr, err := os.ReadFile("../../target/deploy/program-id.txt")
	if err != nil {
		panic(err)
	}
	programID = solana.MustPublicKeyFromBase58(string(programIDStr))

	// 初始化客户端
	client, err = escrow.NewEscrowClient("https://api.devnet.solana.com", programID)
	if err != nil {
		panic(err)
	}

	// 创建测试钱包
	buyer = solana.NewWallet()
	seller = solana.NewWallet()
	moderator = solana.NewWallet()

	// 为测试钱包充值
	requestAirdrop(buyer.PublicKey())
	requestAirdrop(seller.PublicKey())
	requestAirdrop(moderator.PublicKey())

	// 等待资金到账
	time.Sleep(time.Second * 2)
}

func teardown() {
	if client.WsClient != nil {
		client.WsClient.Close()
	}
}

func requestAirdrop(pubkey solana.PublicKey) {
	sig, err := client.RpcClient.RequestAirdrop(
		context.Background(),
		pubkey,
		solana.LAMPORTS_PER_SOL*2,
		rpc.CommitmentFinalized,
	)
	if err != nil {
		panic(err)
	}
	client.WaitForConfirmation(sig.String())
}
