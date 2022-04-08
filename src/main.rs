use std::{convert::Infallible, time::Duration};

use hyper::{server::conn::AddrStream, service::make_service_fn, Body, Request, Response, Server};
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::ServiceBuilder;

const MAX_INFLIGHT_REQUESTS: usize = 5;

#[tokio::main]
async fn main() {
    let reqs_limit = GlobalConcurrencyLimitLayer::new(MAX_INFLIGHT_REQUESTS);
    let app = make_service_fn(move |_stream: &AddrStream| {
        std::future::ready(Ok::<_, Infallible>(
            ServiceBuilder::new()
                .layer(reqs_limit.clone())
                .then(|res: Result<Response<Body>, Infallible>| async move {
                    println!("Just served a request!");
                    res
                })
                .service_fn(hello_world),
        ))
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
