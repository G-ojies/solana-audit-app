# fair-tickets — event ticketing with an on-chain fair-resale guarantee

An everyday system — buying and reselling event tickets — rebuilt as a Solana
program. The friction it removes is **scalping**: the organizer sets a resale
price ceiling that the **program itself enforces**, so no secondary buyer can be
gouged and no platform has to be trusted to police the resale market.

Built for the Superteam **"Build Everyday Real-World Systems as On-Chain Rust
Programs"** challenge. Stack: Anchor 1.0 / Rust, with a TypeScript CLI client.

---

## Why this belongs on-chain

Ticketing is the textbook case where blockchain's properties solve a *daily*
problem rather than being bolted on:

- **Trustlessness** — the resale price cap is guaranteed by code, not by a
  platform promising to fight scalpers (while taking a cut of every resale).
- **Permissionlessness** — resale is peer-to-peer; no marketplace gatekeeper has
  to approve a transfer or hold funds in escrow.
- **Token infrastructure** — payment and ownership transfer settle atomically in
  one transaction; you cannot pay without receiving the ticket, or transfer the
  ticket without paying.

It deliberately does **one** thing well (fair primary + secondary sales with
check-in) — no feature creep.

---

## How this friction works in traditional systems

```
Ticketmaster-style backend
  events(id, price, supply, ...)            -- private DB
  tickets(id, event_id, owner_user_id, ...) -- platform owns the row
  resale: platform-run marketplace sets prices, holds escrow, takes fees
```

- The ticket is a **row the platform controls**; you have a claim, not custody.
- Resale price caps (where they exist) are **policy**, enforced by the platform's
  own marketplace — which also profits from resale, a conflict of interest.
- Off-platform resale (StubHub, a friend, a scalper) is **invisible and
  uncapped**; the original seller can't guarantee a fair price to the next buyer.

## How this works on Solana

| Traditional                          | fair-tickets                                                  |
| ------------------------------------ | ------------------------------------------------------------ |
| `events` row in a private DB         | `Event` PDA, seeds `["event", organizer, event_id]`          |
| `tickets` row owned by the platform  | `Ticket` PDA, seeds `["ticket", event, serial]`; `owner` field is custody |
| resale cap = platform policy         | `max_resale_price` on the event, checked in `list_resale`    |
| marketplace escrow + fees            | peer-to-peer SOL transfer in `buy_resale`, atomic with transfer |
| "trust us not to scalp"              | the program **rejects** any listing above the cap            |
| platform mediates every transfer     | permissionless: any owner lists, any buyer buys              |

Key shifts:

- **The ticket is custody, not a claim.** `Ticket.owner` is the source of truth;
  changing it *is* the transfer, and it happens in the same instruction as
  payment — no double-sell, no "paid but didn't receive".
- **The cap is code.** `list_resale` does `require!(price <= event.max_resale_price)`.
  There is no listing path that bypasses it, on- or off-"platform", because the
  program *is* the platform.
- **Check-in closes the loop.** `redeem` flips `redeemed = true`, which disables
  resale — you can't sell a ticket you've already walked in with.

---

## Account model

```
Event   (PDA: ["event", organizer, event_id])
  organizer: Pubkey
  event_id: u64
  name: String (≤64)
  price: u64            // primary price, lamports
  max_resale_price: u64 // the anti-scalp ceiling, lamports
  supply: u32
  sold: u32
  bump: u8

Ticket  (PDA: ["ticket", event, serial])
  event: Pubkey
  serial: u32
  owner: Pubkey         // custody — changing this is the transfer
  paid: u64
  for_sale: bool
  resale_price: u64
  redeemed: bool        // checked in at the door
  bump: u8
```

### Instructions

| Instruction     | Authority | Effect                                                       |
| --------------- | --------- | ------------------------------------------------------------ |
| `create_event`  | organizer | Create an event with price, **resale cap**, and supply.      |
| `buy_ticket`    | anyone    | Primary sale: pay organizer face price, mint next serial.    |
| `list_resale`   | owner     | List a ticket for resale — **price must be ≤ the cap**.      |
| `cancel_resale` | owner     | Remove a listing.                                            |
| `buy_resale`    | anyone    | Pay the seller (capped price) peer-to-peer; ownership flips. |
| `redeem`        | organizer | Check a ticket in; disables further resale.                  |

Enforcement is layered: PDA seeds bind every ticket to its event; `has_one`
ties actions to the right owner/organizer; the cap and `redeemed`/`for_sale`
flags are explicit `require!` gates; payment and ownership change happen in one
atomic instruction.

---

## Tradeoffs & constraints

- **Rent per ticket.** Each ticket is a rent-exempt account (~0.0015 SOL), unlike
  a free DB row. Fine for events; a closeable ticket after `redeem` could reclaim
  it (left out to avoid feature creep).
- **Ownership is a field, not an SPL/NFT.** This keeps the model minimal and
  legible. Trade-off: tickets aren't visible in wallets as NFTs and can't ride
  existing NFT marketplaces — but that's the point here (resale must go through
  the capped path, not an arbitrary marketplace).
- **Cap is set once.** `max_resale_price` is fixed at creation; a dynamic cap
  (e.g. ≤ face value) would be a small extension.
