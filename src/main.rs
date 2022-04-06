use std::{
    convert::Infallible,
    future::{ready, Ready},
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures::future::BoxFuture;
use hyper::{server::conn::AddrStream, service::Service, Body, Request, Response, Server};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::PollSemaphore;
use tower::limit::ConcurrencyLimit;

#[tokio::main]
async fn main() {
    Server::bind(&([127, 0, 0, 1], 1025).into())
        .serve(MyServiceFactory::default())
        .await
        .unwrap();
}

const MAX_CONNS: usize = 50;
const MAX_INFLIGHT_REQUESTS: usize = 5;

struct MyServiceFactory {
    conn_semaphore: PollSemaphore,
    reqs_semaphore: Arc<Semaphore>,
    permit: Option<OwnedSemaphorePermit>,
}

impl Default for MyServiceFactory {
    fn default() -> Self {
        Self {
            conn_semaphore: PollSemaphore::new(Arc::new(Semaphore::new(MAX_CONNS))),
            reqs_semaphore: Arc::new(Semaphore::new(MAX_INFLIGHT_REQUESTS)),
            permit: None,
        }
    }
}

impl Service<&AddrStream> for MyServiceFactory {
    type Response = ConcurrencyLimit<MyService>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.permit.is_none() {
            self.permit = Some(futures::ready!(self.conn_semaphore.poll_acquire(cx)).unwrap());
        }
        Ok(()).into()
    }

    fn call(&mut self, _req: &AddrStream) -> Self::Future {
        let permit = self.permit.take().expect(
            "you didn't drive me to readiness did you? you know that's a tower crime right?",
        );
        println!(
            "↑ {} connections",
            MAX_CONNS - self.conn_semaphore.available_permits()
        );
        ready(Ok(ConcurrencyLimit::with_semaphore(
            MyService {
                _conn_permit: permit,
            },
            self.reqs_semaphore.clone(),
        )))
    }
}

struct MyService {
    _conn_permit: OwnedSemaphorePermit,
}

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
