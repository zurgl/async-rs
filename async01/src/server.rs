use std::{collections::HashMap, thread::spawn};
use tiny_http::{Response, Server};

pub fn start() {
    spawn(run);
}

fn run() {
    let server = Server::http("127.0.0.1:1729").unwrap();

    let mut map: HashMap<_, usize> = HashMap::new();

    for req in server.incoming_requests() {
        let count = *map
            .entry(req.remote_addr().unwrap().ip())
            .and_modify(|c| *c += 1)
            .or_default();

        log::info!("Serving {:?}", req.url());
        let res = if count < 5 {
            Response::from_string("The mainframe is warming up...").with_status_code(503)
        } else {
            Response::from_string("Hello from test server!")
        };

        req.respond(res).unwrap();
    }
}
