use std::time::Duration;
use async_trait::async_trait;
use minactor::{create_actor, Actor, MinActorResult};


/// The type of messages sent to the HelloWorldActor
#[derive(Debug, PartialEq, Eq)]
enum HelloMsg {
    Hello,
    HelloAgain,
    Goodbye,
}

/// The actor that implements HelloWorld
struct HelloWorldActor {}

#[async_trait]
impl Actor for HelloWorldActor {
    type MessageType = HelloMsg;
    type Arguments = ();

    fn new(_args: Self::Arguments) -> Self {
        HelloWorldActor {}
    }

    async fn handle_sends(&mut self, msg: HelloMsg) -> MinActorResult<()> {
        match msg {
            HelloMsg::Hello => {
                println!("actor: hello");
                Ok(())
            },
            _ => { println!("impossible message received.");
                Ok(())
            }
        }
    }

    async fn handle_calls(&mut self, msg: HelloMsg) -> MinActorResult<HelloMsg> {
        match msg {
            HelloMsg::HelloAgain => {
                println!("actor: hello again");
                Ok(HelloMsg::Goodbye)
            }
            _ => {
                println!("impossible call message received");
                Ok(HelloMsg::Hello)
            }
        }
    }
}


#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let (actor_ref, handle) = create_actor::<HelloWorldActor>(()).await.expect("unable to create actor");
    actor_ref.send(HelloMsg::Hello).await.expect("unable to send message");
    let reply = actor_ref.call(HelloMsg::HelloAgain).await.unwrap();
    if reply == HelloMsg::Goodbye {
        println!("goodbye received from actor")
    }
    tokio::time::sleep(Duration::new(1, 0)).await;
    actor_ref.shutdown().await.unwrap();
    handle.await.expect("error waiting for handle");
}
