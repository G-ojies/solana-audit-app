pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Ggb46RAz1cWos2fQp4eWorFPpjPWouTBzccyCDVMGcgU");

/// fair-tickets — event ticketing with an on-chain fair-resale guarantee.
///
/// An everyday system (buying and reselling event tickets) rebuilt as a Solana
/// program. The core friction it removes is scalping: the organizer sets a
/// resale price ceiling that the program itself enforces, so no secondary buyer
/// can be gouged and no platform has to be trusted to police it.
#[program]
pub mod fair_tickets {
    use super::*;

    /// Organizer creates an event with a face price, resale cap, and supply.
    pub fn create_event(
        ctx: Context<CreateEvent>,
        event_id: u64,
        name: String,
        price: u64,
        max_resale_price: u64,
        supply: u32,
    ) -> Result<()> {
        instructions::create_event::handler(ctx, event_id, name, price, max_resale_price, supply)
    }

    /// Primary sale: buy a ticket at face price from the organizer.
    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        instructions::buy_ticket::handler(ctx)
    }

    /// List an owned ticket for resale (price must be <= the event cap).
    pub fn list_resale(ctx: Context<ListResale>, price: u64) -> Result<()> {
        instructions::list_resale::handler(ctx, price)
    }

    /// Cancel an active resale listing.
    pub fn cancel_resale(ctx: Context<CancelResale>) -> Result<()> {
        instructions::cancel_resale::handler(ctx)
    }

    /// Secondary sale: buy a listed ticket peer-to-peer from its owner.
    pub fn buy_resale(ctx: Context<BuyResale>) -> Result<()> {
        instructions::buy_resale::handler(ctx)
    }

    /// Organizer checks a ticket in at the door (disables further resale).
    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        instructions::redeem::handler(ctx)
    }
}
