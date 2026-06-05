use anchor_lang::prelude::*;

use crate::constants::{MEMBER_SEED, ORG_SEED};
use crate::state::{Membership, Organization};

#[derive(Accounts)]
pub struct SetMembershipStatus<'info> {
    #[account(
        seeds = [ORG_SEED, admin.key().as_ref()],
        bump = organization.bump,
        has_one = admin,
    )]
    pub organization: Account<'info, Organization>,
    #[account(
        mut,
        seeds = [MEMBER_SEED, organization.key().as_ref(), membership.user.as_ref()],
        bump = membership.bump,
        has_one = organization,
    )]
    pub membership: Account<'info, Membership>,
    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<SetMembershipStatus>, active: bool) -> Result<()> {
    let membership = &mut ctx.accounts.membership;
    membership.active = active;
    msg!(
        "Membership for {} set active={}",
        membership.user,
        membership.active
    );
    Ok(())
}
