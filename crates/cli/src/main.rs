use clap::Parser;
use stac_cli::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.log_level())
        .init();
    std::process::exit(match args.run().await {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            err.code()
        }
    })
}
