use clap::Parser;
use stac_cli::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command.execute().await {
        Ok(()) => return,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(err.return_code())
        }
    }
}
