use hyper::{server::conn::AddrStream, service::Service, Body, Request, Response, Server};
use std::{
    convert::Infallible,
    future::{ready, Ready},
    task::{Context, Poll},
};

#[tokio::main]
async fn main() {
    Server::bind(&([127, 0, 0, 1], 1025).into())
        .serve(MyServiceFactory)
        .await
        .unwrap();
}

struct MyServiceFactory;

impl Service<&AddrStream> for MyServiceFactory {
    type Response = MyService;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: &AddrStream) -> Self::Future {
        println!("Accepted connection from {}", req.remote_addr());
        ready(Ok(MyService))
    }
}

struct MyService;

impl Service<Request<Body>> for MyService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        println!("Handling {req:?}");
        ready(Ok(Response::builder()
            .body("Hello World!\n".into())
            .unwrap()))
    }
}
