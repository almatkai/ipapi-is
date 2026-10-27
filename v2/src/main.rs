#[tokio::main(worker_threads = 2)]
async fn main() {
    match tezbar_ipapi_is::Adapter::new() {
        Ok(adapter) => {
            if tezbar_extension_sdk::serve(adapter).await.is_err() {
                eprintln!("ipapi.is protocol failure");
                std::process::exit(1);
            }
        }
        Err(_) => {
            eprintln!("ipapi.is initialization failure");
            std::process::exit(1);
        }
    }
}
