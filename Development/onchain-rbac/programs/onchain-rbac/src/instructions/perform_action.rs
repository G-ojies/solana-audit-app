use anchor_lang::prelude::*;

use crate::constants::{MEMBER_SEED, ROLE_SEED};
use crate::error::RbacError;
use crate::state::{Membership, Organization, Role};

/// A guarded action: it succeeds only if the signer holds an active membership
/// whose role grants every bit in `required_permission`.
///
/// This is the on-chain equivalent of an auth-middleware guard wrapping a
/// protected endpoint — except the check is enforced by the runtime, not by a
/// service the caller could bypass.
#[derive(Accounts)]
pub struct PerformAction<'info> {
    pub organization: Account<'info, Organization>,
    #[account(
        seeds = [ROLE_SEED, organization.key().as_ref(), &role.id.to_le_bytes()],
        bump = role.bump,
        has_one = organization,
    )]
    pub role: Account<'info, Role>,
    #[account(
        seeds = [MEMBER_SEED, organization.key().as_ref(), user.key().as_ref()],
        bump = membership.bump,
        has_one = organization,
        has_one = user,
        constraint = membership.role == role.key() @ RbacError::RoleMismatch,
    )]
    pub membership: Account<'info, Membership>,
    pub user: Signer<'info>,
}

pub fn handler(ctx: Context<PerformAction>, required_permission: u64) -> Result<()> {
    let membership = &ctx.accounts.membership;
    require!(membership.active, RbacError::MembershipInactive);

    let role = &ctx.accounts.role;
    require!(
        role.permissions & required_permission == required_permission,
        RbacError::PermissionDenied
    );

    msg!(
        "Action authorized for {} via role '{}' (required {:#x}, held {:#x})",
        membership.user,
        role.name,
        required_permission,
        role.permissions
    );
    Ok(())
}
