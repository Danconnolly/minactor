use core::future::Future;
use log::warn;
use tokio::task::JoinHandle;
use crate::MinActorResult;
use crate::actor_ref::ActorRef;
use crate::executor::ActorExecutor;


/// The default size of the actor channel buffer. The channel buffers incoming messages, once it is
/// full then sending threads will wait for space in the buffer.
const DEFAULT_ACTOR_BUFFER_SIZE: usize = 10;

///
pub trait Actor {
    /// The type of messages this actor uses.
    ///
    /// The only restrictions on the messages are that they are Send and Clone, so that they can be
    /// passed between threads and ActorRef can be cloned.
    type MessageType: Send + Clone;

    /// The type of the arguments that are used to create the actor.
    ///
    /// The actor struct is created by the create_actor() functions using the new() function
    /// that is implemented by the actor.
    type CreationArguments;

    /// The error type that actor functions return.
    ///
    /// Actor functions will return a Result<_, ErrorType>. The ErrorType must be Send so that it
    /// can be passed between threads.
    type ErrorType: Send;

    /// The new() function must be defined, it is used to create a new instance of the Actor.
    ///
    /// This is a non-async function and must always return an instance. It is executed in the
    /// context that calls the create_actor() function.
    ///
    /// This is not the place to do any complex initialization, it is just intended to set initial
    /// values for the actor struct.
    fn new(args: Self::CreationArguments) -> Self;

    /// The on_initialization() function is called after the actor has started and before
    /// message processing.
    ///
    /// This function can be overridden to provide complex initialization capabilities, such as
    /// opening a file or opening a network connection.
    ///
    /// If not overridden, this function does nothing.
    ///
    /// If the function returns an error, the actor terminates.
    fn on_initialization(&self) -> impl Future<Output = Result<(), Self::ErrorType>> + Send { async {
        Ok(())
    }}

    /// This function handles messages that are sent, without expecting an answer.
    ///
    /// This will always need to be overridden but a default is included which logs
    /// a warning and returns ().
    #[allow(unused)]        // msg is not used in the default
    fn handle_sends(&mut self, msg: Self::MessageType) -> impl Future<Output = Result<(), Self::ErrorType>> + Send  { async {
        warn!("unhandled sent message received.");
        Ok(())
    }}

    /// This function handles call messages, which expect an answering message.
    ///
    /// This will always need to be overridden but a default is included which panics.
    #[allow(unused)]        // msg is not used in the default
    fn handle_calls(&mut self, msg: Self::MessageType) -> impl Future<Output = Result<Self::MessageType, Self::ErrorType>> + Send { async {
        panic!("unhandled call message received.");
    }}
}


/// Instantiate an instance of an actor using default configuration.
pub async fn create_actor<T>(args: T::CreationArguments) -> MinActorResult<(ActorRef<T::MessageType, T::ErrorType>, JoinHandle<Result<(), T::ErrorType>>)>
where
    T: Actor + Send + Sync + 'static
{
    let instance = T::new(args);
    let (outbox, inbox) = tokio::sync::mpsc::channel(DEFAULT_ACTOR_BUFFER_SIZE);
    let j = tokio::spawn( async move {
        let mut exec = ActorExecutor::new(instance, inbox);
        exec.run().await
    });
    Ok((ActorRef::<T::MessageType, T::ErrorType>::new(outbox), j))
}

