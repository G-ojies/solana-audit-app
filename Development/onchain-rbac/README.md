# onchain-rbac — Role-Based Access Control as a Solana program

A classic Web2 backend pattern — **organizations, roles, and per-user role
assignments guarding protected actions** — rebuilt as a Solana program where
every record is an account (PDA) and every authorization check is enforced by
the runtime instead of by application middleware.

Built for the Superteam **"Rebuild Backend Systems as On-Chain Rust Programs"**
challenge. Stack: Anchor 1.0 / Rust, with a TypeScript CLI client.

---

## The pattern: RBAC

RBAC is the authorization model behind most production backends: users are
assigned **roles**, roles carry **permissions**, and protected operations check
"does the caller's role grant permission X?". Think of an admin dashboard where
*editors* can write content, *viewers* can only read, and only an *owner* can
manage members.

This program implements the core of that model:

| Concept        | Meaning                                                        |
| -------------- | -------------------------------------------------------------- |
| `Organization` | An RBAC namespace owned by a single admin authority.           |
| `Role`         | A named permission bitmask (up to 64 capabilities per role).   |
| `Membership`   | An assignment of one user to one role within an organization.  |
| `perform_action` | A guarded instruction that succeeds only if the caller's role grants the required permission bits. |

---

## How this works in Web2

A typical Web2 RBAC service looks like:

```
Postgres
  organizations(id, owner_id, ...)
  roles(id, org_id, name, permissions BIGINT)        -- bitmask / scopes
  user_roles(user_id, role_id, active)               -- join table

API (Express/Rails/etc.)
  authMiddleware:  load user → load role → if (role.permissions & REQUIRED) next() else 403
  POST /resource:  authMiddleware(WRITE) → handler
```

- **State** lives in database rows; the app server owns and mutates them.
- **Identity** comes from a session/JWT the server issues and trusts.
- **Enforcement** is *advisory*: the `403` check lives in application code. Any
  path that forgets the middleware, or any direct DB access, bypasses it.
- **Source of truth** is private and mutable by whoever holds DB credentials.

## How this works on Solana

| Web2                              | This program                                                     |
| --------------------------------- | ---------------------------------------------------------------- |
| `organizations` row               | `Organization` PDA, seeds `["org", admin]`                       |
| `roles` row                       | `Role` PDA, seeds `["role", org, id]`                            |
| `user_roles` join row             | `Membership` PDA, seeds `["member", org, user]`                  |
| `owner_id` column + app check     | `has_one = admin` constraint on every privileged instruction     |
| `permissions BIGINT` bitmask      | `permissions: u64` bitmask on `Role`                             |
| auth middleware (`if ... 403`)    | `perform_action` runtime constraints + explicit permission check |
| server-issued session/JWT         | the transaction **signer** (`Signer<'info>`) — no session to forge |
| `SELECT` by indexed columns       | deterministic PDA derivation (no index needed; address *is* the key) |

Key shifts:

- **Identity is the signature.** There is no session to steal or JWT to forge —
  the caller proves identity by signing the transaction, and the program reads
  `user.key()` directly.
- **Enforcement is mandatory.** The permission check in `perform_action` and the
  `has_one = admin` / `has_one = organization` constraints are evaluated by the
  runtime on every call. There is no "unprotected route" to forget; the only way
  to mutate state is through the instructions, and every instruction carries its
  guards.
- **Addressing replaces indexing.** A `(org, user)` membership is found by
  deriving its PDA, not by querying an index. One membership per user per org is
  structurally guaranteed by the seeds.
- **State is public and rent-backed.** Anyone can read the role/permission graph;
  accounts cost rent, and deleting a membership (`revoke_membership`) explicitly
  refunds it via `close`.

---

## Account model

```
Organization  (PDA: ["org", admin])
  admin: Pubkey            // the sole authority for management actions
  role_count: u32
  member_count: u32
  bump: u8

Role  (PDA: ["role", organization, id_le_bytes])
  organization: Pubkey
  id: u32
  permissions: u64         // bitmask: READ=1, WRITE=2, DELETE=4, ADMIN=8, ...
  name: String (≤32)
  bump: u8

Membership  (PDA: ["member", organization, user])
  organization: Pubkey
  user: Pubkey
  role: Pubkey
  active: bool
  bump: u8
```

### Instructions

