use std::{fs, path::Path};

use anyhow::Result;

use crate::{decode::decode_file, decompress::decompress_file};

#[test]
fn load_file() -> Result<()> {
    let hurricane_harvey = Path::new("resources/KCRP20170825_235733_V06_hurricane_harvey");

    let data: Vec<u8> = fs::read(hurricane_harvey)?;
    let file = decompress_file(&data)?;
    let mut datafile = decode_file(&file)?;

    // Extract a header to determine radar station characteristics
    datafile.first_volume_data().unwrap();

    // Extract elevation scans
    let elevation_scans = datafile.as_elevation_scans();
    assert_eq!(elevation_scans.len(), 19);

    // Ensure the elevation scans are sane
    for (elevation_number, radials) in &elevation_scans {
        println!(
            "Elevation number: {}, len: {}",
            elevation_number,
            radials.len()
        );

        // Check radials are present
        assert!(!radials.is_empty());

        // Store the first elevation to compare against the rest
        let mut expected_elevation = None;

        // Check each radial is sane
        for radial in radials {
            // Ensure reflectivity is present
            let _reflectivity = radial.reflectivity_data().unwrap();

            // Ensure radial header is sane
            let radial_header = radial.header();

            // Ensure the group of radials are from the same elevation
            assert_eq!(elevation_number, &radial_header.elev_num());

            // Ensure the elevation degrees are the same
            if let Some(expected_elevation) = expected_elevation {
                assert_eq!(expected_elevation, radial_header.elev());
            } else {
                expected_elevation = Some(radial_header.elev());
            }
        }
    }

    Ok(())
}
