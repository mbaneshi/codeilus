//! Broadcast event bus for real-time progress streaming.

use crate::events::CodeilusEvent;
use tokio::sync::broadcast;

/// Broadcast-based event bus. Clone is cheap (Arc internally).
#[derive(Debug, Clone)]
pub struct EventBus {
    tx: broadcast::Sender<CodeilusEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Subscribe to receive events. Lagging receivers will skip missed events.
    pub fn subscribe(&self) -> broadcast::Receiver<CodeilusEvent> {
        self.tx.subscribe()
    }

    /// Publish an event. Silently drops if no subscribers.
    pub fn publish(&self, event: CodeilusEvent) {
        let _ = self.tx.send(event);
    }

    /// Create a sink that can be sent to background tasks.
    pub fn sink(&self) -> EventSink {
        EventSink {
            tx: self.tx.clone(),
        }
    }
}

/// Cheaply cloneable handle for publishing events from background tasks.
#[derive(Debug, Clone)]
pub struct EventSink {
    tx: broadcast::Sender<CodeilusEvent>,
}

impl EventSink {
    pub fn publish(&self, event: CodeilusEvent) {
        let _ = self.tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_and_receive() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        bus.publish(CodeilusEvent::AnalysisStarted {
            path: "/tmp/test".into(),
        });
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, CodeilusEvent::AnalysisStarted { .. }));
    }

    #[tokio::test]
    async fn no_subscribers_ok() {
        let bus = EventBus::new(16);
        // Should not panic even with no subscribers.
        bus.publish(CodeilusEvent::GraphBuilding);
    }

    #[tokio::test]
    async fn sink_publishes() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        let sink = bus.sink();
        sink.publish(CodeilusEvent::GraphBuilding);
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, CodeilusEvent::GraphBuilding));
    }
}
