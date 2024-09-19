//! Contains code that is used by tests.

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    use crate::Actor;
    use crate::control::Control;

    /// an atomic counter that we use for testing
    pub static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Message type for DelayingActor sends
    #[derive(Debug, PartialEq, Clone)]
    pub enum DelayingSends {
        Ping,
    }

    /// Message type for DelayingActor calls
    #[derive(Debug, PartialEq, Clone)]
    pub enum DelayingCalls {
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
        type SendMessage = DelayingSends;
        type CallMessage = DelayingCalls;
        type ErrorType = ();

        async fn handle_sends(&mut self, _msg: Self::SendMessage) -> Control {
            if !self.waited {
                tokio::time::sleep(Duration::new(0, 100)).await;
            }
            // add one to the counter
            let _j = COUNTER.fetch_add(1, Ordering::Relaxed);
            Control::Ok
        }

        async fn handle_calls(&mut self, _msg: Self::CallMessage) -> (Control, Result<Self::CallMessage, Self::ErrorType>) {
            (Control::Ok, Ok(DelayingCalls::Pong))
        }
    }

    /// Message type for SimpleCounter sends
    #[derive(Debug, PartialEq, Clone)]
    pub enum CounterSends {
        Count,
    }

    /// Message type for SimpleCounter calls
    #[derive(Debug, PartialEq, Clone)]
    pub enum CounterCalls {
        GetCount,
        Reply(u64),
    }

    /// Simple actor for testing purposes. It counts.
    pub struct SimpleCounter {
        count: u64,
        immediate_quit: bool,       // if true, then the actor will return Shutdown in on_initialization()
    }

    impl SimpleCounter {
        pub fn new(immediate_quit: bool) -> Self {
            Self {
                count: 0,
                immediate_quit,
            }
        }
    }
    impl Actor for SimpleCounter {
        type SendMessage = CounterSends;
        type CallMessage = CounterCalls;
        type ErrorType = ();

        async fn on_initialization(&mut self) -> Control {
            if self.immediate_quit {
                Control::Shutdown
            } else {
                Control::Ok
            }
        }

        async fn handle_sends(&mut self, _msg: Self::SendMessage) -> Control {
            self.count += 1;
            Control::Ok
        }

        async fn handle_calls(&mut self, _msg: Self::CallMessage) -> (Control, Result<Self::CallMessage, Self::ErrorType>) {
            (Control::Ok, Ok(CounterCalls::Reply(self.count)))
        }
    }
}
