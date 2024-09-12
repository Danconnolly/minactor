//! minactor is a minimal [actor](https://en.wikipedia.org/wiki/Actor_model) framework for [tokio](https://tokio.rs/).
//!
//! Actors created in minactor have a tiny overhead and messages are passed using tokio channels. Each instance of an actor
//! has a single thread of control (a tokio async task). Creating actors is simple.
//!
//! It is designed for single system implementations, not clusters of systems.

mod actor;
mod result;
mod actor_ref;
mod executor;

pub use actor::{Actor, create_actor};
pub use actor_ref::ActorRef;
pub use result::*;
