//! This is the classic helloworld implementation for the minactor framework.
//! It creates a very simple actor (HelloWorldActor) which prints hello world when it is
//! sent a Hello message.
use async_trait::async_trait;
use minactor::{create_actor, Actor};


/// The type of messages sent to the HelloWorldActor
#[derive(Clone, PartialEq, Eq)]
enum HelloMsg {
    Hello,
}

/// The actor that implements HelloWorld
#[derive(Clone)]
struct HelloWorldActor {}

#[async_trait]
impl Actor for HelloWorldActor {
    /// We want to send a simple Hello to the actor.
    type MessageType = HelloMsg;
    /// We dont have any need to initialize the actor.
    type CreationArguments = ();
    /// We're not using an error type.
    type ErrorType = ();

    fn new(_args: Self::CreationArguments) -> Self {
        HelloWorldActor {}
    }

    async fn handle_sends(&mut self, msg: HelloMsg) -> Result<(), Self::ErrorType> {
        match msg {
            HelloMsg::Hello => {
                println!("the actor says hello");
                Ok(())
            },
        }
    }
}


#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    // create the actor
    let (actor_ref, handle) = create_actor::<HelloWorldActor>(()).await.expect("unable to create actor");
    // send the hello message
    actor_ref.send(HelloMsg::Hello).await.expect("unable to send message");
    // shutdown the actor straight away, it will finish processing messages before it shuts down
    actor_ref.shutdown().await.unwrap();
    // wait for the actor task to finish
    handle.await.expect("error waiting for handle").expect("minactor returned error");
}
