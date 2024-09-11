use std::time::Duration;
use minactor::{create_actor, Actor, ActorRef};


/// The type of messages sent to the HelloWorldActor
enum HelloMsg {
    Hello,
}

/// The actor that implements HelloWorld
struct HelloWorldActor {}

impl Actor for HelloWorldActor {
    type MessageType = HelloMsg;
    type Arguments = ();

    fn new(args: Self::Arguments) -> Self {
        HelloWorldActor {}
    }
}


#[tokio::main]
async fn main() {
    let (actor_ref, _handle) = create_actor::<HelloWorldActor>(()).await.expect("unable to create actor");
    actor_ref.send(HelloMsg::Hello).await.expect("unable to send message");
    tokio::time::sleep(Duration::new(5, 0)).await;
}
