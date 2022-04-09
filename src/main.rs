use color_eyre::Report;
use hyper::{
    server::accept::Accept, service::make_service_fn, service::service_fn, Body, Request, Response,
};
use socket2::{Domain, Protocol, Socket, Type};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::pin::Pin;
use std::task::Context;
use std::time::Duration;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Report> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let acc = Acceptor::new(addr)?;

    hyper::Server::builder(acc)
        .serve(make_service_fn(|_: &TcpStream| async move {
            Ok::<_, Report>(service_fn(hello_world))
        }))
        .await?;
    Ok(())
}

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", req.method(), req.uri());
    tokio::time::sleep(Duration::from_millis(250)).await;
    Ok(Response::builder()
        .body(Body::from("Hello World!\n"))
        .unwrap())
}

struct Acceptor {
    ln: tokio::net::TcpListener,
}

impl Acceptor {
    fn new(addr: SocketAddr) -> Result<Self, Report> {
        let socket =
            Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP)).unwrap();
        println!("Binding...");
        socket.bind(&addr.into())?;
        println!(
            "Listening on {}...",
            socket.local_addr()?.as_socket().unwrap()
        );
        socket.listen(128)?;
        socket.set_nonblocking(true)?;
        let fd = socket.as_raw_fd();
        std::mem::forget(socket);
        let ln = unsafe { std::net::TcpListener::from_raw_fd(fd) };
        let ln = tokio::net::TcpListener::from_std(ln)?;

        Ok(Self { ln })
    }
}

impl Accept for Acceptor {
    type Conn = TcpStream;
    type Error = Report;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        let (stream, _) = futures::ready!(self.ln.poll_accept(cx)?);
        Some(Ok(stream)).into()
    }
}
