use std::future::Future;
use std::pin::Pin;
use log::warn;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio_util::task::TaskTracker;
use crate::{Actor, ActorRef};
use crate::control::Control;

/// The ActorExecutor executes the actor, receiving messages and forwarding them to handlers.
pub(crate) struct ActorExecutor<T>
where T: Actor + Send {
    /// The actor struct.
    instance: T,
    /// Messages are received here.
    inbox: Receiver<ActorSysMsg<T::SendMessage, T::CallMessage, T::ErrorType>>,
    /// Reference to the actor.
    actor_ref: ActorRef<T>,
    /// Tasks that are being tracked.
    tasks: TaskTracker,
}

impl<T> ActorExecutor<T>
where
    T: Actor + Send + Sync
{
    /// Create a new instance of the executor.
    pub(crate) fn new(instance: T, inbox: Receiver<ActorSysMsg<T::SendMessage, T::CallMessage, T::ErrorType>>, actor_ref: ActorRef<T>) -> Self {
        ActorExecutor {
            instance, inbox, actor_ref, tasks: TaskTracker::new()
        }
    }

    /// Executor run loop.
    pub(crate) async fn run(&mut self) {
        use ActorSysMsg::*;
        let r = self.instance.on_initialization(self.actor_ref.clone()).await;
        match self.handle_control(r).await {
            Ok(()) => {},
            _err => {
                return;
            },
        }
        loop {
            // main message processing loop
            select! {
                _ = self.actor_ref.terminate_token.cancelled() => { break; }
                r = self.inbox.recv() => {
                    match r {
                        None => { break; }
                        Some(sys_msg) => {
                            match sys_msg {
                                Shutdown => {
                                    let r = self.instance.on_shutdown().await;
                                    match r {
                                        Control::Ok | Control::Shutdown | Control::Terminate => {},
                                        Control::SpawnFuture(f) => {
                                            self.spawn_future(f);
                                        }
                                    }
                                    break;
                                },
                                Send(msg) => {
                                    let r = self.instance.handle_sends(msg).await;
                                    self.handle_control(r).await;       // todo
                                },
                                Call(msg, dest) => {
                                    let (control, result) = self.instance.handle_calls(msg).await;
                                    match dest.send(result) {
                                        Ok(()) => {},
                                        Err(_) => {
                                            warn!("unable to send reply of call message to caller.");
                                        }
                                    }
                                    self.handle_control(control).await; // todo
                                },
                            }
                        }
                    }
                }
            }
        }
        if ! self.actor_ref.terminate_token.is_cancelled() {
            // if not terminated, then clean up
            self.tasks.close();
            if ! self.tasks.is_empty() {
                self.tasks.wait().await;
            }
        }
    }

    /// Several of the actor methods return a Control message, handle it here.
    async fn handle_control(&mut self, control: Control) -> crate::result::Result<()> {
        match control {
            Control::Ok => Ok(()),
            Control::Terminate => Err(crate::result::Error::Terminated),
            Control::Shutdown => {
                // queue up a shutdown message
                self.actor_ref.shutdown().await
            },
            Control::SpawnFuture(f) => {
                self.spawn_future(f);
                Ok(())
            }
        }
    }

    /// Spawn the future into a task and track it.
    fn spawn_future(&mut self, f: Pin<Box<dyn Future<Output=()> + Send>>) {
        self.tasks.spawn(f);
    }
}


/// Messages to the actor get wrapped in an ActorSysMsg.
pub(crate) enum ActorSysMsg<S, C, E>
where S: Send, C: Send, E: Send {
    /// Normal shutdown
    Shutdown,
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
        let (_actor, handle) = create_actor(instance).await.unwrap();
        let r = handle.await;
        assert!(r.is_ok());
    }
}