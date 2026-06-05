/**
 * onchain-rbac CLI client
 * -----------------------
 * A minimal, dependency-light CLI that drives the RBAC program. It doubles as
 * the "testable client" deliverable and as the script used to produce devnet
 * transaction links.
 *
 * Usage:
 *   ts-node app/cli.ts <command> [args]   (or: yarn cli <command>)
 *
 * Commands:
 *   init-org                              Create your organization (you = admin)
 *   create-role <id> <name> <perms>      perms: comma list (read,write,delete,admin) or number
 *   assign <userPubkey> <roleId>         Assign a user to a role
 *   set-status <userPubkey> <on|off>     Activate / deactivate a membership
 *   revoke <userPubkey>                  Revoke a membership (refunds rent)
 *   perform <roleId> <perm>              Run the guarded action as the current wallet
 *   show                                 Print the org, roles, and memberships
 *
 * Config (env, with sensible defaults):
 *   ANCHOR_PROVIDER_URL   RPC url            (default: https://api.devnet.solana.com)
 *   ANCHOR_WALLET         keypair json path  (default: ~/.config/solana/id.json)
 *   RBAC_ADMIN            admin pubkey        (default: the wallet's pubkey)
 */
import { AnchorProvider, Program, Wallet, web3, BN } from "@anchor-lang/core";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
// IDL is committed under app/idl so the CLI runs from a fresh clone without
// requiring `anchor build` first. Re-copy from target/idl after rebuilds.
import idl from "./idl/onchain_rbac.json";

const { PublicKey, Keypair, Connection } = web3;

const ORG_SEED = Buffer.from("org");
const ROLE_SEED = Buffer.from("role");
const MEMBER_SEED = Buffer.from("member");

const PERM_BITS: Record<string, number> = {
  read: 1 << 0,
  write: 1 << 1,
  delete: 1 << 2,
  admin: 1 << 3,
};

