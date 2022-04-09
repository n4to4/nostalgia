use socket2::{Domain, Protocol, Socket, Type};
use std::convert::Infallible;
use std::net::{SocketAddr, TcpListener};

use hyper::{
    server::conn::{AddrIncoming, AddrStream},
    service::make_service_fn,
    service::service_fn,
    Body, Request, Response, Server,
};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP)).unwrap();
    socket.bind(&addr.into()).unwrap();

    let addr = socket.local_addr().unwrap().as_socket().unwrap();
    println!("Bound but not listening on {}", addr);
    assert!(TcpListener::bind(addr).is_err());
    println!("As expected, nobody else can listen on the same address");
    println!("Try curling it, it'll fail (press Enter when done)");
    std::io::stdin().read_line(&mut String::new()).unwrap();

    socket.listen(128).unwrap();
    println!("Okay now we're listening (try curling it now, it should hang)");
    std::io::stdin().read_line(&mut String::new()).unwrap();

    //let app = make_service_fn(move |_stream: &AddrStream| async move {
    //    Ok::<_, Infallible>(service_fn(hello_world))
    //});

    //let ln = TcpListener::bind("127.0.0.1:1025").await.unwrap();
    //Server::builder(AddrIncoming::from_listener(ln).unwrap())
    //    .serve(app)
    //    .await
    //    .unwrap();
}

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", req.method(), req.uri());
    Ok(Response::builder()
        .body(Body::from("Hello World!\n"))
        .unwrap())
}
