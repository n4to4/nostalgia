use color_eyre::Report;
use futures::Future;
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
use tokio::time::Sleep;

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

enum Acceptor {
    Waiting { sleep: Sleep, socket: Socket },
    Listening { ln: tokio::net::TcpListener },
}

impl Acceptor {
    fn new(addr: SocketAddr) -> Result<Self, Report> {
        let socket =
            Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP)).unwrap();
        println!("Binding...");
        socket.bind(&addr.into())?;

        Ok(Self::Waiting {
            sleep: tokio::time::sleep(Duration::from_secs(2)),
            socket,
        })
    }
}

impl Accept for Acceptor {
    type Conn = TcpStream;
    type Error = Report;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        match unsafe { self.as_mut().get_unchecked_mut() } {
            Acceptor::Waiting { sleep, socket } => {
                let sleep = unsafe { Pin::new_unchecked(sleep) };
                futures::ready!(sleep.poll(cx));

                println!(
                    "Listening on {}...",
                    socket.local_addr()?.as_socket().unwrap()
                );
                socket.listen(128)?;
                socket.set_nonblocking(true)?;
                let fd = socket.as_raw_fd();
                let ln = unsafe { std::net::TcpListener::from_raw_fd(fd) };
                let ln = tokio::net::TcpListener::from_std(ln)?;
                let mut state = Self::Listening { ln };

                std::mem::swap(unsafe { self.as_mut().get_unchecked_mut() }, &mut state);
                match state {
                    Acceptor::Waiting { socket, .. } => std::mem::forget(socket),
                    _ => unreachable!(),
                };
                match unsafe { self.get_unchecked_mut() } {
                    Acceptor::Listening { ln } => {
                        let (stream, _) = futures::ready!(ln.poll_accept(cx)?);
                        Some(Ok(stream)).into()
                    }
                    _ => unreachable!(),
                }
            }
            Acceptor::Listening { ln } => {
                let (stream, _) = futures::ready!(ln.poll_accept(cx)?);
                Some(Ok(stream)).into()
            }
        }
    }
}
