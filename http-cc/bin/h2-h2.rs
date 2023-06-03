use std::{net::ToSocketAddrs, str::FromStr, sync::Arc};

use color_eyre::eyre::eyre;
use rustls::{Certificate, ClientConfig, KeyLogFile, RootCertStore};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tracing::info;
use tracing_subscriber::{filter::targets::Targets, layer::SubscriberExt, util::SubscriberInitExt};

fn setup() -> color_eyre::Result<()> {
    color_eyre::install().unwrap();

    let filter_layer =
        Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info")).unwrap();
    let format_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();

    Ok(())
}

fn set_tls_connecter() -> color_eyre::Result<TlsConnector> {
    let mut root_store = RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs()? {
        root_store.add(&Certificate(cert.0))?;
    }
    let mut client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    client_config.key_log = Arc::new(KeyLogFile::new());
    client_config.alpn_protocols = vec![b"h2".to_vec()];
    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

    Ok(connector)
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    setup()?;

    let connector = set_tls_connecter()?;

    let addr = "example.org:443"
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| eyre!("Failed to resolve address for example.org:443"))?;

    info!("Establishing TCP connection...");
    let stream = TcpStream::connect(addr).await?;

    info!("Establishing TLS session...");
    let stream = connector.connect("example.org".try_into()?, stream).await?;

    info!("Establishing HTTP/2 connection...");
    let (mut send_req, conn) = h2::client::handshake(stream).await?;
    tokio::spawn(conn);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<color_eyre::Result<()>>(1);
    for i in 0..5 {
        let req = http::Request::builder()
            .uri("https://example.org/")
            .body(())?;
        let (res, _req_body) = send_req.send_request(req, true)?;

        let fut = async move {
            let mut body = res.await?.into_body();
            info!("{i}: received headers");
            let mut body_len = 0;
            while let Some(chunk) = body.data().await.transpose()? {
                body_len += chunk.len();
            }
            info!("{i}: received body ({body_len} bytes)");
            Ok::<_, color_eyre::Report>(())
        };

        let tx = tx.clone();
        tokio::spawn(async move { _ = tx.send(fut.await).await });
    }

    drop(tx);
    while let Some(res) = rx.recv().await {
        res?;
    }

    Ok(())
}
