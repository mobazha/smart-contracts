package main

import (
	"context"
	"encoding/binary"
	"fmt"
	"log"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"
)

// ContractStatus represents the status of a contract version
type ContractStatus uint8

const (
	ContractStatusBeta ContractStatus = iota
	ContractStatusReleaseCandidate
	ContractStatusProduction
	ContractStatusDeprecated
)

// BugLevel represents the bug level of a contract version
type BugLevel uint8

const (
	BugLevelNone BugLevel = iota
	BugLevelLow
	BugLevelMedium
	BugLevelHigh
	BugLevelCritical
)

// Version represents a contract version
type Version struct {
	VersionName string
	Status      ContractStatus
	BugLevel    BugLevel
	ProgramID   solana.PublicKey
	DateAdded   int64
}

// Contract represents a contract with its versions
type Contract struct {
	ContractName       string
	Versions           []Version
	RecommendedVersion *string
}

// ContractManager represents the contract manager state
type ContractManager struct {
	Authority solana.PublicKey
	Contracts []Contract
	Bump      uint8
}

// ContractManagerClient provides methods to interact with the contract manager
type ContractManagerClient struct {
	client     *rpc.Client
	programID  solana.PublicKey
	managerPDA solana.PublicKey
}

// NewContractManagerClient creates a new contract manager client
func NewContractManagerClient(rpcEndpoint string, programID solana.PublicKey) *ContractManagerClient {
	c := rpc.New(rpcEndpoint)

	// Derive the contract manager PDA
	managerPDA, _, err := solana.FindProgramAddress([][]byte{[]byte("contract_manager")}, programID)
	if err != nil {
		log.Fatalf("Failed to derive contract manager PDA: %v", err)
	}

	return &ContractManagerClient{
		client:     c,
		programID:  programID,
		managerPDA: managerPDA,
	}
}

// GetRecommendedVersion returns the recommended version for a contract
func (cmc *ContractManagerClient) GetRecommendedVersion(ctx context.Context, contractName string) (*Version, error) {
	manager, err := cmc.GetContractManager(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get contract manager: %w", err)
	}

	for _, contract := range manager.Contracts {
		if contract.ContractName == contractName {
			if contract.RecommendedVersion == nil {
				return nil, fmt.Errorf("no recommended version set for contract %s", contractName)
			}

			for _, version := range contract.Versions {
				if version.VersionName == *contract.RecommendedVersion {
					return &version, nil
				}
			}
			return nil, fmt.Errorf("recommended version %s not found for contract %s", *contract.RecommendedVersion, contractName)
		}
	}

	return nil, fmt.Errorf("contract %s not found", contractName)
}

// GetContractVersions returns all versions for a contract
func (cmc *ContractManagerClient) GetContractVersions(ctx context.Context, contractName string) ([]Version, error) {
	manager, err := cmc.GetContractManager(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get contract manager: %w", err)
	}

	for _, contract := range manager.Contracts {
		if contract.ContractName == contractName {
			return contract.Versions, nil
		}
	}

	return nil, fmt.Errorf("contract %s not found", contractName)
}

// GetContractManager fetches the contract manager state from the blockchain
func (cmc *ContractManagerClient) GetContractManager(ctx context.Context) (*ContractManager, error) {
	accountInfo, err := cmc.client.GetAccountInfo(ctx, cmc.managerPDA)
	if err != nil {
		return nil, fmt.Errorf("failed to get account info: %w", err)
	}

	if accountInfo.Value == nil {
		return nil, fmt.Errorf("contract manager not initialized")
	}

	// Parse the account data
	manager, err := cmc.parseContractManagerData(accountInfo.Value.Data.GetBinary())
	if err != nil {
		return nil, fmt.Errorf("failed to parse contract manager data: %w", err)
	}

	return manager, nil
}

// parseContractManagerData parses the raw account data into a ContractManager struct
func (cmc *ContractManagerClient) parseContractManagerData(data []byte) (*ContractManager, error) {
	if len(data) < 8 {
		return nil, fmt.Errorf("invalid account data length")
	}

	// Skip the discriminator (first 8 bytes)
	data = data[8:]

	// Parse authority (32 bytes)
	if len(data) < 32 {
		return nil, fmt.Errorf("invalid authority data")
	}
	authority := solana.PublicKeyFromBytes(data[:32])
	data = data[32:]

	// Parse contracts vector length (4 bytes)
	if len(data) < 4 {
		return nil, fmt.Errorf("invalid contracts length data")
	}
	contractsLength := binary.LittleEndian.Uint32(data[:4])
	data = data[4:]

	// Parse contracts
	contracts := make([]Contract, 0, contractsLength)
	for i := uint32(0); i < contractsLength; i++ {
		contract, remainingData, err := cmc.parseContract(data)
		if err != nil {
			return nil, fmt.Errorf("failed to parse contract %d: %w", i, err)
		}
		contracts = append(contracts, contract)
		data = remainingData
	}

	// Parse bump (1 byte)
	if len(data) < 1 {
		return nil, fmt.Errorf("invalid bump data")
	}
	bump := data[0]

	return &ContractManager{
		Authority: authority,
		Contracts: contracts,
		Bump:      bump,
	}, nil
}

