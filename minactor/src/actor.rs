use core::future::Future;
use std::marker::{Send, Sync};
use log::warn;
use tokio::task::JoinHandle;
use crate::result::Result;
use crate::actor_ref::ActorRef;
use crate::control::Control;
use crate::executor::ActorExecutor;


/// The default size of the actor channel buffer. The channel buffers incoming messages, once it is
/// full then sending threads will wait for space in the buffer.
const DEFAULT_ACTOR_BUFFER_SIZE: usize = 10;

/// The Actor trait. This is the trait that structs will need to implement to function as an actor.
///
/// An actor is an independent computational unit that communicates through messages and maintains
/// it's own private state. In this library, each actor has its own single thread of control, which
/// means that each actor runs independently from anything else and each actor runs in a single
/// threaded fashion meaning that there are no concurrency issues within the actor itself.
///
/// A struct can become an actor by implementing this trait. Instances of the actor are created
/// using the [create_actor()] function. This function returns an [ActorRef] which is used to
/// control the instance and to send it messages. [ActorRef]s can be freely cloned and sent, there
/// can be as many references to an actor instance as required.
///
/// The state of the actor is held in the struct that implements the Actor trait and is maintained
/// by the Actor functions.
///
/// Once created, the actor executor will call the following functions as required:
///
/// * on_initialization() - this will be called immediately after the actor is created
/// * handle_sends() - this is called whenever the actor receives a send message
/// * handle_calls() - this is called whenever the actor receives a call message
/// * on_shutdown() - this is called when the actor is being shut down
///
/// These functions return a [Control] enum, which enables the actor to control its execution,
/// including shutdown and termination. [Control] also enables the integration of tasks
/// from outside the actor framework. See [Control] for more information on the types of actions
/// that can be initiated.
///
/// For more details on these functions, see the individual function documentation.
///
/// ## Integration of Futures
///
/// This framework provides the capability to integrate with futures created outside of the
/// framework.
///
/// ## Shutdown, and Termination
///
/// todo: implement
/// A shutdown is a controlled shutdown of the actor. It completes execution of all messages
/// that were received prior to the stop and then shuts down. Messages that are received after the stop
/// are discarded. In the case of send messages this has no direct effect and in the case of call messages
/// this will result in an error for the calling task. The on_shutdown() function is called.
///
/// todo: implement
/// A termination is an quicker shutdown of the actor. Messages that were sent
/// prior to the termination are discarded. todo() finish defining.
///
/// ## Panics
/// todo: what happens if an actor panics?
///
pub trait Actor {
    /// The type of messages this actor uses for sends.
    ///
    /// The only restrictions on the messages are that they are Send and Sync, so that they can be
    /// passed between threads and ActorRef can be cloned.
    type SendMessage: Send + Sync + Clone;
    /// The type of messages this actor uses for calls. These messages include the call itself and
    /// the result of the call.
    ///
    /// The only restrictions on the messages are that they are Send and Sync, so that they can be
    /// passed between threads and ActorRef can be cloned.
    type CallMessage: Send + Sync + Clone;
    /// The type of messages this actor uses internally.
    ///
    /// These messages can be returned by miscellaneous asynchronous tasks that are performed by the
    /// actor. Examples: the result of reading bytes from a file, the result of opening a file.
    ///
    /// The only restrictions on the messages are that they are Send and Sync, so that they can be
    /// passed between threads and ActorRef can be cloned.
    type InternalMessage: Send + Sync;

    /// The error type that actor functions return.
    ///
    /// Actor functions will return a std::result::Result<_, ErrorType>. The ErrorType must be Send so that it
    /// can be passed between threads.
    type ErrorType: Send + Sync + Clone;

    /// This function is called after the actor has started and before
    /// message processing.
    ///
    /// This function is executed in the context of the actor. It can be overridden to provide
    /// complex initialization capabilities, such as opening a file or opening a network connection.
    /// If not overridden, the default function does nothing.
    ///
    /// Note that messages from clients can be received while this function is being executed. These
    /// messages will be executed directly after this function has completed.
    ///
    /// Implementations can return any of the [Control] instructions. If a [Control::Shutdown] is
    /// returned then the shutdown is queued behind other messages that may have already been received.
    /// The [Control::Shutdown] instruction does not preempt these messages. If a [Control::Terminate]
    /// instruction is returned then this does preempt the processing of other messages.
    fn on_initialization(&mut self) -> impl Future<Output = Control<Self::InternalMessage>> + Send { async {
        Control::Ok
    }}

    /// This function handles messages that are sent, without expecting an answer.
    ///
    /// This will always need to be overridden but a default is included which logs
    /// a warning and returns ().
    #[allow(unused)]        // msg is not used in the default
    fn handle_sends(&mut self, msg: Self::SendMessage) -> impl Future<Output = Control<Self::InternalMessage>> + Send  { async {
        warn!("unhandled sent message received.");
        Control::Ok
    }}

    /// This function handles call messages, which expect an answering message.
    ///
    /// This will always need to be overridden but a default is included which panics.
    #[allow(unused)]        // msg is not used in the default
    fn handle_calls(&mut self, msg: Self::CallMessage) -> impl Future<Output = (Control<Self::InternalMessage>, std::result::Result<Self::CallMessage, Self::ErrorType>)> + Send { async {
        panic!("unhandled call message received.");
    }}

    /// This function is called when a previously registered future is completed.
    ///
    /// Futures can be registered with the actor executor by returning the future in a Control message.
    #[allow(unused)]
    fn handle_future(&mut self, msg: Option<Self::InternalMessage>) -> impl Future<Output = Control<Self::InternalMessage>> + Send { async {
       Control::Ok
    }}

    /// This function is called just prior to shutdown.
    ///
    /// The default implementation does nothing.
    fn on_shutdown(&mut self) -> impl Future<Output = Control<Self::InternalMessage>> + Send { async {
        Control::Ok
    }}
}


/// Create an instance of an actor using default configuration.
pub async fn create_actor<T>(instance: T) -> Result<(ActorRef<T::SendMessage, T::CallMessage, T::ErrorType>, JoinHandle<Result<()>>)>
where
    T: Actor + Send + Sync + 'static
{
    let (outbox, inbox) = tokio::sync::mpsc::channel(DEFAULT_ACTOR_BUFFER_SIZE);
    let a_ref = ActorRef::<T::SendMessage, T::CallMessage, T::ErrorType>::new(outbox);
    let a_clone = a_ref.clone();
    let j = tokio::spawn( async move {
        let mut exec = ActorExecutor::new(instance, inbox, a_clone);
        exec.run().await
    });
    Ok((a_ref, j))
}

