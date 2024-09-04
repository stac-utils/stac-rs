use clap::Parser;
use stac_cli::Args;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    if std::env::var("STACRS_TRACING").is_ok() {
        tracing_subscriber::fmt::init();
    }

    let args = Args::parse();
    std::process::exit(match stac_cli::run(args).await {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            err.code()
        }
    })
}
