//! Codeilus core types: IDs, errors, events, and event bus.
//!
//! This crate is the contract for all other crates. Do not duplicate these types elsewhere.

pub mod config;
pub mod error;
pub mod event_bus;
pub mod events;
pub mod ids;
pub mod types;

pub use config::CodeilusConfig;
pub use error::{CodeilusError, CodeilusResult};
pub use event_bus::{EventBus, EventSink};
pub use events::CodeilusEvent;
pub use ids::{ChapterId, CommunityId, EdgeId, FileId, SymbolId};
pub use types::{Confidence, EdgeKind, Language, NarrativeKind, SymbolKind};
