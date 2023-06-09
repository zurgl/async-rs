use color_eyre::Report;
use std::sync::Arc;
use std::{io, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::{
    rustls::{self, OwnedTrustAnchor},
    TlsConnector,
};
use tracing::info;

pub async fn fetch_thing(name: &str) -> Result<(), Report> {
    let addr: SocketAddr = ([1, 1, 1, 1], 443).into();
    let socket = TcpStream::connect(addr).await?;

    let mut root_cert_store = rustls::RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));

    let domain = rustls::ServerName::try_from("one.one.one.one")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?;

    let mut socket = connector.connect(domain, socket).await?;

    socket.write_all(b"GET / HTTP/1.1\r\n").await?;
    socket.write_all(b"Host: one.one.one.one\r\n").await?;
    socket.write_all(b"User-Agent: cool-bear\r\n").await?;
    socket.write_all(b"Connection: close\r\n").await?;
    socket.write_all(b"\r\n").await?;

    let mut response = String::with_capacity(256);
    socket.read_to_string(&mut response).await?;

    let status = response.lines().next().unwrap_or_default();
    info!(%status, %name, "Got response!");

    Ok(())
}
