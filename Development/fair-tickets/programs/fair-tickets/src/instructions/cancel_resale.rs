use anchor_lang::prelude::*;

use crate::constants::TICKET_SEED;
use crate::error::TicketError;
use crate::state::{Event, Ticket};

#[derive(Accounts)]
pub struct CancelResale<'info> {
    pub event: Account<'info, Event>,
    #[account(
        mut,
        seeds = [TICKET_SEED, event.key().as_ref(), &ticket.serial.to_le_bytes()],
        bump = ticket.bump,
        has_one = event,
        has_one = owner @ TicketError::NotOwner,
    )]
    pub ticket: Account<'info, Ticket>,
    pub owner: Signer<'info>,
}

pub fn handler(ctx: Context<CancelResale>) -> Result<()> {
    let ticket = &mut ctx.accounts.ticket;
    ticket.for_sale = false;
    ticket.resale_price = 0;
    msg!("Ticket #{} resale listing cancelled", ticket.serial);
    Ok(())
}
