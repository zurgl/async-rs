use std::{
    convert::Infallible,
    future::{ready, Ready},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use hyper::{server::conn::AddrStream, service::Service, Body, Request, Response, Server};

#[tokio::main]
async fn main() {
    Server::bind(&([127, 0, 0, 1], 1025).into())
        .serve(MyServiceFactory::default())
        .await
        .unwrap();
}

#[derive(Default)]
struct MyServiceFactory {
    num_connected: Arc<AtomicU64>,
}
impl Service<&AddrStream> for MyServiceFactory {
    type Response = MyService;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: &AddrStream) -> Self::Future {
        let prev = self.num_connected.fetch_add(1, Ordering::SeqCst);
        println!(
            "⬆️ {} connections (accepted {})",
            prev + 1,
            req.remote_addr()
        );
        ready(Ok(MyService {
            num_connected: self.num_connected.clone(),
        }))
    }
}

struct MyService {
    num_connected: Arc<AtomicU64>,
}

impl Drop for MyService {
    fn drop(&mut self) {
        let prev = self.num_connected.fetch_sub(1, Ordering::SeqCst);
        println!("⬇️ {} connections (dropped)", prev - 1);
    }
}

impl Service<Request<Body>> for MyService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        println!("{} {}", req.method(), req.uri());
        ready(Ok(Response::builder()
            .body("Hello World!\n".into())
            .unwrap()))
    }
}
