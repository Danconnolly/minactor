//! Contains code that is used by tests.

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    use crate::Actor;


    /// an atomic counter that we use for testing
    pub static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Message type for DelayingActor
    #[derive(Debug, PartialEq, Clone)]
    pub enum DelayingMessage {
        Ping,
        DoPong,
        Pong,
    }

    /// Simple actor for testing purposes. It delays on the first message received.
    pub struct DelayingActor {
        waited: bool,           // has it already waited?
    }

    impl DelayingActor {
        pub fn new() -> Self {
            Self {
                waited: false
            }
        }
    }

    impl Actor for DelayingActor {
        type MessageType = DelayingMessage;
        type ErrorType = ();

        async fn handle_sends(&mut self, _msg: Self::MessageType) -> Result<(), Self::ErrorType> {
            if !self.waited {
                tokio::time::sleep(Duration::new(0, 100)).await;
            }
            // add one to the counter
            let _j = COUNTER.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        async fn handle_calls(&mut self, _msg: Self::MessageType) -> Result<Self::MessageType, Self::ErrorType> {
            Ok(DelayingMessage::Pong)
        }
    }

    /// Message type for SimpleCounter
    #[derive(Debug, PartialEq, Clone)]
    pub enum CounterMessage {
        Count,
        GetCount,
        Reply(u64),
    }

    /// Simple actor for testing purposes. It counts.
    pub struct SimpleCounter {
        count: u64,
    }

    impl SimpleCounter {
        pub fn new() -> Self {
            Self {
                count: 0,
            }
        }
    }
    impl Actor for SimpleCounter {
        type MessageType = CounterMessage;
        type ErrorType = ();

        async fn handle_sends(&mut self, _msg: Self::MessageType) -> Result<(), Self::ErrorType> {
            self.count += 1;
            Ok(())
        }

        async fn handle_calls(&mut self, _msg: Self::MessageType) -> Result<Self::MessageType, Self::ErrorType> {
            Ok(CounterMessage::Reply(self.count))
        }
    }
}
