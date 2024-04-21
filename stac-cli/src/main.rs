use clap::Parser;
use stac_cli::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    std::process::exit(args.execute().await)
}
