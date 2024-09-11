use async_trait::async_trait;
use tokio::task::JoinHandle;
use crate::MinActorResult;


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
        todo!()
    }
}

///
#[async_trait]
pub trait Actor {
    /// The type of messages this actor uses.
    type MessageType: Send;
    /// The type of the arguments that are used to initialize the actor.
    type Arguments;

    /// The new() function must be defined, it is used to create a new instance of the Actor.
    ///
    /// This is a non-async function and must always return an instance. This is not the place to
    /// do any complex initialization, it is just intended to set initial values for the actor
    /// implementing struct.
    fn new(args: Self::Arguments) -> Self;

    /// The on_initialization() function is called immediately after the actor has started.
    ///
    /// If not overridden, this function does nothing
    async fn on_initialization(&self) -> MinActorResult<()> {
        Ok(())
    }
}

/// Instantiate an instance of an actor.
pub async fn create_actor<T>(args: T::Arguments) -> MinActorResult<(ActorRef<T>, JoinHandle<()>)>
    where T: Actor + Send + 'static {
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

impl<T> ActorExecutor<T> where T: Actor + Send {
    /// Executor run loop.
    async fn run(&mut self) -> () {
        while let Some(sys_msg) = self.inbox.recv().await {
            println!("message received");
        }
    }
}


/// The ActorSysMsg is what actually gets sent.
enum ActorSysMsg<T> {
    /// Normal shutdown
    Shutdown,
    /// A send message
    Send(T),
    /// A call message
    Call(T, tokio::sync::oneshot::Sender<T>),
}
