//! QA ONLY. The packaged executable has no producer/certificate/environment override.
#[tokio::main(worker_threads = 2)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let certificate = std::fs::read(&args[1]).unwrap();
    let proxy: url::Url = args[2].parse().unwrap();
    assert_eq!(proxy.scheme(), "http");
    assert_eq!(proxy.host_str(), Some("127.0.0.1"));
    let client = reqwest::Client::builder()
        .https_only(true)
        .no_proxy()
        .proxy(reqwest::Proxy::https(proxy.as_str()).unwrap())
        .add_root_certificate(reqwest::Certificate::from_pem(&certificate).unwrap())
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap();
    tezbar_extension_sdk::serve(tezbar_ipapi_is::Adapter::qa_client(client))
        .await
        .unwrap();
}
