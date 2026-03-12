//! Shared application state.

use codeilus_core::EventBus;
use codeilus_db::DbPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
    pub event_bus: Arc<EventBus>,
}

impl AppState {
    pub fn new(db: Arc<DbPool>, event_bus: Arc<EventBus>) -> Self {
        Self { db, event_bus }
    }
}
