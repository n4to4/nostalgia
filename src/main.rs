use std::convert::Infallible;
use tokio::net::TcpListener;

use hyper::{
    server::conn::{AddrIncoming, AddrStream},
    service::make_service_fn,
    service::service_fn,
    Body, Request, Response, Server,
};

#[tokio::main]
async fn main() {
    let app = make_service_fn(move |_stream: &AddrStream| async move {
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let ln = TcpListener::bind("127.0.0.1:1025").await.unwrap();
    Server::builder(AddrIncoming::from_listener(ln).unwrap())
        .serve(app)
        .await
        .unwrap();
}

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", req.method(), req.uri());
    Ok(Response::builder()
        .body(Body::from("Hello World!\n"))
        .unwrap())
}
