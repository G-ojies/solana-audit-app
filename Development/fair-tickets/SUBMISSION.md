# fair-tickets — submission write-up

_Paste the section below into the Superteam Earn submission form. Repo link and
Telegram handle go in their own fields._

---

## fair-tickets: event ticketing with an on-chain fair-resale guarantee

**The everyday system:** buying and reselling event tickets.

**The friction it removes:** scalping. Today, resale price caps are *platform
policy* — enforced (if at all) by a marketplace that also profits from resale,
and trivially bypassed by selling off-platform. fair-tickets makes the cap a
property of the ticket itself: the organizer sets a `max_resale_price` at event
creation, and the **program rejects any listing above it**. There is no resale
path that bypasses the cap, because the program *is* the marketplace.

### Why it genuinely belongs on-chain
- **Trustlessness** — the resale ceiling is guaranteed by code, not by a
  platform's promise (or its conflict of interest).
- **Permissionlessness** — resale is peer-to-peer; no gatekeeper approves
  transfers or escrows funds.
- **Atomic token settlement** — payment and ownership transfer happen in one
  instruction. You can't pay without receiving the ticket, or move the ticket
  without paying.

It does exactly one thing well — fair primary + secondary sales with check-in —
with no feature creep.

### Account model (Anchor / Rust)
- `Event` PDA `["event", organizer, event_id]` — price, **max_resale_price**,
  supply, sold.
- `Ticket` PDA `["ticket", event, serial]` — `owner` is custody (changing it
  *is* the transfer), `for_sale`, `resale_price`, `redeemed`.

### Instructions
`create_event` · `buy_ticket` (primary) · `list_resale` (price ≤ cap enforced) ·
`cancel_resale` · `buy_resale` (atomic P2P pay + transfer) · `redeem` (check-in
disables resale).

### Correctness & testing
litesvm integration suite, all passing:
- primary sale transfers face price to organizer and records ownership
- listing above the cap is rejected; at the cap succeeds
- resale flips ownership AND moves exactly the resale price to the seller
- a redeemed (checked-in) ticket can no longer be resold
- only the ticket owner can list

### Live on devnet (full lifecycle)
One ticket goes primary sale → listed under the cap → **resold to a second
wallet at the capped price** → checked in.

- Program: `Ggb46RAz1cWos2fQp4eWorFPpjPWouTBzccyCDVMGcgU`
- create_event: https://explorer.solana.com/tx/xTRfYjRsif6tzK6Ko9hXqiCqZrxDChBEVvspzMYG6vjsKf1d3uVqaDMoLpxLh3hypAhT6SRd1nr7zqMAPnaLHxi?cluster=devnet
- buy_ticket: https://explorer.solana.com/tx/2qbdw58hyD7dB9LnsnaUudbiz82xZSZT5Tn4dPqEoXBduidqhx3otjPPrR8SVUCFETsKtEdzYs2M2HrRRmQM1tYy?cluster=devnet
- list_resale: https://explorer.solana.com/tx/2S5QaXx2V4qjcKq5THBWzrC5jf3SWyA7ecvSPXXgXHEDCzPhH9iDdWEzJuH3uHPooscyjVfSqo8VkXTqjxjkrXx1?cluster=devnet
- buy_resale: https://explorer.solana.com/tx/2r5AFQ7FqVUf77ei1incuuyjQ761yS1YUztqXwjDohYRab8m2PX51naEiYAfTZG2yE1xFNQ8hvAbDp4VySvz5VRP?cluster=devnet
- redeem: https://explorer.solana.com/tx/2P66BPfu8Cbn1YC21CgStjFAocB9ipFmJzC2kEpcXfEBx81Lc3SZKBAdujfmqynZVyd1cN8wMDh3R2gFbTK6xRqg?cluster=devnet

### Testable client
A TypeScript CLI drives every instruction and prints an Explorer link per
transaction:
```
yarn cli create-event 1 "Solana Summit" 0.1 0.12 5
yarn cli buy 1
yarn cli list 1 0 0.11
yarn cli buy-resale 1 0 <SELLER_PUBKEY>
yarn cli redeem 1 0
yarn cli show 1
```

**Repo (code, tests, README with full Web2→Solana analysis & tradeoffs):**
https://github.com/G-ojies/solana-audit-app/tree/master/Development/fair-tickets
