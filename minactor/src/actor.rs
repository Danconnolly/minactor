use std::fmt::Debug;
use async_trait::async_trait;
use log::warn;
use tokio::task::JoinHandle;
use crate::{MinActorError, MinActorResult};


/// todo: replace this with configuration
const ACTOR_BUFFER_SIZE: usize = 100;


/// An ActorRef is a reference to an instance of an actor.
pub struct ActorRef<T>
    where T: Actor  {
    /// The channel to the actor for sending messages.
    outbox: tokio::sync::mpsc::Sender<ActorSysMsg<T::MessageType>>,
}

impl<T> ActorRef<T> where T: Actor {
    /// Send a message to the actor without expecting a response.
    pub async fn send(&self, msg: T::MessageType) -> MinActorResult<()> {
        self.outbox.send(ActorSysMsg::Send(msg)).await.expect("couldnt send message");
        Ok(())
    }

    /// Send a message to the actor and await a response.
    pub async fn call(&self, msg: T::MessageType) -> MinActorResult<T::MessageType> {
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

///
#[async_trait]
pub trait Actor {
    /// todo: the functions should return a generic result type, not MinResult

    /// The type of messages this actor uses.
    type MessageType: Send + Debug;
    /// The type of the arguments that are used to initialize the actor.
    type Arguments;

    /// The new() function must be defined, it is used to create a new instance of the Actor.
    ///
    /// This is a non-async function and must always return an instance. This is not the place to
    /// do any complex initialization, it is just intended to set initial values for the struct that
    /// implements the actor.
    fn new(args: Self::Arguments) -> Self;

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
pub async fn create_actor<T>(args: T::Arguments) -> MinActorResult<(ActorRef<T>, JoinHandle<()>)>
    where T: Actor + Send + Sync + 'static {
    let instance = T::new(args);
    let (outbox, inbox) = tokio::sync::mpsc::channel(ACTOR_BUFFER_SIZE);
    let j = tokio::spawn( async move {
        let mut exec = ActorExecutor {
            instance, inbox
        };
        exec.run().await
    });
    Ok((ActorRef::<T> {
        outbox
    }, j))
}

/// The ActorExecutor executes the actor, receiving messages and forwarding them to handlers.
struct ActorExecutor<T> where T: Actor + Send {
    instance: T,
    inbox: tokio::sync::mpsc::Receiver<ActorSysMsg<T::MessageType>>,
}

impl<T> ActorExecutor<T> where T: Actor + Send + Sync {
    /// Executor run loop.
    async fn run(&mut self) -> () {
        use ActorSysMsg::*;
        while let Some(sys_msg) = self.inbox.recv().await {
            match sys_msg {
                Shutdown => {
                    break;
                },
                Send(msg) => {
                    self.instance.handle_sends(msg).await.expect("message handling failed");
                },
                Call(msg, dest) => {
                    let r = self.instance.handle_calls(msg).await.unwrap();
                    dest.send(r).expect("unable to send message");
                },
            }
        }
    }
}


/// The ActorSysMsg is what actually gets sent.
enum ActorSysMsg<U> where U: Send {
    /// Normal shutdown
    Shutdown,
    /// A send message
    Send(U),
    /// A call message
    Call(U, tokio::sync::oneshot::Sender<U>),
}
