use std::{convert::Infallible, time::Duration};

use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use tower::limit::ConcurrencyLimitLayer;
use tower::ServiceBuilder;

const MAX_INFLIGHT_REQUESTS: usize = 5;

#[tokio::main]
async fn main() {
    let app = make_service_fn(move |_stream: &AddrStream| async move {
        let svc = ServiceBuilder::new()
            .layer(ConcurrencyLimitLayer::new(MAX_INFLIGHT_REQUESTS))
            .service(service_fn(hello_world));
        Ok::<_, Infallible>(svc)
    });

    Server::bind(&([127, 0, 0, 1], 1025).into())
        .serve(app)
        .await
        .unwrap();
}

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", req.method(), req.uri());
    tokio::time::sleep(Duration::from_millis(250)).await;
    Ok(Response::builder()
        .body(Body::from("Hello World!\n"))
        .unwrap())
}
