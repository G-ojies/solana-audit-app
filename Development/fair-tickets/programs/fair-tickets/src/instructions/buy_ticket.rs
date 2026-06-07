use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::constants::{EVENT_SEED, TICKET_SEED};
use crate::error::TicketError;
use crate::state::{Event, Ticket};

/// Primary sale: buyer pays the organizer the face price and a new ticket
/// account is minted with the next serial number.
#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(
        mut,
        seeds = [EVENT_SEED, event.organizer.as_ref(), &event.event_id.to_le_bytes()],
        bump = event.bump,
        has_one = organizer,
    )]
    pub event: Account<'info, Event>,
    #[account(
        init,
        payer = buyer,
        space = 8 + Ticket::INIT_SPACE,
        seeds = [TICKET_SEED, event.key().as_ref(), &event.sold.to_le_bytes()],
        bump
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// Receives the face-price payment; pinned to the event's organizer.
    #[account(mut)]
    pub organizer: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyTicket>) -> Result<()> {
    let event = &mut ctx.accounts.event;
    require!(event.sold < event.supply, TicketError::SoldOut);
    let serial = event.sold;

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.organizer.to_account_info(),
            },
        ),
        event.price,
    )?;

    let ticket = &mut ctx.accounts.ticket;
    ticket.event = event.key();
    ticket.serial = serial;
    ticket.owner = ctx.accounts.buyer.key();
    ticket.paid = event.price;
    ticket.for_sale = false;
    ticket.resale_price = 0;
    ticket.redeemed = false;
    ticket.bump = ctx.bumps.ticket;

    event.sold = event.sold.saturating_add(1);
    msg!("Ticket #{} sold to {}", serial, ticket.owner);
    Ok(())
}
