//! Copies a STAC catalog from one location to the other.

use stac::{Layout, Stac, Writer};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        panic!(
            "Example script must be called with two arguments, but it was called with {}: {:?}",
            args.len() - 1,
            args
        );
    }
    let infile = &args[1];
    let outdir = &args[2];

    let (stac, _) = Stac::read(infile).unwrap();
    let layout = Layout::new(outdir);
    let writer = Writer::default();
    stac.write(&layout, &writer).unwrap();
}
