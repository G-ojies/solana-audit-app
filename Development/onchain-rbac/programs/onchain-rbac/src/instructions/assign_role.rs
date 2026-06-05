use anchor_lang::prelude::*;

use crate::constants::{MEMBER_SEED, ORG_SEED, ROLE_SEED};
use crate::state::{Membership, Organization, Role};

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct AssignRole<'info> {
    #[account(
        mut,
        seeds = [ORG_SEED, admin.key().as_ref()],
        bump = organization.bump,
        has_one = admin,
    )]
    pub organization: Account<'info, Organization>,
    #[account(
        seeds = [ROLE_SEED, organization.key().as_ref(), &role.id.to_le_bytes()],
        bump = role.bump,
        has_one = organization,
    )]
    pub role: Account<'info, Role>,
    #[account(
        init,
        payer = admin,
        space = 8 + Membership::INIT_SPACE,
        seeds = [MEMBER_SEED, organization.key().as_ref(), user.as_ref()],
        bump
    )]
    pub membership: Account<'info, Membership>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AssignRole>, user: Pubkey) -> Result<()> {
    let org = &mut ctx.accounts.organization;
    let membership = &mut ctx.accounts.membership;
    membership.organization = org.key();
    membership.user = user;
    membership.role = ctx.accounts.role.key();
    membership.active = true;
    membership.bump = ctx.bumps.membership;

    org.member_count = org.member_count.saturating_add(1);
    msg!("User {} assigned role '{}'", user, ctx.accounts.role.name);
    Ok(())
}
