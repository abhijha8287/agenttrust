import { Connection, PublicKey } from "@solana/web3.js";
import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { readFileSync } from "fs";
import path from "path";

async function main() {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const secret = JSON.parse(
    readFileSync(path.join(__dirname, "..", ".solana-config", "id.json"), "utf-8")
  );
  const wallet = new Wallet(Keypair.fromSecretKey(new Uint8Array(secret)));
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
  const idl = JSON.parse(
    readFileSync(path.join(__dirname, "..", "target", "idl", "agent_trust.json"), "utf-8")
  );
  const program = new Program(idl, provider);

  const agentPda = new PublicKey(process.argv[2]);
  const account = await (program.account as any).agentAccount.fetch(agentPda);
  console.log(JSON.stringify({
    trustScore: account.trustScore,
    auditHash: Buffer.from(account.auditHash).toString("hex"),
    authority: account.authority.toBase58(),
    did: account.did,
  }, null, 2));
}
main().catch((e) => console.error(e));
