use hyper::{server::conn::AddrStream, service::Service, Body, Request, Response, Server};
use std::{
    convert::Infallible,
    future::{ready, Ready},
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::PollSemaphore;

#[tokio::main]
async fn main() {
    Server::bind(&([127, 0, 0, 1], 1025).into())
        .serve(MyServiceFactory::default())
        .await
        .unwrap();
}

const MAX_CONNS: usize = 5;

struct MyServiceFactory {
    semaphore: PollSemaphore,
    permit: Option<OwnedSemaphorePermit>,
}

impl Default for MyServiceFactory {
    fn default() -> Self {
        Self {
            semaphore: PollSemaphore::new(Arc::new(Semaphore::new(MAX_CONNS))),
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
            self.permit = Some(futures::ready!(self.semaphore.poll_acquire(cx)).unwrap());
        }
        Ok(()).into()
    }

    fn call(&mut self, _req: &AddrStream) -> Self::Future {
        let permit = self.permit.take().expect(
            "you didn't drive me to readiness did you? you know that's a tower crime right?",
        );

        println!(
            "⬆️ {} connections",
            MAX_CONNS - self.semaphore.available_permits()
        );
        ready(Ok(MyService { _permit: permit }))
    }
}

struct MyService {
    _permit: OwnedSemaphorePermit,
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
