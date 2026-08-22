//! Tokio wakeup bridge used by the native alert notification callback.

use std::sync::Arc;

use tokio::sync::Notify;

/// Thread-safe wakeup signal shared with libtorrent's alert callback.
pub(crate) struct AlertNotifier {
    notify: Arc<Notify>,
}

impl AlertNotifier {
    /// Creates an empty alert notification signal.
    pub(crate) fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    /// Waits until the native alert callback signals the runner.
    pub(crate) async fn notified(&self) {
        self.notify.notified().await;
    }

    /// Wakes one runner waiting for a native alert.
    pub(crate) fn notify(&self) {
        self.notify.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::AlertNotifier;

    #[tokio::test]
    async fn notification_wakes_waiter() {
        let notifier = Arc::new(AlertNotifier::new());
        let waiter_notifier = Arc::clone(&notifier);
        let wait = tokio::spawn(async move {
            waiter_notifier.notified().await;
        });

        tokio::task::yield_now().await;
        // The callback may run on a native thread, so the test exercises the
        // same cross-task notification path used by the runner.
        notifier.notify();
        tokio::time::timeout(std::time::Duration::from_secs(1), wait)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn notification_is_not_lost_before_waiting() {
        let notifier = AlertNotifier::new();
        notifier.notify();

        tokio::time::timeout(std::time::Duration::from_secs(1), notifier.notified())
            .await
            .unwrap();
    }
}