- **No partial/edge market features.** No auctions, no royalties, no waitlists —
  intentionally. The challenge rewards a perfected minimal core.
- **Organizer is a single key.** It's just a `Pubkey`, so a multisig or DAO can
  be the organizer with no code change.
- **Listing visibility.** "All tickets for sale" is a `getProgramAccounts` scan
  (the CLI `show` does this); a production UI would pair it with an indexer.

---

## Build, test, deploy

```bash
anchor build
cargo test        # litesvm suite (see below)
solana config set --url https://api.devnet.solana.com
anchor deploy --provider.cluster devnet
```

**Program ID:** `Ggb46RAz1cWos2fQp4eWorFPpjPWouTBzccyCDVMGcgU`

### Tests (`programs/fair-tickets/tests/test_tickets.rs`)

- `primary_sale_works` — buyer pays organizer face price; `sold` increments; ownership recorded.
- `resale_above_cap_rejected` — listing above the cap fails; at the cap succeeds.
- `buy_resale_transfers_ownership_and_funds` — ownership flips and the seller receives exactly the resale price.
- `redeemed_ticket_cannot_be_resold` — `redeem` blocks subsequent listing.
- `non_owner_cannot_list` — only the ticket owner can list.

---

## CLI client (`app/cli.ts`)

```bash
yarn install
yarn cli create-event 1 "Solana Summit" 0.1 0.12 5   # price 0.1, resale cap 0.12 SOL
yarn cli buy 1                                        # primary buy (next serial)
yarn cli list 1 0 0.11                                # list ticket #0 at 0.11 (≤ cap)
yarn cli buy-resale 1 0 <SELLER_PUBKEY>              # buy it on the resale market
yarn cli redeem 1 0                                   # organizer checks it in
yarn cli show 1                                       # event + ticket states
```

Prices are in SOL; the CLI prints a transaction signature + Explorer link for
every write. Set `ANCHOR_WALLET` / `TICKETS_ORGANIZER` to act as different parties.

---

## Devnet deployment & transaction links

Deployed and exercised end-to-end on **devnet** — one ticket goes through the
full lifecycle: primary sale → listed under the cap → resold to a second wallet
at the capped price → checked in.

**Program:**
[`Ggb46RAz1cWos2fQp4eWorFPpjPWouTBzccyCDVMGcgU`](https://explorer.solana.com/address/Ggb46RAz1cWos2fQp4eWorFPpjPWouTBzccyCDVMGcgU?cluster=devnet)
&nbsp;|&nbsp; **Event PDA:**
[`BK3Gcts2nCHwmfH6HfHnySfsWfVKWdpFSMygc5DHhBzJ`](https://explorer.solana.com/address/BK3Gcts2nCHwmfH6HfHnySfsWfVKWdpFSMygc5DHhBzJ?cluster=devnet)

| Step | Instruction | Transaction |
| ---- | ----------- | ----------- |
| 1 | `create_event` (price 0.1, cap 0.12) | [`xTRfYj…naLHxi`](https://explorer.solana.com/tx/xTRfYjRsif6tzK6Ko9hXqiCqZrxDChBEVvspzMYG6vjsKf1d3uVqaDMoLpxLh3hypAhT6SRd1nr7zqMAPnaLHxi?cluster=devnet) |
| 2 | `buy_ticket` (primary, serial #0) | [`2qbdw5…QM1tYy`](https://explorer.solana.com/tx/2qbdw58hyD7dB9LnsnaUudbiz82xZSZT5Tn4dPqEoXBduidqhx3otjPPrR8SVUCFETsKtEdzYs2M2HrRRmQM1tYy?cluster=devnet) |
| 3 | `list_resale` (0.11 ≤ cap) | [`2S5QaX…jkrXx1`](https://explorer.solana.com/tx/2S5QaXx2V4qjcKq5THBWzrC5jf3SWyA7ecvSPXXgXHEDCzPhH9iDdWEzJuH3uHPooscyjVfSqo8VkXTqjxjkrXx1?cluster=devnet) |
| 4 | `buy_resale` (second wallet, capped) | [`2r5AFQ…vz5VRP`](https://explorer.solana.com/tx/2r5AFQ7FqVUf77ei1incuuyjQ761yS1YUztqXwjDohYRab8m2PX51naEiYAfTZG2yE1xFNQ8hvAbDp4VySvz5VRP?cluster=devnet) |
| 5 | `redeem` (check-in) | [`2P66BP…TK6xRqg`](https://explorer.solana.com/tx/2P66BPfu8Cbn1YC21CgStjFAocB9ipFmJzC2kEpcXfEBx81Lc3SZKBAdujfmqynZVyd1cN8wMDh3R2gFbTK6xRqg?cluster=devnet) |

---

## Repository layout

```
programs/fair-tickets/src/
  lib.rs                # program entrypoints
  state.rs              # Event / Ticket accounts
  constants.rs          # PDA seeds
  error.rs              # typed errors
  instructions/         # one file per instruction (Accounts + handler)
programs/fair-tickets/tests/
  test_tickets.rs       # litesvm integration tests
app/cli.ts              # TypeScript CLI client
```
