//! Batch writer: events accumulate in a channel and flush to SQLite in batches.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::events::CodeilusEvent;
use crossbeam_channel::{bounded, select, tick, Receiver, Sender};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};

const BATCH_SIZE: usize = 50;
const FLUSH_INTERVAL: Duration = Duration::from_secs(2);

pub struct BatchWriter {
    sender: Sender<CodeilusEvent>,
    handle: Option<thread::JoinHandle<()>>,
}

impl BatchWriter {
    pub fn spawn(conn: Arc<Mutex<Connection>>) -> Self {
        let (sender, receiver) = bounded::<CodeilusEvent>(1024);
        let handle = thread::spawn(move || {
            writer_loop(conn, receiver);
        });
        Self {
            sender,
            handle: Some(handle),
        }
    }

    pub fn write(&self, event: CodeilusEvent) -> CodeilusResult<()> {
        self.sender.send(event).map_err(|e| {
            CodeilusError::Internal(format!("batch writer channel closed: {}", e))
        })
    }

    pub fn shutdown(mut self) -> CodeilusResult<()> {
        drop(self.sender);
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| CodeilusError::Internal("batch writer thread panicked".into()))?;
        }
        Ok(())
    }
}

fn writer_loop(conn: Arc<Mutex<Connection>>, receiver: Receiver<CodeilusEvent>) {
    let mut buffer: Vec<CodeilusEvent> = Vec::with_capacity(BATCH_SIZE);
    let ticker = tick(FLUSH_INTERVAL);

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(event) => {
                        buffer.push(event);
                        if buffer.len() >= BATCH_SIZE {
                            flush_to_db(&conn, &mut buffer);
                        }
                    }
                    Err(_) => {
                        if !buffer.is_empty() {
                            flush_to_db(&conn, &mut buffer);
                        }
                        info!("batch writer shutting down");
                        return;
                    }
                }
            }
            recv(ticker) -> _ => {
                if !buffer.is_empty() {
                    flush_to_db(&conn, &mut buffer);
                }
            }
        }
    }
}

fn flush_to_db(conn: &Arc<Mutex<Connection>>, buffer: &mut Vec<CodeilusEvent>) {
    let conn = conn.lock().expect("db mutex poisoned");
    let count = buffer.len();

    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(e) => {
            error!(error = %e, "failed to begin transaction");
            return;
        }
    };

    for event in buffer.iter() {
        let event_type = event_type_name(event);
        let data_json = serde_json::to_string(event).unwrap_or_default();

        if let Err(e) = tx.execute(
            "INSERT INTO events (type, data, timestamp) VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![event_type, data_json],
        ) {
            error!(error = %e, event_type = event_type, "failed to insert event");
        }
    }

    if let Err(e) = tx.commit() {
        error!(error = %e, "failed to commit batch");
    } else {
        debug!(count = count, "flushed events to db");
    }
    buffer.clear();
}

fn event_type_name(event: &CodeilusEvent) -> &'static str {
    match event {
        CodeilusEvent::AnalysisStarted { .. } => "AnalysisStarted",
        CodeilusEvent::ParsingProgress { .. } => "ParsingProgress",
        CodeilusEvent::ParsingComplete { .. } => "ParsingComplete",
        CodeilusEvent::GraphBuilding => "GraphBuilding",
        CodeilusEvent::GraphComplete { .. } => "GraphComplete",
        CodeilusEvent::MetricsComputed { .. } => "MetricsComputed",
        CodeilusEvent::DiagramGenerated { .. } => "DiagramGenerated",
        CodeilusEvent::LearningPathGenerated { .. } => "LearningPathGenerated",
        CodeilusEvent::LlmStreamChunk { .. } => "LlmStreamChunk",
        CodeilusEvent::LlmStreamComplete => "LlmStreamComplete",
        CodeilusEvent::NarrativeGenerated { .. } => "NarrativeGenerated",
        CodeilusEvent::NarrativeProgress { .. } => "NarrativeProgress",
        CodeilusEvent::HarvestStarted { .. } => "HarvestStarted",
        CodeilusEvent::HarvestRepoFound { .. } => "HarvestRepoFound",
        CodeilusEvent::HarvestComplete { .. } => "HarvestComplete",
        CodeilusEvent::ExportStarted { .. } => "ExportStarted",
        CodeilusEvent::ExportComplete { .. } => "ExportComplete",
        CodeilusEvent::Error { .. } => "Error",
    }
}
