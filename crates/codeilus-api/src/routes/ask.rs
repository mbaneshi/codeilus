//! LLM-powered Q&A endpoint with SSE streaming.

use axum::{
    Router,
    extract::State,
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;

use crate::state::AppState;

#[derive(Deserialize)]
struct AskRequest {
    question: String,
    #[serde(default)]
    context_symbol_ids: Vec<i64>,
}

#[derive(Serialize)]
struct AskChunk {
    #[serde(rename = "type")]
    kind: String,
    content: String,
}

#[derive(Serialize)]
struct LlmStatus {
    available: bool,
}

async fn check_llm() -> Json<LlmStatus> {
    let available = codeilus_llm::is_available().await;
    Json(LlmStatus { available })
}

async fn ask_stream(
    State(state): State<AppState>,
    Json(body): Json<AskRequest>,
) -> Response {
    let available = codeilus_llm::is_available().await;
    if !available {
        return Json(AskChunk {
            kind: "error".into(),
            content: "Claude Code CLI not found. Install it with: npm install -g @anthropic-ai/claude-code".into(),
        }).into_response();
    }

    // Build context from selected symbols
    let mut context_parts = Vec::new();
    if !body.context_symbol_ids.is_empty() {
        let conn = state.db.connection();
        for sid in &body.context_symbol_ids {
            let result: Result<(String, String, String, i64, i64, Option<String>), _> = conn.query_row(
                "SELECT s.name, s.kind, f.path, s.start_line, s.end_line, s.signature
                 FROM symbols s JOIN files f ON s.file_id = f.id WHERE s.id = ?1",
                [sid],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            );
            if let Ok((name, kind, path, start, end, sig)) = result {
                context_parts.push(format!(
                    "- {} `{}` in `{}` (lines {}-{}){}",
                    kind, name, path, start, end,
                    sig.map(|s| format!("\n  Signature: {}", s)).unwrap_or_default()
                ));
            }
        }
    }

    // Also get repo stats for context
    let conn = state.db.connection();
    let file_count: i64 = conn.query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0)).unwrap_or(0);
    let symbol_count: i64 = conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0)).unwrap_or(0);
    let languages: Vec<String> = {
        let mut stmt = conn.prepare("SELECT DISTINCT language FROM files WHERE language IS NOT NULL").unwrap();
        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };
    drop(conn);

    let system_prompt = format!(
        "You are Codeilus, an AI tutor that helps developers understand codebases.\n\
         You are analyzing a codebase with {} files, {} symbols, written in {}.\n\
         Answer concisely and reference specific files/symbols when relevant.\n\
         {}",
        file_count,
        symbol_count,
        languages.join(", "),
        if context_parts.is_empty() { String::new() }
        else { format!("\nThe user has selected these symbols as context:\n{}", context_parts.join("\n")) }
    );

    let request = codeilus_llm::LlmRequest {
        prompt: body.question.clone(),
        system: Some(system_prompt),
        max_tokens: Some(2048),
    };

    info!(question = %body.question, "LLM ask request");

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(32);

    tokio::spawn(async move {
        match codeilus_llm::prompt(request).await {
            Ok(response) => {
                // Send the full response as chunks for SSE
                let _ = tx.send(Ok(Event::default()
                    .event("delta")
                    .data(serde_json::to_string(&AskChunk {
                        kind: "delta".into(),
                        content: response.text,
                    }).unwrap())
                )).await;
                let _ = tx.send(Ok(Event::default()
                    .event("done")
                    .data(serde_json::to_string(&AskChunk {
                        kind: "done".into(),
                        content: format!("{} tokens", response.tokens_used),
                    }).unwrap())
                )).await;
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(serde_json::to_string(&AskChunk {
                        kind: "error".into(),
                        content: format!("LLM error: {}", e),
                    }).unwrap())
                )).await;
            }
        }
    });

    Sse::new(ReceiverStream::new(rx)).into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ask", post(ask_stream))
        .route("/llm/status", get(check_llm))
}
