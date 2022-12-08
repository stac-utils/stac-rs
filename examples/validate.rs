use stac::{Error, Validate};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!(
            "ERROR: wrong number of arguments (expected 2, got {})",
            args.len()
        );
        eprintln!(
            "USAGE: cargo run --example validate --feature jsonschema example/invalid-item.json"
        );
        std::process::exit(1)
    }
    let value = stac::read(&args[1]).unwrap();
    match value.validate() {
        Ok(()) => println!("OK: {} is valid STAC!", args[1]),
        Err(errors) => {
            for error in errors {
                match error {
                    Error::ValidationError(e) => {
                        println!("VALIDATION ERROR at {}: {}", e.instance_path, e);
                    }
                    _ => println!("ERROR: {}", error),
                }
            }
            std::process::exit(1)
        }
    }
}
