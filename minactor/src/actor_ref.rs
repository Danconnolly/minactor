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
    pub async fn shutdown(&self) -> MinActorResult<()> {
        self.outbox.send(ActorSysMsg::Shutdown).await.expect("couldnt send message");
        Ok(())
    }
}
