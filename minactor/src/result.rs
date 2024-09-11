

/// Standard Result used in the library
pub type MinActorResult<T> = Result<T, MinActorError>;

/// Standard error type used in the library
#[derive(Debug)]
pub enum MinActorError {
    Dummy,
}

