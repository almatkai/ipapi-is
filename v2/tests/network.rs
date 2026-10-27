use std::{sync::Arc, time::Duration};
use tezbar_ipapi_is::{domain::MAX_BYTES, network};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tokio_rustls::{
    rustls::{
        self,
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
    },
    TlsAcceptor,
};

async fn server(
    response: Vec<u8>,
    trusted: bool,
) -> (reqwest::Client, url::Url, tokio::task::JoinHandle<Vec<u8>>) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der()));
    let cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![CertificateDer::from(cert.cert.der().to_vec())], key)
        .unwrap();
    let tls = TlsAcceptor::from(Arc::new(cfg));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let job = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let Ok(mut stream) = tls.accept(tcp).await else {
            return Vec::new();
        };
        let mut request = vec![0; 8192];
        let n = stream.read(&mut request).await.unwrap_or(0);
        request.truncate(n);
        let _ = stream.write_all(&response).await;
        let _ = stream.shutdown().await;
        request
    });
    let mut b = reqwest::Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(2));
    if trusted {
        b = b.add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap());
    }
    (
        b.build().unwrap(),
        format!("https://localhost:{port}/?q=192.0.2.1")
            .parse()
            .unwrap(),
        job,
    )
}
fn response(status: &str, body: &str, extra: &str) -> Vec<u8> {
    format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n{body}",body.len()).into_bytes()
}
#[tokio::test]
async fn synthetic_tls_success_and_validation() {
    let (c, u, j) = server(
        response("200 OK", "{\"ip\":\"192.0.2.1\",\"elapsed_ms\":0}", ""),
        true,
    )
    .await;
    assert_eq!(network::receive(&c, u).await.unwrap()["ip"], "192.0.2.1");
    let request = String::from_utf8(j.await.unwrap()).unwrap();
    assert!(request.contains("GET /?q=192.0.2.1"));
    assert!(!request.to_lowercase().contains("authorization"));
    let (c, u, j) = server(response("200 OK", "{}", ""), false).await;
    assert!(network::receive(&c, u).await.is_err());
    j.await.unwrap();
}
#[tokio::test]
async fn statuses_partial_json_size_and_no_raw_echo() {
    for (status, body, extra) in [
        ("401 Unauthorized", "secret synthetic key", ""),
        ("403 Forbidden", "secret synthetic key", ""),
        ("429 Too Many Requests", "secret synthetic key", ""),
        ("500 Error", "secret synthetic key", ""),
        (
            "302 Found",
            "secret synthetic key",
            "Location: https://evil.invalid/?key=synthetic\r\n",
        ),
        ("200 OK", "{broken", ""),
        ("200 OK", "{}", ""),
    ] {
        let (c, u, j) = server(response(status, body, extra), true).await;
        let e = network::receive(&c, u).await.unwrap_err();
        assert!(!e.contains("secret") && !e.contains("synthetic") && !e.contains("evil.invalid"));
        j.await.unwrap();
    }
    let bytes = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        MAX_BYTES + 1
    )
    .into_bytes();
    let (c, u, j) = server(bytes, true).await;
    assert!(network::receive(&c, u)
        .await
        .unwrap_err()
        .contains("size limit"));
    j.await.unwrap();
    let (c, u, j) = server(
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 400\r\n\r\n{}"
            .to_vec(),
        true,
    )
    .await;
    assert!(network::receive(&c, u).await.is_err());
    j.await.unwrap();
}
#[tokio::test]
async fn cleartext_refused_without_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let u = format!("http://{}/", listener.local_addr().unwrap())
        .parse()
        .unwrap();
    assert!(network::receive(&network::client().unwrap(), u)
        .await
        .is_err());
    assert!(
        tokio::time::timeout(Duration::from_millis(20), listener.accept())
            .await
            .is_err()
    );
}
