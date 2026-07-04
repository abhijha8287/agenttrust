/**
 * Creates the 2-of-3 multisig that becomes agent_trust's authority, per the
 * eng review (2026-07-04): two automated hot signers (blockchain-service +
 * chain-cosigner) so routine trust-score writes stay fully live/automatic
 * during a demo — no human approval in the loop. The third key is cold,
 * used only for recovery/rotation if the two hot signers are ever
 * compromised together, never for routine writes.
 *
 * Run once per environment (localnet for tests, devnet for the real demo).
 * Writes the resulting keys + PDAs to squads/multisig-config.json so the
 * test suite and blockchain-service/chain-cosigner can all read the same
 * config without regenerating it.
 */
import * as multisig from "@sqds/multisig";
import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { writeFileSync } from "fs";
import path from "path";

const { Permission, Permissions } = multisig.types;

export interface MultisigConfig {
  multisigPda: string;
  vaultPda: string;
  createKey: string;
  blockchainServiceKey: number[]; // hot signer 1
  chainCosignerKey: number[]; // hot signer 2
  coldRecoveryKey: number[]; // cold signer, recovery/rotation only
}

export async function setupMultisig(
  connection: Connection,
  payer: Keypair
): Promise<MultisigConfig> {
  const createKey = Keypair.generate();
  const blockchainServiceKey = Keypair.generate();
  const chainCosignerKey = Keypair.generate();
  const coldRecoveryKey = Keypair.generate();

  const [multisigPda] = multisig.getMultisigPda({
    createKey: createKey.publicKey,
  });

  const programConfigPda = multisig.getProgramConfigPda({})[0];
  const programConfig = await multisig.accounts.ProgramConfig.fromAccountAddress(
    connection,
    programConfigPda
  );

  const signature = await multisig.rpc.multisigCreateV2({
    connection,
    createKey,
    creator: payer,
    multisigPda,
    configAuthority: null,
    timeLock: 0,
    members: [
      // Both hot signers get full permissions (Initiate + Vote + Execute) so
      // either can propose and both can approve/execute a routine write
      // with zero human involvement.
      { key: blockchainServiceKey.publicKey, permissions: Permissions.all() },
      { key: chainCosignerKey.publicKey, permissions: Permissions.all() },
      // The cold key can only vote — it participates in recovery scenarios
      // (e.g. rotating out a compromised hot signer) but is never one of
      // the two approvals a routine trust-score write needs.
      {
        key: coldRecoveryKey.publicKey,
        permissions: Permissions.fromPermissions([Permission.Vote]),
      },
    ],
    threshold: 2,
    rentCollector: null,
    treasury: programConfig.treasury,
    sendOptions: { skipPreflight: true },
  });

  const block = await connection.getLatestBlockhash("confirmed");
  const result = await connection.confirmTransaction(
    { signature, ...block },
    "confirmed"
  );
  if (result.value.err) {
    throw new Error(`multisig creation failed: ${result.value.err.toString()}`);
  }

  const [vaultPda] = multisig.getVaultPda({ multisigPda, index: 0 });

  const config: MultisigConfig = {
    multisigPda: multisigPda.toBase58(),
    vaultPda: vaultPda.toBase58(),
    createKey: createKey.publicKey.toBase58(),
    blockchainServiceKey: Array.from(blockchainServiceKey.secretKey),
    chainCosignerKey: Array.from(chainCosignerKey.secretKey),
    coldRecoveryKey: Array.from(coldRecoveryKey.secretKey),
  };

  writeFileSync(
    path.join(__dirname, "multisig-config.json"),
    JSON.stringify(config, null, 2)
  );

  return config;
}

export async function fundKeypair(
  connection: Connection,
  payer: Keypair,
  target: Keypair,
  lamports: number = LAMPORTS_PER_SOL
) {
  const sig = await connection.requestAirdrop(target.publicKey, lamports);
  await connection.confirmTransaction(sig);
}
