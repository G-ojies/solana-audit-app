use anchor_lang::prelude::*;

use crate::constants::{MEMBER_SEED, ORG_SEED};
use crate::state::{Membership, Organization};

#[derive(Accounts)]
pub struct RevokeMembership<'info> {
    #[account(
        mut,
        seeds = [ORG_SEED, admin.key().as_ref()],
        bump = organization.bump,
        has_one = admin,
    )]
    pub organization: Account<'info, Organization>,
    // Closing returns the rent lamports to the admin. Web2 analogue: deleting
    // the user_roles row. On Solana, account closure is explicit and refundable.
    #[account(
        mut,
        close = admin,
        seeds = [MEMBER_SEED, organization.key().as_ref(), membership.user.as_ref()],
        bump = membership.bump,
        has_one = organization,
    )]
    pub membership: Account<'info, Membership>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<RevokeMembership>) -> Result<()> {
    let org = &mut ctx.accounts.organization;
    org.member_count = org.member_count.saturating_sub(1);
    msg!("Membership for {} revoked", ctx.accounts.membership.user);
    Ok(())
}
