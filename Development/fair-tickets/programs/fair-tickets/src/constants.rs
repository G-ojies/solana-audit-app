use anchor_lang::prelude::*;

/// PDA seed prefixes.
#[constant]
pub const EVENT_SEED: &[u8] = b"event";

#[constant]
pub const TICKET_SEED: &[u8] = b"ticket";

/// Maximum bytes for a human-readable event name.
pub const MAX_EVENT_NAME_LEN: usize = 64;
