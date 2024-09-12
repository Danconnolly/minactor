//! A simple example of an actor with internal state and a function that returns a value.
//!
//! The actor counts the number of times it is sent a Hello message and returns the count when
//! queried.
use async_trait::async_trait;
use minactor::{create_actor, Actor};


/// The type of messages sent to the HelloCounterActor.
#[derive(Debug, PartialEq, Eq)]
enum HelloCounterMsg {
    Hello,
    QueryCount,
    CountValue(u64),
}

/// The actor that counts the messages.
struct HelloCounterActor {
    /// The count of the number of hello messages received.
    count: u64,
}

#[async_trait]
impl Actor for HelloCounterActor {
    type MessageType = HelloCounterMsg;
    /// Counters should probably always start at zero, but we want to give the capability to
    /// initialize the counter with a particular value. We're also going to add a second
    /// value that we dont use just for demonstration purposes.
    type CreationArguments = (u64, String);
    type ErrorType = ();

    /// Initialize the counter with a value.
    fn new(args: Self::CreationArguments) -> Self  {
        println!("creating actor, msg={}", args.1);
        Self { count: args.0 }
    }

    async fn handle_sends(&mut self, msg: Self::MessageType) -> Result<(), Self::ErrorType> {
        match msg {
            HelloCounterMsg::Hello => {
                self.count += 1;
                Ok(())
            },
            _ => {
                // only the HelloCounterMsg should be received here in the send handler, if another type
                // is received then its probably a programming error
                panic!("impossible message received.");
            }
        }
    }

    async fn handle_calls(&mut self, msg: Self::MessageType) -> Result<Self::MessageType, Self::ErrorType> {
        match msg {
            HelloCounterMsg::QueryCount => {
                // return the count
                // note that since the actor is single-threaded we dont need to bother with complex synchronization logic
                Ok(HelloCounterMsg::CountValue(self.count))
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
    let (actor_ref, handle) = create_actor::<HelloCounterActor>((5, "all fine, nothing to see here".to_string())).await.expect("unable to create actor");
    for i in 0..5082 {
        actor_ref.send(HelloCounterMsg::Hello).await.expect("unable to send message");
    }
    let reply = actor_ref.call(HelloCounterMsg::QueryCount).await.unwrap().unwrap();
    if let HelloCounterMsg::CountValue(count) = reply {
        println!("count is {}", count);
    }
    actor_ref.shutdown().await.unwrap();
    handle.await.expect("error waiting for handle");
}
