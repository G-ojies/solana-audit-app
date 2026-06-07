/**
 * fair-tickets CLI client
 * -----------------------
 * Drives the ticketing program and doubles as the devnet tx-link generator.
 *
 * Usage:  ts-node app/cli.ts <command> [args]   (or: yarn cli <command>)
 *
 * Commands:
 *   create-event <eventId> <name> <priceSol> <capSol> <supply>
 *   buy <eventId>                          Buy the next ticket at face price
 *   list <eventId> <serial> <priceSol>     List a ticket for resale (<= cap)
 *   cancel <eventId> <serial>              Cancel a resale listing
 *   buy-resale <eventId> <serial> <seller> Buy a listed ticket from <seller>
 *   redeem <eventId> <serial>              Organizer checks a ticket in
 *   show <eventId>                         Show the event + its tickets
 *
 * Config (env):
 *   ANCHOR_PROVIDER_URL   RPC url            (default: https://api.devnet.solana.com)
 *   ANCHOR_WALLET         keypair json path  (default: ~/.config/solana/id.json)
 *   TICKETS_ORGANIZER     organizer pubkey   (default: the wallet's pubkey)
 */
import { AnchorProvider, Program, Wallet, web3, BN } from "@anchor-lang/core";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
// IDL committed under app/idl so the CLI runs from a fresh clone without
// requiring `anchor build` first. Re-copy from target/idl after rebuilds.
import idl from "./idl/fair_tickets.json";

const { PublicKey, Keypair, Connection, LAMPORTS_PER_SOL } = web3;

const EVENT_SEED = Buffer.from("event");
const TICKET_SEED = Buffer.from("ticket");

function loadKeypair(file: string) {
  return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(file, "utf8"))));
}
function u64le(n: number | string): Buffer {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(BigInt(n), 0);
  return b;
}
function u32le(n: number): Buffer {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(n, 0);
  return b;
}
const sol = (lamports: any) => (Number(lamports) / LAMPORTS_PER_SOL).toFixed(4);
const toLamports = (s: string) => new BN(Math.round(parseFloat(s) * LAMPORTS_PER_SOL));

