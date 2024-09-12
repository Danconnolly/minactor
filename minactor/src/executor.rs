use log::warn;
use tokio::sync::mpsc::Receiver;
use crate::{Actor, MinActorResult};

/// The ActorExecutor executes the actor, receiving messages and forwarding them to handlers.
pub(crate) struct ActorExecutor<T> where T: Actor + Send {
    instance: T,
    inbox: Receiver<ActorSysMsg<T::MessageType, T::ErrorType>>,
}

impl<T> ActorExecutor<T> where T: Actor + Send + Sync {
    /// Create a new instance of the executor.
    pub(crate) fn new(instance: T, inbox: Receiver<ActorSysMsg<T::MessageType, T::ErrorType>>) -> Self {
        ActorExecutor {
            instance, inbox
        }
    }

    /// Executor run loop.
    pub(crate) async fn run(&mut self) -> Result<(), T::ErrorType> {
        use ActorSysMsg::*;
        match self.instance.on_initialization().await {
            Err(e) => { return Err(e); },
            Ok(()) => {}
        }
        while let Some(sys_msg) = self.inbox.recv().await {
            match sys_msg {
                Shutdown => {
                    break;
                },
                Send(msg) => {
                    match self.instance.handle_sends(msg).await {
                        Ok(()) => {},
                        Err(_) => {
                            warn!("actor returned error during handling a send msg.");
                        }
                    }
                },
                Call(msg, dest) => {
                    let r = self.instance.handle_calls(msg).await;
                    match dest.send(r) {
                        Ok(()) => {},
                        Err(_) => {
                            warn!("unable to send reply of call message to caller.");
                        }
                    }
                },
            }
        }
        Ok(())
    }
}


/// The ActorSysMsg is what actually gets sent.
pub(crate) enum ActorSysMsg<U, E> where U: Send, E: Send {
    /// Normal shutdown
    Shutdown,
    /// A send message
    Send(U),
    /// A call message
    Call(U, tokio::sync::oneshot::Sender<Result<U, E>>),
}
