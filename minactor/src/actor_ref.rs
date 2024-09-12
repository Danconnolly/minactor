use tokio::sync::mpsc::Sender;
use crate::{Actor, MinActorResult};
use crate::executor::ActorSysMsg;

/// An ActorRef is a reference to an instance of an actor.
pub struct ActorRef<T>
where T: Actor  {
    /// The channel to the actor for sending messages.
    outbox: Sender<ActorSysMsg<T::MessageType, T::ErrorType>>,
}

impl<T> ActorRef<T> where T: Actor {
    pub(crate) fn new(outbox: Sender<ActorSysMsg<T::MessageType, T::ErrorType>>) -> Self {
        Self {
            outbox
        }
    }

    /// Send a message to the actor without expecting a response.
    pub async fn send(&self, msg: T::MessageType) -> MinActorResult<()> {
        self.outbox.send(ActorSysMsg::Send(msg)).await.expect("couldnt send message");
        Ok(())
    }

    /// Send a message to the actor and await a response.
    pub async fn call(&self, msg: T::MessageType) -> MinActorResult<Result<T::MessageType, T::ErrorType>> {
        let (send, recv) = tokio::sync::oneshot::channel();
        self.outbox.send(ActorSysMsg::Call(msg, send)).await.expect("couldnt send message");
        let reply = recv.await.expect("couldnt receive message");
        Ok(reply)
    }

    /// Shutdown the actor.
    ///
    /// This is a controlled, orderly shutdown. Previous sends and calls will be processed before the
    /// actor is shut down. Subsequent sends and calls will be ignored, which will have no effect
    /// on sends but will produce an error for calls.
    pub async fn shutdown(&self) -> MinActorResult<()> {
        self.outbox.send(ActorSysMsg::Shutdown).await.expect("couldnt send message");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use crate::create_actor;
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use async_trait::async_trait;

    /// an atomic counter that we use for testing
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Message type for DelayingActor
    enum DelayingMessage {
        Ping,
    }

    /// Simple actor for testing purposes. It delays on the first message received.
    struct DelayingActor {
        waited: bool,           // has it already waited?
    }

    #[async_trait]
    impl Actor for DelayingActor {
        type MessageType = DelayingMessage;
        type CreationArguments = ();
        type ErrorType = ();

        fn new(args: Self::CreationArguments) -> Self {
            Self { waited: false }
        }

        async fn handle_sends(&mut self, _msg: Self::MessageType) -> Result<(), Self::ErrorType> {
            if ! self.waited {
                tokio::time::sleep(Duration::new(0, 100)).await;
            }
            // add one to the counter
            let _j = COUNTER.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    /// Test that shutdown will produce an error for calls.
    #[tokio::test]
    async fn test_shutdown_process() {
        let (actor, handle) = create_actor::<DelayingActor>(()).await.unwrap();
        // send 8 messages, just less than the default buffer size, these will get sent quickly
        for i in 0..8 {
            let r = actor.send(DelayingMessage::Ping).await;
            assert!(r.is_ok());
        }
        // tell the actor to shutdown, this instruction will get sent quickly
        let r = actor.shutdown().await;
        assert!(r.is_ok());
        // the counter value should still be zero, the actor is still in its sleep for the first message
        let v = COUNTER.load(Ordering::Relaxed);
        assert_eq!(v, 0);
        // wait for the actor to finish processing all messages
        let r = handle.await.unwrap();
        assert!(r.is_ok());
        // the counter value should now be 8, showing that the messages were processed
        // before the actor shut down
        let v = COUNTER.load(Ordering::Relaxed);
        assert_eq!(v, 8);
    }
}