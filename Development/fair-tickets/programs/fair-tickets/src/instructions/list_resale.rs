use anchor_lang::prelude::*;

use crate::constants::TICKET_SEED;
use crate::error::TicketError;
use crate::state::{Event, Ticket};

/// Owner lists their ticket for resale. The price is checked against the
/// event's `max_resale_price` — the program refuses to list a scalped ticket.
#[derive(Accounts)]
pub struct ListResale<'info> {
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

pub fn handler(ctx: Context<ListResale>, price: u64) -> Result<()> {
    require!(!ctx.accounts.ticket.redeemed, TicketError::AlreadyRedeemed);
    require!(
        price <= ctx.accounts.event.max_resale_price,
        TicketError::ResaleAboveCap
    );

    let ticket = &mut ctx.accounts.ticket;
    ticket.for_sale = true;
    ticket.resale_price = price;
    msg!("Ticket #{} listed for resale at {} lamports", ticket.serial, price);
    Ok(())
}
