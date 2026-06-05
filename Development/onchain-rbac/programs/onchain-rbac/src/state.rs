use anchor_lang::prelude::*;

use crate::constants::MAX_ROLE_NAME_LEN;

/// An RBAC namespace owned by a single admin.
///
/// Web2 analogue: a `tenants`/`organizations` row plus the app-level notion of
/// "who is the owner". On Solana the owner is just a pubkey stored in the
/// account and enforced by `has_one` constraints on every privileged action.
#[account]
#[derive(InitSpace)]
pub struct Organization {
    /// The authority allowed to create roles and manage memberships.
    pub admin: Pubkey,
    /// Monotonic counter used to mint role ids.
    pub role_count: u32,
    /// Number of currently-assigned memberships.
    pub member_count: u32,
    pub bump: u8,
}

/// A named bundle of permissions. Web2 analogue: a `roles` table row with a
/// `permissions` bitmask / scopes column.
#[account]
#[derive(InitSpace)]
pub struct Role {
    pub organization: Pubkey,
    pub id: u32,
    /// u64 bitmask — up to 64 independent capabilities per role.
    pub permissions: u64,
    #[max_len(MAX_ROLE_NAME_LEN)]
    pub name: String,
    pub bump: u8,
}

/// Assignment of a user to a role within an organization. Web2 analogue: a
/// `user_roles` join-table row. The PDA address is derived from
/// (organization, user), so a user can hold at most one membership per org and
/// it is addressable without an index.
#[account]
#[derive(InitSpace)]
pub struct Membership {
    pub organization: Pubkey,
    pub user: Pubkey,
    pub role: Pubkey,
    pub active: bool,
    pub bump: u8,
}
