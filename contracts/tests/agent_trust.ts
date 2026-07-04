import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import { randomBytes } from "crypto";

import { AgentTrust } from "../target/types/agent_trust";

// Base program-logic proof: single-keypair authority, no multisig. This
// isolates "does OUR program work" from "does Squads' SDK behave as
// documented" (see multisig_authority.ts for the second half of that split).
describe("agent_trust: single-authority", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.AgentTrust as Program<AgentTrust>;

  const owner = provider.wallet as anchor.Wallet;
  const authority = Keypair.generate();
  const wrongAuthority = Keypair.generate();

  let agentId: number[];
  let agentPda: PublicKey;

  before(async () => {
    agentId = Array.from(randomBytes(16));
    const sig = await provider.connection.requestAirdrop(
      authority.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), owner.publicKey.toBuffer(), Buffer.from(agentId)],
      program.programId
    );
  });

  it("registers an agent with trust_score 0 and Unverified status", async () => {
    await program.methods
      .registerAgent(agentId, "did:agenttrust:test", "1.0.0", authority.publicKey)
      .accounts({
        agent: agentPda,
        owner: owner.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const account = await program.account.agentAccount.fetch(agentPda);
    expect(account.owner.toBase58()).to.equal(owner.publicKey.toBase58());
    expect(account.authority.toBase58()).to.equal(authority.publicKey.toBase58());
    expect(account.did).to.equal("did:agenttrust:test");
    expect(account.trustScore).to.equal(0);
    expect(account.verificationStatus).to.equal(0); // Unverified
    expect(Buffer.from(account.auditHash)).to.deep.equal(Buffer.alloc(32));
  });

  it("rejects a did longer than 64 bytes", async () => {
    const badAgentId = Array.from(randomBytes(16));
    const [badPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), owner.publicKey.toBuffer(), Buffer.from(badAgentId)],
      program.programId
    );
    const longDid = "did:agenttrust:" + "x".repeat(60); // > 64 bytes total

    let threw = false;
    try {
      await program.methods
        .registerAgent(badAgentId, longDid, "1.0.0", authority.publicKey)
        .accounts({
          agent: badPda,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    } catch (err) {
      threw = true;
      expect(String(err)).to.match(/DidTooLong/);
    }
    expect(threw, "expected registration to reject an oversized DID").to.be.true;
  });

  it("lets the registered authority update the trust score", async () => {
    await program.methods
      .updateTrustScore(8734)
      .accounts({ agent: agentPda, authority: authority.publicKey })
      .signers([authority])
      .rpc();

    const account = await program.account.agentAccount.fetch(agentPda);
    expect(account.trustScore).to.equal(8734);
  });

  it("rejects a trust score above 10000", async () => {
    let threw = false;
    try {
      await program.methods
        .updateTrustScore(10001)
        .accounts({ agent: agentPda, authority: authority.publicKey })
        .signers([authority])
        .rpc();
    } catch (err) {
      threw = true;
      expect(String(err)).to.match(/InvalidScore/);
    }
    expect(threw, "expected an out-of-range score to be rejected").to.be.true;
  });

  it("rejects a trust score update from a non-authority signer", async () => {
    const sig = await provider.connection.requestAirdrop(
      wrongAuthority.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    let threw = false;
    try {
      await program.methods
        .updateTrustScore(1)
        .accounts({ agent: agentPda, authority: wrongAuthority.publicKey })
        .signers([wrongAuthority])
        .rpc();
    } catch (err) {
      threw = true;
      expect(String(err)).to.match(/Unauthorized/);
    }
    expect(threw, "expected a non-authority signer to be rejected").to.be.true;
  });

  it("lets the registered authority record an audit hash", async () => {
    const hash = Array.from(randomBytes(32));
    await program.methods
      .recordAuditHash(hash)
      .accounts({ agent: agentPda, authority: authority.publicKey })
      .signers([authority])
      .rpc();

    const account = await program.account.agentAccount.fetch(agentPda);
    expect(Array.from(account.auditHash)).to.deep.equal(hash);
  });
});
