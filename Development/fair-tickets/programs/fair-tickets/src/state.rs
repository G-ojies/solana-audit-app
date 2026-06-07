use anchor_lang::prelude::*;

use crate::constants::MAX_EVENT_NAME_LEN;

/// An event created by an organizer. Web2 analogue: a row in an events table on
/// a ticketing platform's private database. Here the rules (price, supply, and
/// crucially the resale ceiling) live on-chain and bind everyone equally.
#[account]
#[derive(InitSpace)]
pub struct Event {
    pub organizer: Pubkey,
    pub event_id: u64,
    #[max_len(MAX_EVENT_NAME_LEN)]
    pub name: String,
    /// Primary sale price, in lamports.
    pub price: u64,
    /// The maximum lamports a ticket may be resold for. This is the anti-scalp
    /// guarantee — enforced by the program, not by a platform's goodwill.
    pub max_resale_price: u64,
    pub supply: u32,
    pub sold: u32,
    pub bump: u8,
}

/// A single ticket. Web2 analogue: a barcode/row owned by the platform that you
/// merely have a claim to. Here the ticket is an on-chain account whose `owner`
/// field is the source of truth; transfers are atomic with payment.
#[account]
#[derive(InitSpace)]
pub struct Ticket {
    pub event: Pubkey,
    pub serial: u32,
    pub owner: Pubkey,
    /// Lamports actually paid by the current owner (primary or resale).
    pub paid: u64,
    pub for_sale: bool,
    pub resale_price: u64,
    /// True once checked in at the door; a redeemed ticket can't be resold.
    pub redeemed: bool,
    pub bump: u8,
}
