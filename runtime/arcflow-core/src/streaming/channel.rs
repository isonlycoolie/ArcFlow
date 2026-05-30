//! Per-run stream channel (Phase 2.1).

use tokio::sync::mpsc;

use super::events::StreamEvent;

const DEFAULT_CAPACITY: usize = 256;

/// Sender half of a per-run stream channel.
#[derive(Clone, Debug)]
pub struct StreamChannelSender {
    tx: mpsc::Sender<StreamEvent>,
}

impl StreamChannelSender {
    /// Non-blocking send; drops the event when the buffer is full.
    pub fn try_send(&self, event: StreamEvent) {
        let _ = self.tx.try_send(event);
    }
}

/// Creates a bounded sender/receiver pair for one workflow run.
pub fn stream_pair(capacity: usize) -> (StreamChannelSender, mpsc::Receiver<StreamEvent>) {
    let (tx, rx) = mpsc::channel(capacity.max(1));
    (StreamChannelSender { tx }, rx)
}

/// Creates a pair with the default buffer size.
pub fn default_stream_pair() -> (StreamChannelSender, mpsc::Receiver<StreamEvent>) {
    stream_pair(DEFAULT_CAPACITY)
}
