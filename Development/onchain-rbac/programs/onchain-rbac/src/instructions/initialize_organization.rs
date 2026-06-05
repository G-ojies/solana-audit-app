use anchor_lang::prelude::*;

use crate::constants::ORG_SEED;
use crate::state::Organization;

#[derive(Accounts)]
pub struct InitializeOrganization<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Organization::INIT_SPACE,
        seeds = [ORG_SEED, admin.key().as_ref()],
        bump
    )]
    pub organization: Account<'info, Organization>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeOrganization>) -> Result<()> {
    let org = &mut ctx.accounts.organization;
    org.admin = ctx.accounts.admin.key();
    org.role_count = 0;
    org.member_count = 0;
    org.bump = ctx.bumps.organization;
    msg!("Organization initialized for admin {}", org.admin);
    Ok(())
}
