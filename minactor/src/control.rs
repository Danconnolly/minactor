use std::future::Future;


/// The Control enum is used by an [Actor] to pass instructions the actor executor.
///
/// This enum is expected to be parameterized on an actor's InternalMessage type.
pub enum Control<InternalMessage>
where InternalMessage: Send + Sync {
    /// Processing was completed with no errors and no additional instructions needed.
    Ok,
    /// Initiate Shutdown of the actor. For a description of shutdown, see [Actor].
    Shutdown,
    /// Terminate the actor. For a description of termination, see [Actor].
    Terminate,
    /// Add the future to the actor's waitlist. The actor's handle_future() is called when the
    /// future completes.
    AddFuture(Box<dyn Future<Output=Option<InternalMessage>> + Send>),
}
