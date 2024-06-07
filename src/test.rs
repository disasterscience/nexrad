use std::{fs, path::Path};

use anyhow::Result;

use crate::{decode::decode_file, decompress::decompress_file, model::Message31Header};

#[test]
fn load_file() -> Result<()> {
    let hurricane_harvey = Path::new("resources/KCRP20170825_235733_V06_hurricane_harvey");

    let data: Vec<u8> = fs::read(hurricane_harvey)?;
    let file = decompress_file(&data)?;
    let mut datafile = decode_file(&file)?;

    // Extract a header to determine radar station characteristics
    datafile.first_volume_data().unwrap();

    // Extract elevation scans
    let elevation_scans = datafile.elevation_scans();
    assert_eq!(elevation_scans.len(), 19);

    // Ensure the elevation scans are sane
    for (elevation_number, radials) in elevation_scans {
        println!(
            "Elevation number: {}, len: {}",
            elevation_number,
            radials.len()
        );

        // Check radials are present
        assert!(!radials.is_empty());

        // Store the first elevation to compare against the rest
        // let mut previous_header: Option<Message31Header> = None;

        // Check each radial is sane
        for radial in radials {
            // Ensure reflectivity is present
            let _reflectivity = radial.reflectivity_data().unwrap();

            // Ensure radial header is sane
            let radial_header = radial.header().to_owned();

            // Ensure the group of radials are from the same elevation
            assert_eq!(elevation_number, &radial_header.elev_num());

            // // Ensure the elevation degrees are the same
            // if let Some(previous_header) = &previous_header {
            //     assert_eq!(
            //         previous_header.elev(),
            //         radial_header.elev(),
            //         "elevations mismatched: {:#?}, and {:#?}",
            //         previous_header,
            //         radial_header
            //     );
            // } else {
            //     previous_header = Some(radial_header);
            // }
        }
    }

    Ok(())
}
