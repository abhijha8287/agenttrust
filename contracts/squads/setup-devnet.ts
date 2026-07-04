/**
 * Runs setupMultisig() against real Solana devnet (not the local test
 * validator) and funds the two hot-signer keys via a direct transfer from
 * the deployer wallet rather than requestAirdrop — the devnet faucet is
 * already rate-limited per IP, and the deployer wallet already has SOL from
 * the program deployment, so a transfer sidesteps that entirely.
 */
import { Connection, Keypair, LAMPORTS_PER_SOL, SystemProgram, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { readFileSync } from "fs";
import path from "path";
import { setupMultisig } from "./setup-multisig";

async function transferSol(connection: Connection, payer: Keypair, to: import("@solana/web3.js").PublicKey, lamports: number) {
  const tx = new Transaction().add(
    SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: to, lamports })
  );
  const sig = await sendAndConfirmTransaction(connection, tx, [payer], { commitment: "confirmed" });
  return sig;
}

async function main() {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");

  const keypairPath = path.join(process.env.HOME || "/root", ".config/solana/id.json");
  const secret = JSON.parse(readFileSync(keypairPath, "utf-8"));
  const payer = Keypair.fromSecretKey(new Uint8Array(secret));

  console.log("Deployer/payer:", payer.publicKey.toBase58());
  const balance = await connection.getBalance(payer.publicKey);
  console.log("Deployer balance:", balance / LAMPORTS_PER_SOL, "SOL");

  const config = await setupMultisig(connection, payer);
  console.log("Multisig created:");
  console.log("  multisigPda:", config.multisigPda);
  console.log("  vaultPda:", config.vaultPda);

  const blockchainServiceKey = Keypair.fromSecretKey(new Uint8Array(config.blockchainServiceKey));
  const chainCosignerKey = Keypair.fromSecretKey(new Uint8Array(config.chainCosignerKey));

  const fundAmount = 0.1 * LAMPORTS_PER_SOL;
  const sig1 = await transferSol(connection, payer, blockchainServiceKey.publicKey, fundAmount);
  console.log("Funded blockchainServiceKey:", blockchainServiceKey.publicKey.toBase58(), sig1);
  const sig2 = await transferSol(connection, payer, chainCosignerKey.publicKey, fundAmount);
  console.log("Funded chainCosignerKey:", chainCosignerKey.publicKey.toBase58(), sig2);

  console.log("\nDone. Config written to squads/multisig-config.json");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
