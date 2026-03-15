//! Shared application state.

use codeilus_core::EventBus;
use codeilus_db::DbPool;
use codeilus_llm::LlmProvider;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
    pub event_bus: Arc<EventBus>,
    pub repo_root: Option<PathBuf>,
    pub llm: Arc<dyn LlmProvider>,
}

impl AppState {
    pub fn new(db: Arc<DbPool>, event_bus: Arc<EventBus>, llm: Arc<dyn LlmProvider>) -> Self {
        Self { db, event_bus, repo_root: None, llm }
    }

    pub fn with_repo_root(mut self, path: PathBuf) -> Self {
        self.repo_root = Some(path);
        self
    }
}
