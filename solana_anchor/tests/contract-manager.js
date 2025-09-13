import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("contract-manager", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ContractManager;
  const provider = anchor.getProvider();

  it("Initialize contract manager", async () => {
    const [contractManagerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contract_manager")],
      program.programId
    );

    const tx = await program.methods
      .initialize()
      .accounts({
        contractManager: contractManagerPDA,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize transaction signature", tx);

    // Verify the contract manager was initialized
    const contractManager = await program.account.contractManager.fetch(contractManagerPDA);
    expect(contractManager.authority.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(contractManager.contracts.length).to.equal(0);
  });

  it("Add version to contract manager", async () => {
    const [contractManagerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contract_manager")],
      program.programId
    );

    const escrowProgramId = new anchor.web3.PublicKey("25ecY9sGUkFyy78aYaSbdWGMgySSKZvPjQunf6Uk23qk");

    const tx = await program.methods
      .addVersion(
        "escrow_program",
        "v1.0",
        { production: {} },
        escrowProgramId
      )
      .accounts({
        contractManager: contractManagerPDA,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Add version transaction signature", tx);

    // Verify the version was added
    const contractManager = await program.account.contractManager.fetch(contractManagerPDA);
    expect(contractManager.contracts.length).to.equal(1);
    expect(contractManager.contracts[0].contractName).to.equal("escrow_program");
    expect(contractManager.contracts[0].versions.length).to.equal(1);
    expect(contractManager.contracts[0].versions[0].versionName).to.equal("v1.0");
    expect(contractManager.contracts[0].versions[0].programId.toString()).to.equal(escrowProgramId.toString());
  });

  it("Add another version", async () => {
    const [contractManagerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contract_manager")],
      program.programId
    );

    const escrowProgramIdV2 = new anchor.web3.PublicKey("25ecY9sGUkFyy78aYaSbdWGMgySSKZvPjQunf6Uk23qk");

    const tx = await program.methods
      .addVersion(
        "escrow_program",
        "v2.0",
        { beta: {} },
        escrowProgramIdV2
      )
      .accounts({
        contractManager: contractManagerPDA,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Add version v2.0 transaction signature", tx);

    // Verify the version was added
    const contractManager = await program.account.contractManager.fetch(contractManagerPDA);
    expect(contractManager.contracts.length).to.equal(1);
    expect(contractManager.contracts[0].versions.length).to.equal(2);
    expect(contractManager.contracts[0].versions[1].versionName).to.equal("v2.0");
  });

  it("Mark recommended version", async () => {
    const [contractManagerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contract_manager")],
      program.programId
    );

    const tx = await program.methods
      .markRecommended("escrow_program", "v1.0")
      .accounts({
        contractManager: contractManagerPDA,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Mark recommended transaction signature", tx);

    // Verify the recommended version was set
    const contractManager = await program.account.contractManager.fetch(contractManagerPDA);
    expect(contractManager.contracts[0].recommendedVersion).to.equal("v1.0");
  });

  it("Update version status", async () => {
    const [contractManagerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contract_manager")],
      program.programId
    );

    const tx = await program.methods
      .updateVersion(
        "escrow_program",
        "v2.0",
        { production: {} },
        { low: {} }
      )
      .accounts({
        contractManager: contractManagerPDA,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Update version transaction signature", tx);

    // Verify the version was updated
    const contractManager = await program.account.contractManager.fetch(contractManagerPDA);
    const v2Version = contractManager.contracts[0].versions.find(v => v.versionName === "v2.0");
    expect(v2Version.status).to.deep.equal({ production: {} });
    expect(v2Version.bugLevel).to.deep.equal({ low: {} });
  });

  it("Remove recommended version", async () => {
    const [contractManagerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contract_manager")],
      program.programId
    );

    const tx = await program.methods
      .removeRecommended("escrow_program")
      .accounts({
        contractManager: contractManagerPDA,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Remove recommended transaction signature", tx);

    // Verify the recommended version was removed
    const contractManager = await program.account.contractManager.fetch(contractManagerPDA);
    expect(contractManager.contracts[0].recommendedVersion).to.be.null;
  });
});
