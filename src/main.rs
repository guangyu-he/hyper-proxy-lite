use anyhow::Result;
use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{body::Incoming, Method, Request, Response};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioIo;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server starts at http://127.0.0.1:8080");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("Error: {}", e);
            }
        });
    }
}

/// Handle a client connection.
/// This function serves HTTP/1.1 connections and handles both regular HTTP requests
/// and CONNECT requests for HTTPS tunneling.
async fn handle_client(stream: TcpStream) -> Result<()> {
    let io = TokioIo::new(stream);

    http1::Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .serve_connection(io, service_fn(proxy))
        .with_upgrades()
        .await?;

    Ok(())
}

/// Proxy function to handle incoming requests.
/// It distinguishes between regular HTTP requests and CONNECT requests.
/// For CONNECT requests, it establishes a tunnel to the target server.
/// For regular HTTP requests, it forwards the request and returns the response.
async fn proxy(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
    if req.method() == Method::CONNECT {
        handle_connect(req).await
    } else {
        handle_http(req).await
    }
}

/// Handle regular HTTP requests by forwarding them to the target server
/// and returning the response back to the client.
/// This function modifies the request URI to ensure it is in the correct format
/// for the target server.
/// It uses a hyper client to send the request and retrieve the response.
/// The response body is boxed for compatibility with the expected return type.
/// It also includes error handling to manage potential issues during the request process.
async fn handle_http(mut req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
    println!("HTTP: {} {}", req.method(), req.uri());

    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();
    let uri = req.uri().clone();

    let uri_string = format!(
        "http://{}{}",
        uri.authority()
            .ok_or_else(|| anyhow::anyhow!("Missing authority in URI"))?,
        uri.path_and_query().map(|x| x.as_str()).unwrap_or("/")
    );

    *req.uri_mut() = uri_string
        .parse()
        .map_err(|_| anyhow::anyhow!("Failed to parse URI: {}", uri_string))?;

    let response = client
        .request(req)
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request error: {}", e))?;

    Ok(response.map(|body| body.boxed()))
}

/// Handle CONNECT requests to establish a tunnel for HTTPS traffic.
/// This function upgrades the connection and spawns a new task to manage
/// the bidirectional data transfer between the client and the target server.
/// It returns a 200 OK response to the client to indicate that the tunnel
/// has been successfully established.
/// It includes error handling to manage potential issues during the upgrade process.
async fn handle_connect(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
    let addr = req
        .uri()
        .authority()
        .ok_or_else(|| anyhow::anyhow!("CONNECT request missing authority in URI"))?
        .to_string();

    println!("HTTPS CONNECT: {}", addr);

    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                if let Err(e) = tunnel(upgraded, addr).await {
                    eprintln!("Tunnel error: {}", e);
                }
            }
            Err(e) => eprintln!("Upgrade error: {}", e),
        }
    });

    let response = Response::builder()
        .status(200)
        .body(
            Empty::<Bytes>::new()
                .map_err(|never| match never {})
                .boxed(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to build response: {}", e))?;

    Ok(response)
}

/// Establish a bidirectional tunnel between the upgraded client connection
/// and the target server specified by the address.
/// This function connects to the target server and uses `copy_bidirectional`
/// to transfer data between the client and server.
/// It includes error handling to manage potential issues during the connection
/// and data transfer process.
/// It returns a Result indicating success or failure of the tunneling operation.
/// The function is asynchronous and leverages Tokio's async I/O capabilities.
/// It is designed to work with upgraded HTTP connections, typically used for HTTPS tunneling.
async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    copy_bidirectional(&mut upgraded, &mut server).await?;

    Ok(())
}