// parseContract parses a single contract from the data
func (cmc *ContractManagerClient) parseContract(data []byte) (Contract, []byte, error) {
	// Parse contract name length (4 bytes)
	if len(data) < 4 {
		return Contract{}, nil, fmt.Errorf("invalid contract name length")
	}
	nameLength := binary.LittleEndian.Uint32(data[:4])
	data = data[4:]

	// Parse contract name
	if len(data) < int(nameLength) {
		return Contract{}, nil, fmt.Errorf("invalid contract name data")
	}
	contractName := string(data[:nameLength])
	data = data[nameLength:]

	// Parse versions vector length (4 bytes)
	if len(data) < 4 {
		return Contract{}, nil, fmt.Errorf("invalid versions length")
	}
	versionsLength := binary.LittleEndian.Uint32(data[:4])
	data = data[4:]

	// Parse versions
	versions := make([]Version, 0, versionsLength)
	for i := uint32(0); i < versionsLength; i++ {
		version, remainingData, err := cmc.parseVersion(data)
		if err != nil {
			return Contract{}, nil, fmt.Errorf("failed to parse version %d: %w", i, err)
		}
		versions = append(versions, version)
		data = remainingData
	}

	// Parse recommended version (optional)
	var recommendedVersion *string
	if len(data) > 0 {
		// Check if recommended version exists (1 byte flag)
		hasRecommended := data[0] != 0
		data = data[1:]

		if hasRecommended {
			// Parse recommended version name length (4 bytes)
			if len(data) < 4 {
				return Contract{}, nil, fmt.Errorf("invalid recommended version name length")
			}
			recNameLength := binary.LittleEndian.Uint32(data[:4])
			data = data[4:]

			// Parse recommended version name
			if len(data) < int(recNameLength) {
				return Contract{}, nil, fmt.Errorf("invalid recommended version name data")
			}
			recName := string(data[:recNameLength])
			recommendedVersion = &recName
			data = data[recNameLength:]
		}
	}

	return Contract{
		ContractName:       contractName,
		Versions:           versions,
		RecommendedVersion: recommendedVersion,
	}, data, nil
}

// parseVersion parses a single version from the data
func (cmc *ContractManagerClient) parseVersion(data []byte) (Version, []byte, error) {
	// Parse version name length (4 bytes)
	if len(data) < 4 {
		return Version{}, nil, fmt.Errorf("invalid version name length")
	}
	nameLength := binary.LittleEndian.Uint32(data[:4])
	data = data[4:]

	// Parse version name
	if len(data) < int(nameLength) {
		return Version{}, nil, fmt.Errorf("invalid version name data")
	}
	versionName := string(data[:nameLength])
	data = data[nameLength:]

	// Parse status (1 byte)
	if len(data) < 1 {
		return Version{}, nil, fmt.Errorf("invalid status data")
	}
	status := ContractStatus(data[0])
	data = data[1:]

	// Parse bug level (1 byte)
	if len(data) < 1 {
		return Version{}, nil, fmt.Errorf("invalid bug level data")
	}
	bugLevel := BugLevel(data[0])
	data = data[1:]

	// Parse program ID (32 bytes)
	if len(data) < 32 {
		return Version{}, nil, fmt.Errorf("invalid program ID data")
	}
	programID := solana.PublicKeyFromBytes(data[:32])
	data = data[32:]

	// Parse date added (8 bytes)
	if len(data) < 8 {
		return Version{}, nil, fmt.Errorf("invalid date added data")
	}
	dateAdded := int64(binary.LittleEndian.Uint64(data[:8]))
	data = data[8:]

	return Version{
		VersionName: versionName,
		Status:      status,
		BugLevel:    bugLevel,
		ProgramID:   programID,
		DateAdded:   dateAdded,
	}, data, nil
}

// GetProgramID returns the program ID for a contract name and version
func (cmc *ContractManagerClient) GetProgramID(ctx context.Context, contractName, versionName string) (solana.PublicKey, error) {
	manager, err := cmc.GetContractManager(ctx)
	if err != nil {
		return solana.PublicKey{}, fmt.Errorf("failed to get contract manager: %w", err)
	}

	for _, contract := range manager.Contracts {
		if contract.ContractName == contractName {
			for _, version := range contract.Versions {
				if version.VersionName == versionName {
					return version.ProgramID, nil
				}
			}
			return solana.PublicKey{}, fmt.Errorf("version %s not found for contract %s", versionName, contractName)
		}
	}

	return solana.PublicKey{}, fmt.Errorf("contract %s not found", contractName)
}

// String methods for better debugging
func (cs ContractStatus) String() string {
	switch cs {
	case ContractStatusBeta:
		return "Beta"
	case ContractStatusReleaseCandidate:
		return "ReleaseCandidate"
	case ContractStatusProduction:
		return "Production"
	case ContractStatusDeprecated:
		return "Deprecated"
	default:
		return "Unknown"
	}
}

func (bl BugLevel) String() string {
	switch bl {
	case BugLevelNone:
		return "None"
	case BugLevelLow:
		return "Low"
	case BugLevelMedium:
		return "Medium"
	case BugLevelHigh:
		return "High"
	case BugLevelCritical:
		return "Critical"
	default:
		return "Unknown"
	}
}
