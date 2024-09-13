use tokio::sync::mpsc::Sender;
use crate::{MinActorError, MinActorResult};
use crate::executor::ActorSysMsg;

/// An ActorRef is a reference to an instance of an actor. It is the main contact point with the
/// running actor.
///
/// An ActorRef is returned from [create_actor()] and is used to send messages to the actor.
///
/// ActorRefs can be cloned as many times as required and can be sent across threads.
// T is message type and U is error type
#[derive(Clone)]
pub struct ActorRef<T, U> where T: Send + Clone, U: Send {
    /// The channel to the actor for sending messages.
    outbox: Sender<ActorSysMsg<T, U>>,
}

impl<T, U> ActorRef<T, U> where T: Send + Clone, U: Send {
    pub(crate) fn new(outbox: Sender<ActorSysMsg<T, U>>) -> Self {
        Self {
            outbox
        }
    }

    /// Send a message to the actor without expecting a response.
    pub async fn send(&self, msg: T) -> MinActorResult<()> {
        self.outbox.send(ActorSysMsg::Send(msg)).await.map_err(|_| MinActorError::UnableToSend)?;
        Ok(())
    }

    /// Send a message to the actor and await a response.
    pub async fn call(&self, msg: T) -> MinActorResult<Result<T, U>> {
        let (send, recv) = tokio::sync::oneshot::channel();
        self.outbox.send(ActorSysMsg::Call(msg, send)).await.map_err(|_| MinActorError::UnableToSend)?;
        let reply = recv.await.map_err(|_| MinActorError::UnableToReceive)?;
        Ok(reply)
    }

    /// Shutdown the actor.
    ///
    /// This is a controlled, orderly shutdown. Previous sends and calls will be processed before the
    /// actor is shut down. Subsequent sends and calls will be ignored, which will have no effect
    /// for sends but will produce an error for outstanding calls.
    pub async fn shutdown(&self) -> MinActorResult<()> {
        self.outbox.send(ActorSysMsg::Shutdown).await.map_err(|_| MinActorError::UnableToSend)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::create_actor;
    use super::*;
    use std::sync::atomic::Ordering;
    use crate::test_code::tests::*;

    /// Test that shutdown will produce an error for calls.
    #[tokio::test]
    async fn test_shutdown_process() {
        let instance = DelayingActor::new();
        let (actor, handle) = create_actor(instance).await.unwrap();
        // send 8 messages, just less than the default buffer size, these will get sent quickly
        for _i in 0..8 {
            let r = actor.send(DelayingMessage::Ping).await;
            assert!(r.is_ok());
        }
        // tell the actor to shutdown, this instruction will get sent quickly
        let r = actor.shutdown().await;
        assert!(r.is_ok());
        // the counter value should still be zero, the actor is still in its sleep for the first message
        let v = COUNTER.load(Ordering::Relaxed);
        assert_eq!(v, 0);
        // send a call, this wont finish until after all the other messages are processed, including
        // the shutdown message. Since the system is then shutdown, this will result in an error.
        let r = actor.call(DelayingMessage::DoPong).await;
        assert!(r.is_err());
        assert_eq!(r, Err(MinActorError::UnableToReceive));
        // although the actor ref struct still exists, it should produce an error when we try to send
        let r = actor.send(DelayingMessage::Ping).await;
        assert!(r.is_err());
        assert_eq!(r, Err(MinActorError::UnableToSend));
        // wait for the actor to finish processing all messages, which should be immediate
        let r = handle.await.unwrap();
        assert!(r.is_ok());
        // the counter value should now be 8, showing that the messages were processed
        // before the actor shut down
        let v = COUNTER.load(Ordering::Relaxed);
        assert_eq!(v, 8);
    }

    /// Test whether we can make arbitary clones of ActorRef
    #[tokio::test]
    async fn test_ref_clone() {
        let instance = SimpleCounter::new();
        let (actor, handle) = create_actor(instance).await.unwrap();
        let act_clone = actor.clone();
        // confirm that both references have 0
        if let CounterMessage::Reply(a) = actor.call(CounterMessage::GetCount).await.unwrap().unwrap() {
            if let CounterMessage::Reply(b) = act_clone.call(CounterMessage::GetCount).await.unwrap().unwrap() {
                assert_eq!(a, b);
                assert_eq!(a, 0);
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
        // increment original
        let r = actor.send(CounterMessage::Count).await;
        assert!(r.is_ok());
        // confirm that both references have 1
        if let CounterMessage::Reply(a) = actor.call(CounterMessage::GetCount).await.unwrap().unwrap() {
            if let CounterMessage::Reply(b) = act_clone.call(CounterMessage::GetCount).await.unwrap().unwrap() {
                assert_eq!(a, b);
                assert_eq!(a, 1);
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
        // shutdown the first ref
        let r = actor.shutdown().await;
        assert!(r.is_ok());
        let r = handle.await;
        assert!(r.is_ok());
        // try send to clone, should get error
        let r = act_clone.send(CounterMessage::Count).await;
        assert!(r.is_err());
    }
}