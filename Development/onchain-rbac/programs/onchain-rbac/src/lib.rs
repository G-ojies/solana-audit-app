pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8VF17ZETUYhtM4omTTgUa6ghESD7J9L9tcG5FJmMkoa2");

/// On-chain Role-Based Access Control.
///
/// A classic Web2 backend pattern — organizations, roles, and per-user role
/// assignments guarding protected actions — rebuilt as a Solana program where
/// every record is a PDA and every guard is enforced by the runtime.
#[program]
pub mod onchain_rbac {
    use super::*;

    /// Create an organization (RBAC namespace) owned by the signer.
    pub fn initialize_organization(ctx: Context<InitializeOrganization>) -> Result<()> {
        instructions::initialize_organization::handler(ctx)
    }

    /// Admin: define a role with a permission bitmask.
    pub fn create_role(
        ctx: Context<CreateRole>,
        id: u32,
        name: String,
        permissions: u64,
    ) -> Result<()> {
        instructions::create_role::handler(ctx, id, name, permissions)
    }

    /// Admin: assign a user to a role (creates an active membership).
    pub fn assign_role(ctx: Context<AssignRole>, user: Pubkey) -> Result<()> {
        instructions::assign_role::handler(ctx, user)
    }

    /// Admin: activate or deactivate an existing membership.
    pub fn set_membership_status(ctx: Context<SetMembershipStatus>, active: bool) -> Result<()> {
        instructions::set_membership_status::handler(ctx, active)
    }

    /// Admin: revoke a membership and refund its rent.
    pub fn revoke_membership(ctx: Context<RevokeMembership>) -> Result<()> {
        instructions::revoke_membership::handler(ctx)
    }

    /// Guarded action: succeeds only if the signer's role grants the
    /// required permission bits.
    pub fn perform_action(ctx: Context<PerformAction>, required_permission: u64) -> Result<()> {
        instructions::perform_action::handler(ctx, required_permission)
    }
}
