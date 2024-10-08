

/// Standard Result used in the library
#[doc(hidden)]
pub type Result<T> = std::result::Result<T, Error>;

/// Standard error type used in the library
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// An unrecognized message was received.
    UnrecognizedMessage,
    /// The handler was not implemented, for example a message was sent with no handler defined.
    HandlerNotImplemented,
    /// Unable to send a message, probably due to actor termination.
    UnableToSend,
    /// Unable to receive a message, probably due to actor termination.
    UnableToReceive,
    /// Processing has been interrupted due to a terminate instruction.
    Terminated,
}

// toco: implement display

