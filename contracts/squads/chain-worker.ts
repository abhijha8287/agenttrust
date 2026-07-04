/**
 * blockchain-service's actual on-chain client. Squads v4 only has a
 * maintained TypeScript SDK (@sqds/multisig) — no equivalent Rust crate —
 * so rather than hand-rolling borsh-encoded Squads instructions in Rust
 * (fragile, no upstream support), blockchain-service (Rust) shells out to
 * this script per call and reads back a JSON result. Rust still owns the
 * HTTP API, the database, and orchestration; this is purely the chain leg.
 *
 * Usage:
 *   ts-node chain-worker.ts register '{"agentUuid":"...","did":"...","version":"..."}'
 *   ts-node chain-worker.ts anchor '{"agentPda":"...","auditHash":"<64-hex-chars>","trustScore":42}'
 */
import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import * as multisig from "@sqds/multisig";
import { Connection, Keypair, PublicKey, TransactionMessage } from "@solana/web3.js";
import { createHash } from "crypto";
import { readFileSync } from "fs";
import path from "path";

const PROGRAM_ID = new PublicKey("DQZdU6jeY2SF1bYXNv9NuEW9JK26ZEaRfNjWG4MoqcSX");
const RPC_URL = process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";

function loadKeypair(p: string): Keypair {
  const secret = JSON.parse(readFileSync(p, "utf-8"));
  return Keypair.fromSecretKey(new Uint8Array(secret));
}

// Deterministic, not random — the same identity-service UUID always maps to
// the same on-chain agent_id, so re-running register is idempotent-checkable
// (caller decides whether to skip based on its own "already registered?"
// record) without needing any additional on-chain lookup table.
function agentIdFromUuid(uuid: string): number[] {
  return Array.from(createHash("sha256").update(uuid).digest().subarray(0, 16));
}

async function main() {
  const [, , action, argJson] = process.argv;
  if (!action || !argJson) {
    throw new Error("usage: chain-worker.ts <register|anchor> '<json-args>'");
  }
  const args = JSON.parse(argJson);

  const connection = new Connection(RPC_URL, "confirmed");
  const deployer = loadKeypair(path.join(__dirname, "..", ".solana-config", "id.json"));
  const wallet = new Wallet(deployer);
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });

  const idl = JSON.parse(
    readFileSync(path.join(__dirname, "..", "target", "idl", "agent_trust.json"), "utf-8")
  );
  const program = new Program(idl, provider);

  const multisigConfig = JSON.parse(
    readFileSync(path.join(__dirname, "multisig-config.json"), "utf-8")
  );
  const vaultPda = new PublicKey(multisigConfig.vaultPda);
  const multisigPda = new PublicKey(multisigConfig.multisigPda);
  const blockchainServiceKey = Keypair.fromSecretKey(
    new Uint8Array(multisigConfig.blockchainServiceKey)
  );
  const chainCosignerKey = Keypair.fromSecretKey(
    new Uint8Array(multisigConfig.chainCosignerKey)
  );

  if (action === "register") {
    const { agentUuid, did, version } = args as {
      agentUuid: string;
      did: string;
      version: string;
    };
    const agentId = agentIdFromUuid(agentUuid);
    const [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), deployer.publicKey.toBuffer(), Buffer.from(agentId)],
      PROGRAM_ID
    );

    const sig = await program.methods
      .registerAgent(agentId, did, version, vaultPda)
      .accounts({
        agent: agentPda,
        owner: deployer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log(JSON.stringify({ agentPda: agentPda.toBase58(), txSignature: sig }));
    return;
  }

  if (action === "anchor") {
    const { agentPda: agentPdaStr, auditHash, trustScore } = args as {
      agentPda: string;
      auditHash: string;
      trustScore: number;
    };
    const agentPda = new PublicKey(agentPdaStr);
    const hashBytes = Buffer.from(auditHash, "hex");
    if (hashBytes.length !== 32) {
      throw new Error(`auditHash must decode to 32 bytes, got ${hashBytes.length}`);
    }

    // Both writes go in one vault transaction — one round of approvals
    // covers the score update and the hash anchor atomically, rather than
    // two separate proposals racing each other.
    const updateScoreIx = await program.methods
      .updateTrustScore(trustScore)
      .accounts({ agent: agentPda, authority: vaultPda })
      .instruction();
    const recordHashIx = await program.methods
      .recordAuditHash(Array.from(hashBytes))
      .accounts({ agent: agentPda, authority: vaultPda })
      .instruction();

    const multisigInfo = await multisig.accounts.Multisig.fromAccountAddress(
      connection,
      multisigPda
    );
    const transactionIndex = BigInt(Number(multisigInfo.transactionIndex) + 1);

    const message = new TransactionMessage({
      payerKey: vaultPda,
      recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
      instructions: [updateScoreIx, recordHashIx],
    });

    // Each step depends on on-chain state the previous step just wrote.
    // Against a local validator (near-zero latency, instant finality) firing
    // these back-to-back works; against real devnet there's genuine network
    // propagation delay, so each signature is confirmed before the next
    // step reads the state it produced.
    const confirm = async (sig: string) => {
      const block = await connection.getLatestBlockhash("confirmed");
      await connection.confirmTransaction({ signature: sig, ...block }, "confirmed");
    };

    const createSig = await multisig.rpc.vaultTransactionCreate({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      creator: blockchainServiceKey.publicKey,
      vaultIndex: 0,
      ephemeralSigners: 0,
      transactionMessage: message,
      memo: "audit anchor: update_trust_score + record_audit_hash",
    });
    await confirm(createSig);

    const proposalSig = await multisig.rpc.proposalCreate({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      creator: blockchainServiceKey,
    });
    await confirm(proposalSig);

    // Both hot signers approve automatically — no human in the loop for a
    // routine audit-driven write, per the design's security model.
    const approveSig1 = await multisig.rpc.proposalApprove({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      member: blockchainServiceKey,
    });
    await confirm(approveSig1);

    const approveSig2 = await multisig.rpc.proposalApprove({
      connection,
      feePayer: chainCosignerKey,
      multisigPda,
      transactionIndex,
      member: chainCosignerKey,
    });
    await confirm(approveSig2);

    const sig = await multisig.rpc.vaultTransactionExecute({
      connection,
      feePayer: blockchainServiceKey,
      multisigPda,
      transactionIndex,
      member: blockchainServiceKey.publicKey,
      signers: [blockchainServiceKey],
      sendOptions: { skipPreflight: true },
    });
    await connection.confirmTransaction(sig, "confirmed");

    console.log(JSON.stringify({ txSignature: sig }));
    return;
  }

  throw new Error(`unknown action: ${action}`);
}

main().catch((err) => {
  console.error(JSON.stringify({ error: err instanceof Error ? err.message : String(err) }));
  process.exit(1);
});
