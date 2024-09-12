use tokio::sync::mpsc::Receiver;
use crate::Actor;

/// The ActorExecutor executes the actor, receiving messages and forwarding them to handlers.
pub(crate) struct ActorExecutor<T> where T: Actor + Send {
    instance: T,
    inbox: Receiver<ActorSysMsg<T::MessageType>>,
}

impl<T> ActorExecutor<T> where T: Actor + Send + Sync {
    /// Create a new instance of the executor.
    pub(crate) fn new(instance: T, inbox: Receiver<ActorSysMsg<T::MessageType>>) -> Self {
        ActorExecutor {
            instance, inbox
        }
    }

    /// Executor run loop.
    pub(crate) async fn run(&mut self) -> () {
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
pub(crate) enum ActorSysMsg<U> where U: Send {
    /// Normal shutdown
    Shutdown,
    /// A send message
    Send(U),
    /// A call message
    Call(U, tokio::sync::oneshot::Sender<U>),
}
