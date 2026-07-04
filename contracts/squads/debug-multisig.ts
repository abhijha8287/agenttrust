import { Connection, PublicKey } from "@solana/web3.js";
import * as multisig from "@sqds/multisig";
import { readFileSync } from "fs";
import path from "path";

async function main() {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const config = JSON.parse(
    readFileSync(path.join(__dirname, "multisig-config.json"), "utf-8")
  );
  const multisigPda = new PublicKey(config.multisigPda);
  const info = await multisig.accounts.Multisig.fromAccountAddress(connection, multisigPda);
  console.log("transactionIndex:", info.transactionIndex.toString());
  console.log("staleTransactionIndex:", info.staleTransactionIndex.toString());
  console.log("threshold:", info.threshold);
  console.log(
    "members:",
    info.members.map((m) => m.key.toBase58())
  );
}
main().catch((e) => console.error(e));
