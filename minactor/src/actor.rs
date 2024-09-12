use std::fmt::Debug;
use async_trait::async_trait;
use log::warn;
use tokio::task::JoinHandle;
use crate::{MinActorError, MinActorResult};
use crate::actor_ref::ActorRef;
use crate::executor::ActorExecutor;

/// todo: replace this with configuration
const ACTOR_BUFFER_SIZE: usize = 100;

///
#[async_trait]
pub trait Actor {
    /// todo: the functions should return a generic result type, not MinResult

    /// The type of messages this actor uses.
    ///
    /// The only restrictions on the messages are that they are Send, so that they can be
    /// passed between tokio threads.
    type MessageType: Send;
    /// The type of the arguments that are used to create the actor.
    ///
    /// The actor struct is created by the create_actor() functions using the new() function
    /// that is implemented by the actor.
    type CreationArguments;

    /// The new() function must be defined, it is used to create a new instance of the Actor.
    ///
    /// This is a non-async function and must always return an instance. It is executed in the
    /// context that calls the create_actor() function.
    ///
    /// This is not the place to do any complex initialization, it is just intended to set initial
    /// values for the actor struct.
    fn new(args: Self::CreationArguments) -> Self;

    /// The on_initialization() function is called immediately after the actor has started.
    ///
    /// If not overridden, this function does nothing
    async fn on_initialization(&self) -> MinActorResult<()> {
        Ok(())
    }

    /// This function handles messages that are sent, without expecting an answer.
    ///
    /// This will always need to be overridden but a default is included which logs
    /// a warning and returns a HandlerNotImplemented error.
    async fn handle_sends(&mut self, msg: Self::MessageType) -> MinActorResult<()> {
        warn!("unhandled sent message received: {:?}", msg);
        Err(MinActorError::HandlerNotImplemented)
    }

    /// This function handles call messages, which expect an answering message.
    ///
    /// This will always need to be overridden but a default is included which logs
    /// a warning and returns a HandlerNotImplemented error.
    async fn handle_calls(&mut self, msg: Self::MessageType) -> MinActorResult<Self::MessageType> {
        warn!("unhandled call message received: {:?}", msg);
        Err(MinActorError::HandlerNotImplemented)
    }
}

/// Instantiate an instance of an actor.
pub async fn create_actor<T>(args: T::CreationArguments) -> MinActorResult<(ActorRef<T>, JoinHandle<()>)>
    where T: Actor + Send + Sync + 'static {
    let instance = T::new(args);
    let (outbox, inbox) = tokio::sync::mpsc::channel(ACTOR_BUFFER_SIZE);
    let j = tokio::spawn( async move {
        let mut exec = ActorExecutor::new(instance, inbox);
        exec.run().await
    });
    Ok((ActorRef::<T> {
        outbox
    }, j))
}