| Instruction               | Authority | Effect                                              |
| ------------------------- | --------- | --------------------------------------------------- |
| `initialize_organization` | anyone    | Creates an org owned by the signer.                 |
| `create_role`             | admin     | Defines a role with a permission bitmask.           |
| `assign_role`             | admin     | Creates an active membership for a user.            |
| `set_membership_status`   | admin     | Activates / deactivates a membership.               |
| `revoke_membership`       | admin     | Closes a membership and refunds rent.               |
| `perform_action`          | member    | Guarded action: requires the role to grant the bits.|

Authorization is layered:
1. **`has_one = admin`** gates every management instruction to the org owner.
2. **PDA seed derivation** ties each `Role`/`Membership` to its organization —
   you cannot mix a role from org A with a membership from org B.
3. **`has_one = user` + `membership.role == role.key()`** on `perform_action`
   binds the signer to their own membership and the named role.
4. **`active` flag + bitmask check** is the final permission gate.

---

## Tradeoffs & constraints

- **Rent, not free rows.** Every role and membership is a rent-exempt account
  (~0.001–0.002 SOL). At very large member counts this is a real cost; a Web2
  table row is effectively free. `revoke_membership` refunds rent on cleanup.
- **No server-side joins or scans.** Listing "all members of org X" means a
  `getProgramAccounts` filter scan (the CLI `show` does this), which is heavier
  than an indexed SQL query. Production systems usually pair this with an indexer.
- **Bitmask cap.** A single `u64` role holds 64 distinct permissions. Beyond
  that you'd shard permissions across multiple role accounts.
- **One membership per (org, user).** The PDA seeds enforce this by design; a
  multi-role-per-user model would change the seed scheme (e.g. include role id).
- **Admin is a single key.** A real deployment would likely make `admin` a
  multisig or a governance PDA; the model already supports that since `admin` is
  just a `Pubkey` (swap in any signer).
- **Compute.** Checks are O(1) and cheap; this is not a compute-bound program.

---

## Build, test, deploy

```bash
# Build the program + IDL
anchor build

# Run the test suite (litesvm — happy path, permission-denied, inactive, non-admin)
cargo test

# Deploy to devnet
solana config set --url https://api.devnet.solana.com
anchor deploy --provider.cluster devnet
```

**Program ID:** `8VF17ZETUYhtM4omTTgUa6ghESD7J9L9tcG5FJmMkoa2`

### Tests

`programs/onchain-rbac/tests/test_rbac.rs` runs against `litesvm` and covers:

- `editor_can_write` — a role with `READ|WRITE` passes a `WRITE`-gated action.
- `viewer_cannot_write` — a `READ`-only role is denied `WRITE` but allowed `READ`.
- `deactivated_membership_is_denied` — `set_membership_status(false)` blocks actions.
- `non_admin_cannot_create_role` — `has_one = admin` rejects a non-owner.

---

## CLI client

A minimal TypeScript CLI (`app/cli.ts`) drives every instruction. It reads the
wallet from `ANCHOR_WALLET` (default `~/.config/solana/id.json`) and the RPC from
`ANCHOR_PROVIDER_URL` (default devnet).

```bash
yarn install

# you = admin of your own org
yarn cli init-org
yarn cli create-role 0 editor read,write
yarn cli create-role 1 viewer read
yarn cli assign <USER_PUBKEY> 0          # assign user to the editor role
yarn cli perform 0 write                 # guarded action (run as the assigned user)
yarn cli set-status <USER_PUBKEY> off    # deactivate
yarn cli revoke <USER_PUBKEY>            # delete + refund rent
yarn cli show                            # print org, roles, memberships
```

Permissions accept names (`read,write,delete,admin`) or a raw number. Every
write command prints the transaction signature and an Explorer link.

---

## Devnet deployment & transaction links

<!-- DEVNET_LINKS -->
_Populated after `anchor deploy` to devnet — program account + sample
instruction transactions (init-org, create-role, assign, perform) on Solana
Explorer (`?cluster=devnet`)._

---

## Repository layout

```
programs/onchain-rbac/src/
  lib.rs                       # program entrypoints
  state.rs                     # Organization / Role / Membership accounts
  constants.rs                 # PDA seeds + permission bits
  error.rs                     # typed errors
  instructions/                # one file per instruction (Accounts + handler)
programs/onchain-rbac/tests/
  test_rbac.rs                 # litesvm integration tests
app/cli.ts                     # TypeScript CLI client
```
