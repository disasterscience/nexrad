//! examples/decode
//!
//! This example loads a data file and decodes it.
//!
//! Usage: cargo run --example decode -- <file>
//!

use std::env;
use std::path::Path;

use anyhow::Result;
use nexrad::DataFile;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: cargo run --example decode -- <file>");
    }

    let file = Path::new(&args[1]);
    let datafile = DataFile::new(file)?;

    println!(
        "Decoded file with {} elevations.",
        datafile.elevation_scans().len()
    );

    Ok(())
}
