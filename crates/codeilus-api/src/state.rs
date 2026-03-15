//! Shared application state.

use codeilus_core::{CodeilusConfig, EventBus};
use codeilus_db::DbPool;
use codeilus_llm::LlmProvider;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
    pub event_bus: Arc<EventBus>,
    pub repo_root: Option<PathBuf>,
    pub llm: Arc<dyn LlmProvider>,
    pub config: Arc<CodeilusConfig>,
    pub llm_semaphore: Arc<Semaphore>,
}

impl AppState {
    pub fn new(
        db: Arc<DbPool>,
        event_bus: Arc<EventBus>,
        llm: Arc<dyn LlmProvider>,
        config: Arc<CodeilusConfig>,
    ) -> Self {
        let llm_permits = config.llm_max_concurrent;
        Self {
            db, event_bus, repo_root: None, llm,
            llm_semaphore: Arc::new(Semaphore::new(llm_permits)),
            config,
        }
    }

    pub fn with_repo_root(mut self, path: PathBuf) -> Self {
        self.repo_root = Some(path);
        self
    }
}
