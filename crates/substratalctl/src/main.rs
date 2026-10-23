use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = substratalctl::Cli::parse();
    if let Err(e) = substratalctl::run(cli).await {
        eprintln!("substratalctl: {e:#}");
        if let Some(id) = e
            .downcast_ref::<substratal_core::Error>()
            .and_then(|e| e.request_id())
        {
            eprintln!("request id: {id}");
        }
        std::process::exit(1);
    }
}
