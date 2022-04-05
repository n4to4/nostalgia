use std::{
    convert::Infallible,
    future::{ready, Future, Ready},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use hyper::{server::conn::AddrStream, service::Service, Body, Request, Response, Server};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::Sleep;
use tokio_util::sync::PollSemaphore;

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
    reqs_semaphore: PollSemaphore,
    permit: Option<OwnedSemaphorePermit>,
}

impl Default for MyServiceFactory {
    fn default() -> Self {
        Self {
            conn_semaphore: PollSemaphore::new(Arc::new(Semaphore::new(MAX_CONNS))),
            reqs_semaphore: PollSemaphore::new(Arc::new(Semaphore::new(MAX_INFLIGHT_REQUESTS))),
            permit: None,
        }
    }
}

impl Service<&AddrStream> for MyServiceFactory {
    type Response = MyService;
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
        ready(Ok(MyService {
            _permit: permit,
            semaphore: self.reqs_semaphore.clone(),
            reqs_permit: None,
        }))
    }
}

struct MyService {
    _permit: OwnedSemaphorePermit,
    semaphore: PollSemaphore,
    reqs_permit: Option<OwnedSemaphorePermit>,
}

impl Service<Request<Body>> for MyService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = PretendFuture;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.reqs_permit.is_none() {
            self.reqs_permit = Some(futures::ready!(self.semaphore.poll_acquire(cx)).unwrap());
        }
        Ok(()).into()
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let permit = self.reqs_permit.take().expect(
            "you didn't drive me to readiness did you? you know that's a tower crime right?",
        );
        println!("{} {}", req.method(), req.uri());
        PretendFuture {
            sleep: tokio::time::sleep(Duration::from_millis(250)),
            response: Some(Response::builder().body("Hello World\n".into()).unwrap()),
            permit,
        }
    }
}

pin_project_lite::pin_project! {
    struct PretendFuture {
        #[pin]
        sleep: Sleep,
        response: Option<Response<Body>>,
        permit: OwnedSemaphorePermit,
    }
}

impl Future for PretendFuture {
    type Output = Result<Response<Body>, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        futures::ready!(this.sleep.poll(cx));
        Ok(this.response.take().unwrap()).into()
    }
}
