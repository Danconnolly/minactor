use log::warn;
use tokio::sync::mpsc::Receiver;
use crate::Actor;
use crate::control::Control;

/// The ActorExecutor executes the actor, receiving messages and forwarding them to handlers.
pub(crate) struct ActorExecutor<T> where T: Actor + Send {
    instance: T,
    inbox: Receiver<ActorSysMsg<T::SendMessage, T::CallMessage, T::ErrorType>>,
}

impl<T> ActorExecutor<T>
where
    T: Actor + Send + Sync
{
    /// Create a new instance of the executor.
    pub(crate) fn new(instance: T, inbox: Receiver<ActorSysMsg<T::SendMessage, T::CallMessage, T::ErrorType>>) -> Self {
        ActorExecutor {
            instance, inbox
        }
    }

    /// Executor run loop.
    pub(crate) async fn run(&mut self) -> Result<(), T::ErrorType> {
        use ActorSysMsg::*;
        match self.instance.on_initialization().await {
            Control::Ok => {},
            _ => { todo!(); },
        }
        while let Some(sys_msg) = self.inbox.recv().await {
            match sys_msg {
                Shutdown => {
                    break;
                },
                Send(msg) => {
                    match self.instance.handle_sends(msg).await {
                        Control::Ok => {},
                        _ => { todo!(); }
                    }
                },
                Call(msg, dest) => {
                    let (control, result) = self.instance.handle_calls(msg).await;
                    match dest.send(result) {
                        Ok(()) => {},
                        Err(_) => {
                            warn!("unable to send reply of call message to caller.");
                        }
                    }
                    // todo: handle control message
                },
            }
        }
        Ok(())
    }
}


/// The ActorSysMsg is what actually gets sent.
pub(crate) enum ActorSysMsg<S, C, E>
where S: Send, C: Send, E: Send {
    /// Normal shutdown
    Shutdown,
    /// A send message
    Send(S),
    /// A call message
    Call(C, tokio::sync::oneshot::Sender<Result<C, E>>),
}
