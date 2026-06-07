use anchor_lang::prelude::*;

use crate::constants::{EVENT_SEED, TICKET_SEED};
use crate::error::TicketError;
use crate::state::{Event, Ticket};

/// Organizer checks a ticket in at the door. A redeemed ticket can no longer be
/// resold — closing the door on "sell after entry" fraud.
#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(
        seeds = [EVENT_SEED, organizer.key().as_ref(), &event.event_id.to_le_bytes()],
        bump = event.bump,
        has_one = organizer,
    )]
    pub event: Account<'info, Event>,
    #[account(
        mut,
        seeds = [TICKET_SEED, event.key().as_ref(), &ticket.serial.to_le_bytes()],
        bump = ticket.bump,
        has_one = event,
    )]
    pub ticket: Account<'info, Ticket>,
    pub organizer: Signer<'info>,
}

pub fn handler(ctx: Context<Redeem>) -> Result<()> {
    require!(!ctx.accounts.ticket.redeemed, TicketError::AlreadyRedeemed);
    let ticket = &mut ctx.accounts.ticket;
    ticket.redeemed = true;
    ticket.for_sale = false;
    ticket.resale_price = 0;
    msg!("Ticket #{} redeemed (checked in)", ticket.serial);
    Ok(())
}
