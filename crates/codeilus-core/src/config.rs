//! Centralized configuration for Codeilus.
//!
//! All magic numbers and tunables live here. Each field can be overridden
//! via an environment variable prefixed with `CODEILUS_`.

/// Application-wide configuration with sensible defaults.
#[derive(Debug, Clone)]
pub struct CodeilusConfig {
    /// HTTP server port.
    pub server_port: u16,
    /// Number of items per DB batch insert.
    pub batch_size: usize,
    /// Seconds before a partial batch is flushed.
    pub batch_flush_secs: u64,
    /// Capacity of the in-memory event bus (tokio broadcast channel).
    pub event_bus_capacity: usize,
    /// Maximum file size (in bytes) accepted for parsing.
    pub max_file_bytes: usize,
    /// Default limit for search result sets.
    pub search_default_limit: usize,
    /// Maximum concurrent LLM requests.
    pub llm_max_concurrent: usize,
    /// Timeout (in seconds) for a single LLM call.
    pub llm_timeout_secs: u64,
    /// Whether to persist every event to the DB.
    pub persist_events: bool,
}

impl Default for CodeilusConfig {
    fn default() -> Self {
        Self {
            server_port: 4174,
            batch_size: 50,
            batch_flush_secs: 2,
            event_bus_capacity: 256,
            max_file_bytes: 20 * 1024 * 1024,
            search_default_limit: 20,
            llm_max_concurrent: 1,
            llm_timeout_secs: 180,
            persist_events: false,
        }
    }
}

impl CodeilusConfig {
    /// Build a config by reading `CODEILUS_*` environment variables,
    /// falling back to [`Default`] for any that are absent or unparseable.
    pub fn from_env() -> Self {
        let d = Self::default();
        Self {
            server_port: env_parse("CODEILUS_SERVER_PORT").unwrap_or(d.server_port),
            batch_size: env_parse("CODEILUS_BATCH_SIZE").unwrap_or(d.batch_size),
            batch_flush_secs: env_parse("CODEILUS_BATCH_FLUSH_SECS").unwrap_or(d.batch_flush_secs),
            event_bus_capacity: env_parse("CODEILUS_EVENT_BUS_CAPACITY")
                .unwrap_or(d.event_bus_capacity),
            max_file_bytes: env_parse("CODEILUS_MAX_FILE_BYTES").unwrap_or(d.max_file_bytes),
            search_default_limit: env_parse("CODEILUS_SEARCH_DEFAULT_LIMIT")
                .unwrap_or(d.search_default_limit),
            llm_max_concurrent: env_parse("CODEILUS_LLM_MAX_CONCURRENT")
                .unwrap_or(d.llm_max_concurrent),
            llm_timeout_secs: env_parse("CODEILUS_LLM_TIMEOUT_SECS")
                .unwrap_or(d.llm_timeout_secs),
            persist_events: env_parse_bool("CODEILUS_PERSIST_EVENTS").unwrap_or(d.persist_events),
        }
    }
}

/// Parse an env var into any `FromStr` type, returning `None` on missing or
/// invalid values.
fn env_parse<T: std::str::FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

/// Parse a boolean env var (`true`, `1`, `yes` → true; everything else → false).
fn env_parse_bool(key: &str) -> Option<bool> {
    std::env::var(key).ok().map(|v| matches!(v.as_str(), "true" | "1" | "yes"))
}
