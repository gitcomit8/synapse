use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct WebSocketManager {
    subscribers: HashMap<String, mpsc::Sender<String>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    pub async fn subscribe(&mut self, channel: &str) -> mpsc::Sender<String> {
        let (tx, _rx) = mpsc::channel(100);
        self.subscribers.insert(channel.to_string(), tx);
        tx
    }

    pub async fn unsubscribe(&mut self, channel: &str) {
        self.subscribers.remove(channel);
    }

    pub async fn broadcast(&self, channel: &str, message: String) {
        if let Some(tx) = self.subscribers.get(channel) {
            let _ = tx.send(message).await;
        }
    }
}
