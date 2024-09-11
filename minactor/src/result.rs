

/// Standard Result used in the library
pub type MinActorResult<T> = Result<T, MinActorError>;

/// Standard error type used in the library
#[derive(Debug)]
pub enum MinActorError {
    /// The handler was not implemented, for example a message was sent with no handler defined.
    HandlerNotImplemented,
}

