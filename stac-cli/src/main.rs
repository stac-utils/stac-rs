use clap::Parser;
use stac_cli::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    std::process::exit(match stac_cli::run(args).await {
        Ok(()) => 0,
        Err(err) => err.code()
    })
}
