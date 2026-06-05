use anchor_lang::prelude::*;

use crate::constants::{MAX_ROLE_NAME_LEN, ORG_SEED, ROLE_SEED};
use crate::error::RbacError;
use crate::state::{Organization, Role};

#[derive(Accounts)]
#[instruction(id: u32)]
pub struct CreateRole<'info> {
    #[account(
        mut,
        seeds = [ORG_SEED, admin.key().as_ref()],
        bump = organization.bump,
        has_one = admin,
    )]
    pub organization: Account<'info, Organization>,
    #[account(
        init,
        payer = admin,
        space = 8 + Role::INIT_SPACE,
        seeds = [ROLE_SEED, organization.key().as_ref(), &id.to_le_bytes()],
        bump
    )]
    pub role: Account<'info, Role>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateRole>, id: u32, name: String, permissions: u64) -> Result<()> {
    require!(name.len() <= MAX_ROLE_NAME_LEN, RbacError::NameTooLong);

    let org = &mut ctx.accounts.organization;
    let role = &mut ctx.accounts.role;
    role.organization = org.key();
    role.id = id;
    role.permissions = permissions;
    role.name = name;
    role.bump = ctx.bumps.role;

    org.role_count = org.role_count.saturating_add(1);
    msg!(
        "Role '{}' (id {}) created with permission mask {:#x}",
        role.name,
        role.id,
        role.permissions
    );
    Ok(())
}
