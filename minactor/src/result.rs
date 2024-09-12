

/// Standard Result used in the library
pub type MinActorResult<T> = Result<T, MinActorError>;

/// Standard error type used in the library
#[derive(Debug, PartialEq, Eq)]
pub enum MinActorError {
    /// The handler was not implemented, for example a message was sent with no handler defined.
    HandlerNotImplemented,
    /// Unable to send a message, probably due to actor termination.
    UnableToSend,
    /// Unable to receive a message, probably due to actor termination.
    UnableToReceive,
}

