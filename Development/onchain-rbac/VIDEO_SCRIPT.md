# onchain-rbac — 3-minute pitch video script + shot list

Target: ≤ 3:00. Screen recording + voiceover. Keep cuts tight; show real
terminal/Explorer, not slides where you can avoid it.

Setup before recording:
- Terminal in the repo, font bumped up, `clear` between commands.
- Browser tab open to the devnet program on Explorer.
- Have these ready to paste so you don't type live:
  `yarn cli init-org`, `create-role`, `assign`, `perform`.

---

## 0:00–0:20 — Hook & what it is
**Voiceover:**
"Most production backends gate every action with role-based access control —
users get roles, roles carry permissions, and a middleware checks 'are you
allowed?'. I rebuilt that core pattern as a Solana program, where the
permission check isn't app code you can forget — it's enforced by the runtime."

**Shot:** title card or the README header, then cut to the account-model diagram
in the README.

## 0:20–0:50 — The Web2 → Solana mapping
**Voiceover:**
"In Web2 this is three Postgres tables — organizations, roles, user_roles — and
an `if role.permissions & WRITE` check in middleware. On Solana each row becomes
a PDA: an Organization owned by an admin, Role accounts holding a u64 permission
bitmask, and Membership accounts keyed by (org, user). The owner check is a
`has_one` constraint; identity is the transaction signer — there's no session to
forge."

**Shot:** scroll the README's "How this works in Web2 / on Solana" table, then
flash `state.rs` (the three account structs).

## 0:50–1:50 — Live demo on devnet (the core)
**Voiceover (narrate as you run each):**
"Let's run it against devnet. Create an organization… define an editor role with
read+write, and a viewer role with read only… assign a user to editor… now the
guarded action: perform a WRITE — authorized, because the role grants it."

**Shot:** run in sequence, let each tx + Explorer link print:
```
yarn cli init-org
yarn cli create-role 0 editor read,write
yarn cli create-role 1 viewer read
yarn cli assign <USER> 0
yarn cli perform 0 write      # ✅ authorized
```
Click one Explorer link to show the real on-chain tx.

## 1:50–2:20 — The part that matters: denial is enforced
**Voiceover:**
"The point isn't that it works — it's that it can't be bypassed. A viewer trying
to write is rejected by the program. Deactivate a membership and even a valid
role is denied. These aren't app checks — they're runtime constraints, proven by
the test suite."

**Shot:** run `yarn cli perform 1 write` as the viewer → error; then show
`cargo test` output (4 passing: write allowed, write denied, inactive denied,
non-admin denied).

## 2:20–2:50 — Tradeoffs (shows judgment)
**Voiceover:**
"Honest tradeoffs: every role and membership is a rent-backed account, so this
isn't free like a SQL row — revoke refunds the rent. Listing all members is a
getProgramAccounts scan, not an indexed query. And admin is a single key today —
but since it's just a Pubkey, swapping in a multisig or governance PDA is
trivial."

**Shot:** scroll the README "Tradeoffs & constraints" section.

## 2:50–3:00 — Close
**Voiceover:**
"Full source, tests, CLI, and the devnet transaction links are in the repo.
That's RBAC as an on-chain state machine."

**Shot:** repo URL on screen:
`github.com/G-ojies/solana-audit-app/tree/master/Development/onchain-rbac`

---

### Tips
- Record the demo once end-to-end first; if a devnet tx is slow, cut the dead air.
- If you re-run the demo, use a fresh keypair or you'll hit "already initialized"
  on `init-org` (the org PDA is per-admin).
- Aim to finish ~0:10 under budget — judges prefer tight.
