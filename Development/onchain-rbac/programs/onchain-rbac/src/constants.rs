use anchor_lang::prelude::*;

/// PDA seed prefixes. In a Web2 backend these would be table names / key
/// prefixes; on Solana they namespace deterministic account addresses.
#[constant]
pub const ORG_SEED: &[u8] = b"org";

#[constant]
pub const ROLE_SEED: &[u8] = b"role";

#[constant]
pub const MEMBER_SEED: &[u8] = b"member";

/// Maximum bytes allowed for a human-readable role name.
pub const MAX_ROLE_NAME_LEN: usize = 32;

// Example permission bits. Permissions are a u64 bitmask, so a single role can
// carry up to 64 independent capabilities — the on-chain analogue of a Web2
// permissions/scopes column.
pub const PERM_READ: u64 = 1 << 0;
pub const PERM_WRITE: u64 = 1 << 1;
pub const PERM_DELETE: u64 = 1 << 2;
pub const PERM_ADMIN: u64 = 1 << 3;
