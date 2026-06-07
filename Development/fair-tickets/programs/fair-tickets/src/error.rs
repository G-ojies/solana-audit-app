use anchor_lang::prelude::*;

#[error_code]
pub enum TicketError {
    #[msg("Event is sold out")]
    SoldOut,
    #[msg("Resale price exceeds the organizer's fair-resale cap")]
    ResaleAboveCap,
    #[msg("Ticket is not listed for resale")]
    NotForSale,
    #[msg("Only the ticket owner may perform this action")]
    NotOwner,
    #[msg("Ticket has already been redeemed (checked in)")]
    AlreadyRedeemed,
    #[msg("Buyer already owns this ticket")]
    AlreadyOwner,
    #[msg("Event name exceeds the maximum length")]
    NameTooLong,
}
