#[macro_use]
extern crate log;

use async_stream::stream;
use poem::{listener::TcpListener, Server};
use poem_grpc::{Request, Response, RouteGrpc, Status};

poem_grpc::include_proto!("helloworld");

struct GreeterService;

#[poem::async_trait]
impl Greeter for GreeterService {

    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status>{
        let meta = request.metadata().clone();
        let request = request.into_inner();
        info!("'{}' from: {meta:?}", request.name);

        let reply = HelloReply {
            message: format!("Hello {}", request.name),
        };
        Ok(Response::new(reply))
    }

    async fn say_hello_lots(
        &self,
        request: poem_grpc::Request<poem_grpc::Streaming<HelloRequest>>,
    ) -> ::std::result::Result<
        poem_grpc::Response<poem_grpc::Streaming<HelloReply>>,
        poem_grpc::Status,
    > {
        let meta = request.metadata().clone();
        info!("say_hello_lots: {meta:?}");
        let responses = stream! {
            for await res in request.into_inner() {
                yield res.map(|hello_request| {
                    info!("say_hello_lots: {}", hello_request.name);
                    HelloReply { message: format!("Hello Lots {}", hello_request.name), }
                })
            }
        };
        Ok(Response::new_streaming(responses))
    }

}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    info!("Starting up...");

    let route = RouteGrpc::new().add_service(GreeterServer::new(GreeterService));
    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(route)
        .await
    // Err(std::io::ErrorKind::Other)?
}
