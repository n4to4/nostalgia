use std::{convert::Infallible, sync::Arc, time::Duration};

use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use tokio::sync::Semaphore;
use tower::limit::ConcurrencyLimit;

const MAX_INFLIGHT_REQUESTS: usize = 5;

#[tokio::main]
async fn main() {
    let sem = Arc::new(Semaphore::new(MAX_INFLIGHT_REQUESTS));
    let app = make_service_fn(move |_stream: &AddrStream| {
        let sem = sem.clone();
        async move {
            Ok::<_, Infallible>(ConcurrencyLimit::with_semaphore(
                service_fn(|req: Request<Body>| async move {
                    println!("{} {}", req.method(), req.uri());
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    Ok::<_, Infallible>(
                        Response::builder()
                            .body(Body::from("Hello World!\n"))
                            .unwrap(),
                    )
                }),
                sem,
            ))
        }
    });

    Server::bind(&([127, 0, 0, 1], 1025).into())
        .serve(app)
        .await
        .unwrap();
}
