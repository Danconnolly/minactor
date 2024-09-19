//! minactor is a minimal [actor](https://en.wikipedia.org/wiki/Actor_model) framework for [tokio](https://tokio.rs/).
//!
//! Actors created in minactor have a tiny overhead and messages are passed using tokio channels. Each instance of an actor
//! has a single thread of control (a tokio async task). Creating actors is simple.
//!
//! It is designed for single system implementations, not clusters of systems.

mod actor;
mod actor_ref;
mod control;
mod executor;
mod result;
mod test_code;


pub use actor::{Actor, create_actor};
pub use actor_ref::ActorRef;
pub use control::Control;
pub use result::Error;
