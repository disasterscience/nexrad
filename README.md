# NEXRAD

[![Crate](https://img.shields.io/crates/v/nexrad.svg)](https://crates.io/crates/nexrad)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/rust_ci.yml/badge.svg?branch=master)](https://github.com/danielway/nexrad/actions/workflows/rust_ci.yml)
[![Rust CD](https://github.com/danielway/nexrad/actions/workflows/rust_cd.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/rust_cd.yml)

Download and decode functions for NEXRAD radar data.

## Summary

This library provides functions to download and decode NEXRAD Level II data from AWS uploaded in near real-time by NOAA.

![Example image](examples/render_kdmx_030522_1730.png)

_An EF4 tornado near Des Moines, IA on March 5, 2022 rendered using this library's "render" example._

## Decoding and Decompression

A data file consists of binary-encoded messages containing sweep data. It is often compressed with bzip2 and must be decompressed prior to decoding. Here is an example of decoding a
file:

```rust
use anyhow::Result;
use std::path::Path;
use nexrad::DataFile;

fn main() -> Result<()> {
    let hurricane_harvey = Path::new("resources/KCRP20170825_235733_V06_hurricane_harvey");
    let datafile = DataFile::new(hurricane_harvey)?;
    println!("Decoded file with {} elevations.", datafile.elevation_scans().len());
    Ok(())
}
```

## Downloading

The `download` feature may be enabled to download NEXRAD Level II data from AWS. For more information on this data
source, see the [Registry of Open Data](https://registry.opendata.aws/noaa-nexrad/)'s page. As the radar rotates or
"sweeps" it collects data which is aggregated into messages. The messages are broken into 5-minute "chunks" before being
compressed and uploaded to AWS.

The data is organized by site and date. Here is an example of downloading the first file for April 6, 2023 from KDMX:

```rust
use anyhow::Result;
use std::path::Path;
use chrono::NaiveDate;
use nexrad::download::{list_files, download_file};
use nexrad::file_metadata::is_compressed;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    let metas = list_files(site, &date).await?;
    if let Some(meta) = metas.first() {
        println!("Downloading {}...", meta.identifier());
        let downloaded_file = download_file(meta).await?;

        println!("Data file size (bytes): {}", downloaded_file.len());
        println!("Data file is compressed: {}", is_compressed(&downloaded_file));
    } else {
        println!("No files found for the specified date/site to download.");
    }
    Ok(())
}
```

In this example, `list_files` is being used to query which files are available for the specified site and date, and
`download_file` is used to download the contents of the first file. The downloaded file will need to be decompressed and
decoded before the data can be inspected.

## Rendering

A downloaded file can be rendered to an image using the `render` example. Here is an example usage and the result:

```bash
cargo run --example render KDMX20220305_233003_V06
```

## Acknowledgements

I consulted the following resources when developing this library:

<https://github.com/bwiggs/go-nexrad>

<https://trmm-fc.gsfc.nasa.gov/trmm_gv/software/rsl/>
