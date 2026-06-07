use anchor_lang::prelude::*;

use crate::constants::{EVENT_SEED, MAX_EVENT_NAME_LEN};
use crate::error::TicketError;
use crate::state::Event;

#[derive(Accounts)]
#[instruction(event_id: u64)]
pub struct CreateEvent<'info> {
    #[account(
        init,
        payer = organizer,
        space = 8 + Event::INIT_SPACE,
        seeds = [EVENT_SEED, organizer.key().as_ref(), &event_id.to_le_bytes()],
        bump
    )]
    pub event: Account<'info, Event>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateEvent>,
    event_id: u64,
    name: String,
    price: u64,
    max_resale_price: u64,
    supply: u32,
) -> Result<()> {
    require!(name.len() <= MAX_EVENT_NAME_LEN, TicketError::NameTooLong);

    let event = &mut ctx.accounts.event;
    event.organizer = ctx.accounts.organizer.key();
    event.event_id = event_id;
    event.name = name;
    event.price = price;
    event.max_resale_price = max_resale_price;
    event.supply = supply;
    event.sold = 0;
    event.bump = ctx.bumps.event;

    msg!(
        "Event '{}' created: price {} lamports, resale cap {}, supply {}",
        event.name,
        event.price,
        event.max_resale_price,
        event.supply
    );
    Ok(())
}
