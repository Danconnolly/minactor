use log::warn;
use tokio::sync::mpsc::Receiver;
use crate::{Actor, ActorRef};
use crate::control::Control;

/// The ActorExecutor executes the actor, receiving messages and forwarding them to handlers.
pub(crate) struct ActorExecutor<T>
where T: Actor + Send {
    instance: T,
    inbox: Receiver<ActorSysMsg<T::SendMessage, T::CallMessage, T::ErrorType>>,
    actor_ref: ActorRef<T::SendMessage, T::CallMessage, T::ErrorType>,
}

impl<T> ActorExecutor<T>
where
    T: Actor + Send + Sync
{
    /// Create a new instance of the executor.
    pub(crate) fn new(instance: T, inbox: Receiver<ActorSysMsg<T::SendMessage, T::CallMessage, T::ErrorType>>, actor_ref: ActorRef<T::SendMessage, T::CallMessage, T::ErrorType>) -> Self {
        ActorExecutor {
            instance, inbox, actor_ref
        }
    }

    /// Executor run loop.
    pub(crate) async fn run(&mut self) -> crate::result::Result<()> {
        use ActorSysMsg::*;
        let r = self.instance.on_initialization().await;
        match self.handle_control(r).await {
            Ok(()) => {},
            err => {
                return err;
            },
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
                Terminate => {
                    todo!();
                }
            }
        }
        Ok(())
    }

    /// Several of the actor methods return a Control message, handle it here.
    async fn handle_control(&mut self, control: Control<T::InternalMessage>) -> crate::result::Result<()> {
        match control {
            Control::Ok => Ok(()),
            Control::Terminate => Err(crate::result::Error::Terminated),
            Control::Shutdown => {
                self.actor_ref.shutdown().await
            },
            Control::AddFuture(f) => {
                todo!();
            }
        }
    }
}


/// Messages to the actor get wrapped in an ActorSysMsg.
pub(crate) enum ActorSysMsg<S, C, E>
where S: Send, C: Send, E: Send {
    /// Normal shutdown
    Shutdown,
    /// Terminate immediately
    Terminate,
    /// A send message
    Send(S),
    /// A call message
    Call(C, tokio::sync::oneshot::Sender<Result<C, E>>),
}


#[cfg(test)]
mod tests {
    use crate::create_actor;
    use crate::test_code::tests::SimpleCounter;

    /// Test that the actor shuts down if quit is returned by on_initialization()
    #[tokio::test]
    async fn test_init_quit() {
        let instance = SimpleCounter::new(true);
        let (actor, handle) = create_actor(instance).await.unwrap();
        let r = handle.await;
        assert!(r.is_ok());
    }
}