//! # NT-Protocol Layer
//!
//! Protocol-first unified layer (Buzz NIP-01 + OpenResearch local-first).
//! All desktop operations encoded as NIP-01 events.

pub mod nt_nostr;
pub use nt_nostr::{NeoTrixEvent, EventKind, EventStore, EventTag};