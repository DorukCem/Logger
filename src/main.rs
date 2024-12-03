#![feature(backtrace_frames)]

mod logger;

use std::any::Any;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use http_body_util::{Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::Frame;
use hyper::{Method, StatusCode};

#[derive(Eq, Hash, PartialEq)]
struct RequestSignature {
    method: Method,
    path: String,
}

type CallbackFunction = fn() -> Option<Box<dyn Any>>;
pub struct Router {
    routes: HashMap<RequestSignature, CallbackFunction>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    fn add_route(&mut self, method: Method, path: String, handler: CallbackFunction) {
        let route = RequestSignature { method, path };
        self.routes.insert(route, handler);
    }

    async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let request_signature = RequestSignature {
            method: req.method().to_owned(),
            path: req.uri().path().to_string(),
        };

        let whole_body = req.collect().await?.to_bytes();

        if let Some(callback_function) = self.routes.get(&request_signature) {
            let result = callback_function();
            return Ok(Response::new(full("Try POSTing data to /echo")));
        } else {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            return Ok(not_found);
        }
    }

    pub fn get(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::GET, path, handler);
    }

    pub fn post(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::POST, path, handler);
    }

    pub fn put(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::PUT, path, handler);
    }

    pub fn delete(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::DELETE, path, handler);
    }

    pub fn patch(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::PATCH, path, handler);
    }

    pub fn head(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::HEAD, path, handler);
    }

    pub fn options(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::OPTIONS, path, handler);
    }

    pub fn connect(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::CONNECT, path, handler);
    }

    pub fn trace(&mut self, path: String, handler: CallbackFunction) {
        self.add_route(Method::TRACE, path, handler);
    }

    pub fn print_routes(&self) {
        for (k, _) in &self.routes {
            println!("{} {}", k.method, k.path)
        }
    }
}

// Some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn start_http_server(router: Router)  -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    let router = Arc::new(router);

    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let router_ref = Arc::clone(&router);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                // We bind the incoming connection to our `hello` service
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(|req| router_ref.handle_request(req)))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = Router::new();

    router.get("/".to_string(), || println!("Hello"));

    start_http_server(router).await
}
