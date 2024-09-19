//! This is the classic helloworld implementation for the minactor framework.
//! It creates a very simple actor (HelloWorldActor) which prints hello world when it is
//! sent a Hello message.
use minactor::{create_actor, Actor, Control};


/// The type of messages sent to the HelloWorldActor
#[derive(Clone, PartialEq, Eq)]
enum HelloMsg {
    Hello,
}

/// The actor that implements HelloWorld
#[derive(Clone)]
struct HelloWorldActor {}

impl HelloWorldActor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for HelloWorldActor {
    /// We want to send a simple Hello to the actor.
    type SendMessage = HelloMsg;
    /// We're not using these types.
    type CallMessage = ();
    type InternalMessage = ();
    type ErrorType = ();

    async fn handle_sends(&mut self, msg: HelloMsg) -> Control<Self::InternalMessage> {
        match msg {
            HelloMsg::Hello => {
                println!("the actor says hello");
                Control::Ok
            },
        }
    }
}


#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let instance = HelloWorldActor::new();
    // create the actor
    let (actor_ref, handle) = create_actor(instance).await.expect("unable to create actor");
    // send the hello message
    actor_ref.send(HelloMsg::Hello).await.expect("unable to send message");
    // shutdown the actor straight away, it will finish processing messages before it shuts down
    actor_ref.shutdown().await.unwrap();
    // wait for the actor task to finish
    handle.await.expect("error waiting for handle").expect("minactor returned error");
}
