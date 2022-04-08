use std::{convert::Infallible, sync::Arc, time::Duration};

use hyper::{server::conn::AddrStream, service::make_service_fn, Body, Request, Response, Server};
use tokio::sync::Semaphore;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::ServiceBuilder;

const MAX_CONNS: usize = 50;

#[tokio::main]
async fn main() {
    let conns_limit = Arc::new(Semaphore::new(MAX_CONNS));
    let app = make_service_fn(move |_stream: &AddrStream| {
        let conns_limit = conns_limit.clone();
        async move {
            let permit = Arc::new(conns_limit.acquire_owned().await.unwrap());
            Ok::<_, Infallible>(
                ServiceBuilder::new()
                    .then(|res: Result<Response<Body>, Infallible>| {
                        drop(permit);
                        std::future::ready(res)
                    })
                    .service_fn(hello_world),
            )
        }
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
