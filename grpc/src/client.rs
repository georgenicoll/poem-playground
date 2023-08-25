#[macro_use]
extern crate log;

use poem_grpc::{ClientConfig, Request};
use tokio_stream::StreamExt;

poem_grpc::include_proto!("helloworld");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    env_logger::init();

    let client = GreeterClient::new(
        ClientConfig::builder()
            .uri("http://localhost:3000")
            .build()
            .unwrap()
    );

    //say_hello
    info!("=== Sending single request...");
    let request = Request::new(HelloRequest {
        name: "Bob".into(),
    });
    let response = client.say_hello(request).await?;
    info!("RESPONSE={response:?}");

    info!("=== Sending streaming request...");
    //say_hello_lots
    // use async_stream::stream;
    // let hello_requests = stream! {
    //     for i in 0..10 {
    //         yield Ok(HelloRequest {
    //             name: format!("request {}", i),
    //         })
    //     }
    // };
    use tokio_stream::{self as ts_stream};
    let hello_requests = ts_stream::iter(0..10).map(|i|
        Ok(HelloRequest { name: format!("Request {}", i) })
    );
    let requests = poem_grpc::Request::new_streaming(hello_requests);
    let responses = client.say_hello_lots(requests).await?;
    let mut streaming_responses = responses.into_inner();
    while let Some(response) = streaming_responses.next().await {
        match response {
            Ok(reply) => info!("REPLY={reply:?}"),
            Err(status) => error!("FAILED: {status}"),
        }
    };

    Ok(())
}