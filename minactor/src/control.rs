use std::future::Future;
use std::pin::Pin;

/// The Control enum is used by an [Actor] to pass instructions the actor executor.
pub enum Control {
    /// Processing was completed with no errors and no additional instructions needed.
    Ok,
    /// Initiate Shutdown of the actor. For a description of shutdown, see [Actor].
    Shutdown,
    /// Terminate the actor. For a description of termination, see [Actor].
    Terminate,
    /// Spawn the future as a new task and add to the actor's waitlist.
    SpawnFuture(Pin<Box<dyn Future<Output=()> + Send>>),
}