async function main() {
  const [, , cmd, ...args] = process.argv;
  if (!cmd) {
    console.error("No command. See header of app/cli.ts for usage.");
    process.exit(1);
  }

  const url = process.env.ANCHOR_PROVIDER_URL || "https://api.devnet.solana.com";
  const cluster = url.includes("devnet") ? "devnet"
    : url.includes("testnet") ? "testnet"
    : url.includes("mainnet") ? "mainnet-beta" : "custom";
  const walletPath = process.env.ANCHOR_WALLET || path.join(os.homedir(), ".config/solana/id.json");
  const payer = loadKeypair(walletPath);
  const provider = new AnchorProvider(new Connection(url, "confirmed"), new Wallet(payer), {
    commitment: "confirmed",
  });
  const program = new Program(idl as any, provider);
  const pid = program.programId;

  const organizer = process.env.TICKETS_ORGANIZER
    ? new PublicKey(process.env.TICKETS_ORGANIZER)
    : payer.publicKey;

  const eventPda = (eventId: string) =>
    PublicKey.findProgramAddressSync([EVENT_SEED, organizer.toBuffer(), u64le(eventId)], pid)[0];
  const ticketPda = (event: web3.PublicKey, serial: number) =>
    PublicKey.findProgramAddressSync([TICKET_SEED, event.toBuffer(), u32le(serial)], pid)[0];

  const link = (sig: string) => `https://explorer.solana.com/tx/${sig}?cluster=${cluster}`;
  const logTx = (label: string, sig: string) =>
    console.log(`✅ ${label}\n   tx: ${sig}\n   ${link(sig)}`);

  switch (cmd) {
    case "create-event": {
      const [eventId, name, priceSol, capSol, supply] = args;
      if (!eventId || !name || !priceSol || !capSol || !supply)
        throw new Error("usage: create-event <eventId> <name> <priceSol> <capSol> <supply>");
      const evt = eventPda(eventId);
      const sig = await program.methods
        .createEvent(new BN(eventId), name, toLamports(priceSol), toLamports(capSol), parseInt(supply, 10))
        .accounts({ event: evt, organizer: payer.publicKey })
        .rpc();
      logTx(`Event '${name}' created: ${evt.toBase58()} (price ${priceSol} SOL, cap ${capSol} SOL, supply ${supply})`, sig);
      break;
    }
    case "buy": {
      const [eventId] = args;
      if (!eventId) throw new Error("usage: buy <eventId>");
      const evt = eventPda(eventId);
      const ev: any = await program.account.event.fetch(evt);
      const serial = ev.sold; // next serial
      const sig = await program.methods
        .buyTicket()
        .accounts({ event: evt, ticket: ticketPda(evt, serial), buyer: payer.publicKey, organizer })
        .rpc();
      logTx(`Bought ticket #${serial} of '${ev.name}'`, sig);
      break;
    }
    case "list": {
      const [eventId, serial, priceSol] = args;
      if (!eventId || !serial || !priceSol) throw new Error("usage: list <eventId> <serial> <priceSol>");
      const evt = eventPda(eventId);
      const sig = await program.methods
        .listResale(toLamports(priceSol))
        .accounts({ event: evt, ticket: ticketPda(evt, parseInt(serial, 10)), owner: payer.publicKey })
        .rpc();
      logTx(`Listed ticket #${serial} for ${priceSol} SOL`, sig);
      break;
    }
    case "cancel": {
      const [eventId, serial] = args;
      if (!eventId || !serial) throw new Error("usage: cancel <eventId> <serial>");
      const evt = eventPda(eventId);
      const sig = await program.methods
        .cancelResale()
        .accounts({ event: evt, ticket: ticketPda(evt, parseInt(serial, 10)), owner: payer.publicKey })
        .rpc();
      logTx(`Cancelled resale of ticket #${serial}`, sig);
      break;
    }
    case "buy-resale": {
      const [eventId, serial, seller] = args;
      if (!eventId || !serial || !seller) throw new Error("usage: buy-resale <eventId> <serial> <sellerPubkey>");
      const evt = eventPda(eventId);
      const sig = await program.methods
        .buyResale()
        .accounts({
          event: evt,
          ticket: ticketPda(evt, parseInt(serial, 10)),
          buyer: payer.publicKey,
          seller: new PublicKey(seller),
        })
        .rpc();
      logTx(`Bought ticket #${serial} on the resale market`, sig);
      break;
    }
    case "redeem": {
      const [eventId, serial] = args;
      if (!eventId || !serial) throw new Error("usage: redeem <eventId> <serial>");
      const evt = eventPda(eventId);
      const sig = await program.methods
        .redeem()
        .accounts({ event: evt, ticket: ticketPda(evt, parseInt(serial, 10)), organizer: payer.publicKey })
        .rpc();
      logTx(`Redeemed ticket #${serial}`, sig);
      break;
    }
    case "show": {
      const [eventId] = args;
      if (!eventId) throw new Error("usage: show <eventId>");
      const evt = eventPda(eventId);
      const ev: any = await program.account.event.fetch(evt);
      console.log(`Event '${ev.name}'  ${evt.toBase58()}`);
      console.log(`  organizer: ${ev.organizer.toBase58()}`);
      console.log(`  price: ${sol(ev.price)} SOL | resale cap: ${sol(ev.maxResalePrice)} SOL`);
      console.log(`  sold: ${ev.sold}/${ev.supply}`);
      const tickets: any[] = await program.account.ticket.all();
      const mine = tickets.filter((t) => t.account.event.equals(evt));
      console.log("\nTickets:");
      for (const t of mine.sort((a, b) => a.account.serial - b.account.serial)) {
        const a = t.account;
        const state = a.redeemed ? "redeemed" : a.forSale ? `for sale @ ${sol(a.resalePrice)} SOL` : "held";
        console.log(`  #${a.serial}  owner ${a.owner.toBase58().slice(0, 8)}…  paid ${sol(a.paid)} SOL  [${state}]`);
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
