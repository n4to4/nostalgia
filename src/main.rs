use std::{
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures::future::BoxFuture;
use hyper::{
    server::conn::AddrStream, service::make_service_fn, service::Service, Body, Request, Response,
    Server,
};
use tokio::sync::Semaphore;
use tower::limit::ConcurrencyLimit;

const MAX_INFLIGHT_REQUESTS: usize = 5;

#[tokio::main]
async fn main() {
    let sem = Arc::new(Semaphore::new(MAX_INFLIGHT_REQUESTS));
    let app = make_service_fn(move |_stream: &AddrStream| {
        let sem = sem.clone();
        async move { Ok::<_, Infallible>(ConcurrencyLimit::with_semaphore(MyService, sem)) }
    });

    Server::bind(&([127, 0, 0, 1], 1025).into())
        .serve(app)
        .await
        .unwrap();
}

struct MyService;

impl Service<Request<Body>> for MyService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        println!("{} {}", req.method(), req.uri());
        Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(250)).await;
            Ok(Response::builder().body("Hello World\n".into()).unwrap())
        })
    }
}
