use std::sync::Arc;

use hyper_rustls::ConfigBuilderExt;
use rustls::{ClientConfig, KeyLogFile};

#[tokio::main]
async fn main() {
    let mut client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_native_roots()
        .with_no_client_auth();
    // this is the fun option
    client_config.key_log = Arc::new(KeyLogFile::new());

    let conn = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(client_config)
        .https_or_http()
        .enable_http1()
        .build();

    let client = hyper::Client::builder().build::<_, hyper::Body>(conn);

    let response = client
        .get("https://example.org".parse().unwrap())
        .await
        .unwrap();

    let body = String::from_utf8(
        hyper::body::to_bytes(response.into_body())
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    println!("response body: {body}");
}
