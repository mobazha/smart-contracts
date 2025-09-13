package main

import (
	"context"
	"fmt"
	"log"

	"github.com/gagliardetto/solana-go"
)

func main() {
	// Initialize the contract manager client
	// Replace with your actual RPC endpoint and program ID
	rpcEndpoint := "https://api.devnet.solana.com"
	programID := solana.MustPublicKeyFromBase58("6LmWMjAMAfVdc8mpgPjHvFLa2sbcudiLiJT3bAGRYMMD")

	client := NewContractManagerClient(rpcEndpoint, programID)

	ctx := context.Background()

	// Example 1: Get recommended version for escrow program
	fmt.Println("=== Getting Recommended Version for Escrow Program ===")
	recommendedVersion, err := client.GetRecommendedVersion(ctx, "escrow_program")
	if err != nil {
		log.Printf("Error getting recommended version: %v", err)
	} else {
		fmt.Printf("Recommended Version: %s\n", recommendedVersion.VersionName)
		fmt.Printf("Status: %s\n", recommendedVersion.Status.String())
		fmt.Printf("Bug Level: %s\n", recommendedVersion.BugLevel.String())
		fmt.Printf("Program ID: %s\n", recommendedVersion.ProgramID.String())
		fmt.Printf("Date Added: %d\n", recommendedVersion.DateAdded)
	}

	// Example 2: Get all versions for escrow program
	fmt.Println("\n=== Getting All Versions for Escrow Program ===")
	versions, err := client.GetContractVersions(ctx, "escrow_program")
	if err != nil {
		log.Printf("Error getting contract versions: %v", err)
	} else {
		fmt.Printf("Found %d versions:\n", len(versions))
		for i, version := range versions {
			fmt.Printf("  %d. Version: %s\n", i+1, version.VersionName)
			fmt.Printf("     Status: %s\n", version.Status.String())
			fmt.Printf("     Bug Level: %s\n", version.BugLevel.String())
			fmt.Printf("     Program ID: %s\n", version.ProgramID.String())
			fmt.Printf("     Date Added: %d\n", version.DateAdded)
		}
	}

	// Example 3: Get specific program ID
	fmt.Println("\n=== Getting Specific Program ID ===")
	escrowProgramID, err := client.GetProgramID(ctx, "escrow_program", "v1.0")
	if err != nil {
		log.Printf("Error getting program ID: %v", err)
	} else {
		fmt.Printf("Program ID for escrow_program v1.0: %s\n", escrowProgramID.String())
	}

	// Example 4: Get contract manager state
	fmt.Println("\n=== Getting Contract Manager State ===")
	manager, err := client.GetContractManager(ctx)
	if err != nil {
		log.Printf("Error getting contract manager: %v", err)
	} else {
		fmt.Printf("Authority: %s\n", manager.Authority.String())
		fmt.Printf("Bump: %d\n", manager.Bump)
		fmt.Printf("Total Contracts: %d\n", len(manager.Contracts))

		for i, contract := range manager.Contracts {
			fmt.Printf("  Contract %d: %s\n", i+1, contract.ContractName)
			fmt.Printf("    Versions: %d\n", len(contract.Versions))
			if contract.RecommendedVersion != nil {
				fmt.Printf("    Recommended: %s\n", *contract.RecommendedVersion)
			} else {
				fmt.Printf("    Recommended: None\n")
			}
		}
	}
}
