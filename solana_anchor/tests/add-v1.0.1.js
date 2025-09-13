import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("add-v1.0.1", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ContractManager;
  const provider = anchor.getProvider();

  it("Add v1.0.1 version and mark as recommended", async () => {
    const [contractManagerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contract_manager")],
      program.programId
    );

    const escrowProgramId = new anchor.web3.PublicKey("25ecY9sGUkFyy78aYaSbdWGMgySSKZvPjQunf6Uk23qk");

    // Add v1.0.1 version
    const addTx = await program.methods
      .addVersion(
        "escrow_program",
        "v1.0.1",
        { production: {} },
        escrowProgramId
      )
      .accounts({
        contractManager: contractManagerPDA,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Add v1.0.1 transaction signature", addTx);

    // Mark v1.0.1 as recommended
    const markTx = await program.methods
      .markRecommended("escrow_program", "v1.0.1")
      .accounts({
        contractManager: contractManagerPDA,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Mark v1.0.1 as recommended transaction signature", markTx);

    // Verify the version was added and marked as recommended
    const contractManager = await program.account.contractManager.fetch(contractManagerPDA);
    expect(contractManager.contracts[0].versions.length).to.equal(3);
    expect(contractManager.contracts[0].recommendedVersion).to.equal("v1.0.1");
    
    // Find the v1.0.1 version
    const v101Version = contractManager.contracts[0].versions.find(v => v.versionName === "v1.0.1");
    expect(v101Version).to.not.be.undefined;
    expect(v101Version.versionName).to.equal("v1.0.1");
    expect(v101Version.status).to.deep.equal({ production: {} });
    expect(v101Version.programId.toString()).to.equal(escrowProgramId.toString());

    console.log("✅ Successfully added v1.0.1 and marked as recommended");
  });
});
