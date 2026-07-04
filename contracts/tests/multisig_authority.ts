import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as multisig from "@sqds/multisig";
import {
  PublicKey,
  Keypair,
  TransactionMessage,
} from "@solana/web3.js";
import { expect } from "chai";
import { randomBytes } from "crypto";

import { AgentTrust } from "../target/types/agent_trust";
import { setupMultisig, fundKeypair } from "../squads/setup-multisig";

// Second half of the split from agent_trust.ts: proves the actual security
// model from the eng review — a trust-score write authorized by a Squads
// 2-of-3 vault, with both hot signers (blockchain-service + chain-cosigner)
// approving automatically, no human in the loop. If agent_trust.ts passes
// but this doesn't, the bug is in the Squads wiring, not our own program.
describe("agent_trust: Squads multisig authority", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.AgentTrust as Program<AgentTrust>;
  const connection = provider.connection;

  const owner = provider.wallet as anchor.Wallet;
  let blockchainServiceKey: Keypair;
  let chainCosignerKey: Keypair;
  let multisigPda: PublicKey;
  let vaultPda: PublicKey;

  let agentId: number[];
  let agentPda: PublicKey;

  before(async function () {
    this.timeout(60_000);

    const payer = Keypair.fromSecretKey(
      (provider.wallet as anchor.Wallet).payer.secretKey
    );
    const config = await setupMultisig(connection, payer);

    multisigPda = new PublicKey(config.multisigPda);
    vaultPda = new PublicKey(config.vaultPda);
    blockchainServiceKey = Keypair.fromSecretKey(
      new Uint8Array(config.blockchainServiceKey)
    );
    chainCosignerKey = Keypair.fromSecretKey(
      new Uint8Array(config.chainCosignerKey)
    );

    await fundKeypair(connection, payer, blockchainServiceKey);
    await fundKeypair(connection, payer, chainCosignerKey);

    agentId = Array.from(randomBytes(16));
    [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), owner.publicKey.toBuffer(), Buffer.from(agentId)],
      program.programId
    );

    // Register the agent with the Squads vault PDA as its authority — not
    // a bare keypair. This is the actual design being proven: the program
    // doesn't know or care that its authority is a multisig vault, it just
    // stores a pubkey and checks a signer against it later.
    await program.methods
      .registerAgent(agentId, "did:agenttrust:multisig-demo", "1.0.0", vaultPda)
      .accounts({
        agent: agentPda,
        owner: owner.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  it("updates the trust score via a 2-of-3 Squads vault transaction", async function () {
    this.timeout(60_000);

    const newScore = 8734;
    const updateIx = await program.methods
      .updateTrustScore(newScore)
      .accounts({ agent: agentPda, authority: vaultPda })
      .instruction();

    const transferMessage = new TransactionMessage({
      payerKey: vaultPda,
      recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
      instructions: [updateIx],
    });

    const multisigInfo = await multisig.accounts.Multisig.fromAccountAddress(
      connection,
      multisigPda
    );
    const transactionIndex = BigInt(Number(multisigInfo.transactionIndex) + 1);

    await multisig.rpc.vaultTransactionCreate({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      creator: blockchainServiceKey.publicKey,
      vaultIndex: 0,
      ephemeralSigners: 0,
      transactionMessage: transferMessage,
      memo: "update_trust_score: routine automated write",
    });

    await multisig.rpc.proposalCreate({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      creator: blockchainServiceKey,
    });

    // Both hot signers approve automatically — this is the whole point of
    // the 2-of-3 design: routine writes never wait on a human.
    await multisig.rpc.proposalApprove({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      member: blockchainServiceKey,
    });
    await multisig.rpc.proposalApprove({
      connection,
      feePayer: chainCosignerKey,
      multisigPda,
      transactionIndex,
      member: chainCosignerKey,
    });

    const executeSig = await multisig.rpc.vaultTransactionExecute({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      member: blockchainServiceKey.publicKey,
      signers: [blockchainServiceKey],
      sendOptions: { skipPreflight: true },
    });
    await connection.confirmTransaction(executeSig, "confirmed");

    const account = await program.account.agentAccount.fetch(agentPda);
    expect(account.trustScore).to.equal(newScore);
  });

  it("rejects a proposal with only 1 of 3 approvals", async function () {
    this.timeout(60_000);

    const updateIx = await program.methods
      .updateTrustScore(1)
      .accounts({ agent: agentPda, authority: vaultPda })
      .instruction();

    const transferMessage = new TransactionMessage({
      payerKey: vaultPda,
      recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
      instructions: [updateIx],
    });

    const multisigInfo = await multisig.accounts.Multisig.fromAccountAddress(
      connection,
      multisigPda
    );
    const transactionIndex = BigInt(Number(multisigInfo.transactionIndex) + 1);

    await multisig.rpc.vaultTransactionCreate({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      creator: blockchainServiceKey.publicKey,
      vaultIndex: 0,
      ephemeralSigners: 0,
      transactionMessage: transferMessage,
      memo: "single-approval attempt — must not execute",
    });
    await multisig.rpc.proposalCreate({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      creator: blockchainServiceKey,
    });
    await multisig.rpc.proposalApprove({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      member: blockchainServiceKey,
    });

    let threw = false;
    try {
      await multisig.rpc.vaultTransactionExecute({
        connection,
        feePayer: blockchainServiceKey,
        multisigPda,
        transactionIndex,
        member: blockchainServiceKey.publicKey,
        signers: [blockchainServiceKey],
        sendOptions: { skipPreflight: true },
      });
    } catch (err) {
      threw = true;
    }
    expect(threw, "expected execution to fail with only 1 of 2 required approvals")
      .to.be.true;

    // Confirm the score genuinely didn't change — the rejection isn't just
    // an RPC error, the on-chain state actually held.
    const account = await program.account.agentAccount.fetch(agentPda);
    expect(account.trustScore).to.equal(8734);
  });
});
