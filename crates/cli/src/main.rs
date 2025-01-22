use clap::Parser;
use stac_cli::Stacrs;

#[tokio::main]
async fn main() {
    let args = Stacrs::parse();
    std::process::exit(match args.run().await {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            1 // TODO make this more meaningful
        }
    })
}