function loadKeypair(file: string) {
  const secret = JSON.parse(fs.readFileSync(file, "utf8"));
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

function u32le(n: number): Buffer {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(n, 0);
  return b;
}

function parsePerms(spec: string): BN {
  if (/^\d+$/.test(spec)) return new BN(spec);
  let mask = 0;
  for (const part of spec.split(",").map((s) => s.trim()).filter(Boolean)) {
    const bit = PERM_BITS[part.toLowerCase()];
    if (bit === undefined) throw new Error(`Unknown permission '${part}'`);
    mask |= bit;
  }
  return new BN(mask);
}

function permNames(mask: number): string {
  const names = Object.entries(PERM_BITS)
    .filter(([, bit]) => (mask & bit) === bit)
    .map(([name]) => name);
  return names.length ? names.join("|") : "(none)";
}

function explorer(sig: string, cluster: string): string {
  return `https://explorer.solana.com/tx/${sig}?cluster=${cluster}`;
}

async function main() {
  const [, , cmd, ...args] = process.argv;
  if (!cmd) {
    console.error("No command. See header of app/cli.ts for usage.");
    process.exit(1);
  }

  const url = process.env.ANCHOR_PROVIDER_URL || "https://api.devnet.solana.com";
  const cluster = url.includes("devnet")
    ? "devnet"
    : url.includes("testnet")
    ? "testnet"
    : url.includes("mainnet")
    ? "mainnet-beta"
    : "custom";
  const walletPath =
    process.env.ANCHOR_WALLET || path.join(os.homedir(), ".config/solana/id.json");
  const payer = loadKeypair(walletPath);
  const connection = new Connection(url, "confirmed");
  const provider = new AnchorProvider(connection, new Wallet(payer), {
    commitment: "confirmed",
  });

  const program = new Program(idl as any, provider);
  const programId = program.programId;

  const adminPk = process.env.RBAC_ADMIN
    ? new PublicKey(process.env.RBAC_ADMIN)
    : payer.publicKey;

  const [orgPda] = PublicKey.findProgramAddressSync(
    [ORG_SEED, adminPk.toBuffer()],
    programId
  );
  const rolePda = (id: number) =>
    PublicKey.findProgramAddressSync(
      [ROLE_SEED, orgPda.toBuffer(), u32le(id)],
      programId
    )[0];
  const memberPda = (user: web3.PublicKey) =>
    PublicKey.findProgramAddressSync(
      [MEMBER_SEED, orgPda.toBuffer(), user.toBuffer()],
      programId
    )[0];

  const logTx = (label: string, sig: string) => {
    console.log(`✅ ${label}`);
    console.log(`   tx: ${sig}`);
    console.log(`   ${explorer(sig, cluster)}`);
  };

  switch (cmd) {
    case "init-org": {
      const sig = await program.methods
        .initializeOrganization()
        .accounts({ organization: orgPda, admin: payer.publicKey })
        .rpc();
      logTx(`Organization created: ${orgPda.toBase58()}`, sig);
      break;
    }
    case "create-role": {
      const [idStr, name, permSpec] = args;
      if (!idStr || !name || !permSpec)
        throw new Error("usage: create-role <id> <name> <perms>");
      const id = parseInt(idStr, 10);
      const perms = parsePerms(permSpec);
      const sig = await program.methods
        .createRole(id, name, perms)
        .accounts({ organization: orgPda, role: rolePda(id), admin: payer.publicKey })
        .rpc();
      logTx(`Role '${name}' (id ${id}, perms ${permNames(perms.toNumber())}) created`, sig);
      break;
    }
    case "assign": {
      const [userStr, roleIdStr] = args;
      if (!userStr || !roleIdStr) throw new Error("usage: assign <userPubkey> <roleId>");
      const user = new PublicKey(userStr);
      const roleId = parseInt(roleIdStr, 10);
      const sig = await program.methods
        .assignRole(user)
        .accounts({
          organization: orgPda,
          role: rolePda(roleId),
          membership: memberPda(user),
          admin: payer.publicKey,
        })
        .rpc();
      logTx(`Assigned ${user.toBase58()} to role ${roleId}`, sig);
      break;
    }
    case "set-status": {
      const [userStr, status] = args;
      if (!userStr || !status) throw new Error("usage: set-status <userPubkey> <on|off>");
      const user = new PublicKey(userStr);
      const active = ["on", "true", "1"].includes(status.toLowerCase());
      const sig = await program.methods
        .setMembershipStatus(active)
        .accounts({
          organization: orgPda,
          membership: memberPda(user),
          admin: payer.publicKey,
        })
        .rpc();
      logTx(`Membership ${user.toBase58()} active=${active}`, sig);
      break;
    }
    case "revoke": {
      const [userStr] = args;
      if (!userStr) throw new Error("usage: revoke <userPubkey>");
      const user = new PublicKey(userStr);
      const sig = await program.methods
        .revokeMembership()
        .accounts({
          organization: orgPda,
          membership: memberPda(user),
          admin: payer.publicKey,
        })
        .rpc();
      logTx(`Revoked membership ${user.toBase58()}`, sig);
      break;
    }
    case "perform": {
      const [roleIdStr, permSpec] = args;
      if (!roleIdStr || !permSpec) throw new Error("usage: perform <roleId> <perm>");
      const roleId = parseInt(roleIdStr, 10);
      const perm = parsePerms(permSpec);
      const sig = await program.methods
        .performAction(perm)
        .accounts({
          organization: orgPda,
          role: rolePda(roleId),
          membership: memberPda(payer.publicKey),
          user: payer.publicKey,
        })
        .rpc();
      logTx(`Action authorized (required ${permNames(perm.toNumber())})`, sig);
      break;
    }
    case "show": {
      const org = await program.account.organization.fetch(orgPda);
      console.log(`Organization ${orgPda.toBase58()}`);
      console.log(`  admin:   ${org.admin.toBase58()}`);
      console.log(`  roles:   ${org.roleCount}`);
      console.log(`  members: ${org.memberCount}`);
      const roles = await program.account.role.all();
      console.log("\nRoles:");
      for (const r of roles.filter((x: any) => x.account.organization.equals(orgPda))) {
        console.log(
          `  [${r.account.id}] ${r.account.name} → ${permNames(
            r.account.permissions.toNumber()
          )}`
        );
      }
      const members = await program.account.membership.all();
      console.log("\nMemberships:");
      for (const m of members.filter((x: any) => x.account.organization.equals(orgPda))) {
        console.log(
          `  ${m.account.user.toBase58()} → role ${m.account.role.toBase58()} (active=${m.account.active})`
        );
      }
      break;
    }
    default:
      console.error(`Unknown command '${cmd}'. See header of app/cli.ts for usage.`);
      process.exit(1);
  }
}

main().catch((e) => {
  console.error("❌", e.message || e);
  process.exit(1);
});
