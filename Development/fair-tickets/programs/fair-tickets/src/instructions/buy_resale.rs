use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::constants::TICKET_SEED;
use crate::error::TicketError;
use crate::state::{Event, Ticket};

/// Secondary sale: a buyer purchases a listed ticket directly from its owner.
/// Payment goes peer-to-peer to the seller; ownership flips atomically. Because
/// the listing price was capped at list time, the buyer is structurally
/// protected from scalping.
#[derive(Accounts)]
pub struct BuyResale<'info> {
    pub event: Account<'info, Event>,
    #[account(
        mut,
        seeds = [TICKET_SEED, event.key().as_ref(), &ticket.serial.to_le_bytes()],
        bump = ticket.bump,
        has_one = event,
        constraint = ticket.owner == seller.key() @ TicketError::NotOwner,
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// Current owner; receives the resale payment.
    #[account(mut)]
    pub seller: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyResale>) -> Result<()> {
    let ticket = &ctx.accounts.ticket;
    require!(ticket.for_sale, TicketError::NotForSale);
    require!(!ticket.redeemed, TicketError::AlreadyRedeemed);
    require!(
        ctx.accounts.buyer.key() != ticket.owner,
        TicketError::AlreadyOwner
    );

    let price = ticket.resale_price;
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            },
        ),
        price,
    )?;

    let ticket = &mut ctx.accounts.ticket;
    ticket.owner = ctx.accounts.buyer.key();
    ticket.paid = price;
    ticket.for_sale = false;
    ticket.resale_price = 0;
    msg!("Ticket #{} resold to {} for {} lamports", ticket.serial, ticket.owner, price);
    Ok(())
}
