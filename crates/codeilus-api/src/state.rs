//! Shared application state.

use codeilus_core::{CodeilusConfig, EventBus};
use codeilus_db::DbPool;
use codeilus_llm::LlmProvider;
use moka::sync::Cache;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// Shared in-memory cache for expensive API responses.
pub struct AppCache {
    pub json: Cache<String, serde_json::Value>,
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            json: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(600)) // 10 min TTL
                .build(),
        }
    }

    pub fn invalidate_all(&self) {
        self.json.invalidate_all();
    }
}

impl Clone for AppCache {
    fn clone(&self) -> Self {
        Self {
            json: self.json.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
    pub event_bus: Arc<EventBus>,
    pub repo_root: Option<PathBuf>,
    pub llm: Arc<dyn LlmProvider>,
    pub config: Arc<CodeilusConfig>,
    pub llm_semaphore: Arc<Semaphore>,
    pub cache: AppCache,
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
            cache: AppCache::new(),
        }
    }

    pub fn with_repo_root(mut self, path: PathBuf) -> Self {
        self.repo_root = Some(path);
        self
    }
}
