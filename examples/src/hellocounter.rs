//! A simple example of an actor with internal state and a function that returns a value.
//!
//! The actor counts the number of times it is sent a Hello message and returns the count when
//! queried.
use minactor::{create_actor, Actor, Control};


/// The type of messages sent to the HelloCounterActor.
#[derive(Clone, Debug, PartialEq, Eq)]
enum HelloCounterMsg {
    Hello,
}

/// The type of messages used for HelloCounterActor calls.
#[derive(Clone, Debug, PartialEq, Eq)]
enum HelloCounterCalls {
    QueryCount,
    CountValue(u64),
}

/// The actor that counts the messages.
#[derive(Clone)]
struct HelloCounterActor {
    /// The count of the number of hello messages received.
    count: u64,
}

impl HelloCounterActor {
    pub fn new() -> Self {
        Self {
            count: 0,
        }
    }
}


impl Actor for HelloCounterActor {
    type SendMessage = HelloCounterMsg;
    type CallMessage = HelloCounterCalls;
    type ErrorType = ();

    async fn handle_sends(&mut self, msg: Self::SendMessage) -> Control {
        match msg {
            HelloCounterMsg::Hello => {
                self.count += 1;
                Control::Ok
            }
        }
    }

    async fn handle_calls(&mut self, msg: Self::CallMessage) -> (Control, Result<Self::CallMessage, Self::ErrorType>) {
        match msg {
            HelloCounterCalls::QueryCount => {
                // return the count
                // note that since the actor is single-threaded we dont need to bother with complex synchronization logic
                (Control::Ok, Ok(HelloCounterCalls::CountValue(self.count)))
            }
            _ => {
                panic!("impossible call message received");
            }
        }
    }
}


#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let instance = HelloCounterActor::new();
    let (actor_ref, handle) = create_actor(instance).await.expect("unable to create actor");
    for _i in 0..5082 {
        actor_ref.send(HelloCounterMsg::Hello).await.expect("unable to send message");
    }
    let reply = actor_ref.call(HelloCounterCalls::QueryCount).await.unwrap().unwrap();
    if let HelloCounterCalls::CountValue(count) = reply {
        println!("count is {}", count);
    }
    actor_ref.shutdown().await.unwrap();
    handle.await.expect("error waiting for handle");
}
