use std::path::Path;

use anyhow::Result;

use crate::DataFile;

#[test]
fn load_file() -> Result<()> {
    let hurricane_harvey = Path::new("resources/KCRP20170825_235733_V06_hurricane_harvey");

    let datafile = DataFile::new(hurricane_harvey)?;

    // Extract a header to determine radar station characteristics
    datafile.first_volume_data().expect("No volume data found");

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

        // Check each radial is sane
        for radial in radials {
            // Ensure reflectivity is present
            let _reflectivity = radial.reflectivity_data().unwrap();

            // Ensure radial header is sane
            let radial_header = radial.header().to_owned();

            // Ensure the group of radials are from the same elevation
            assert_eq!(elevation_number, &radial_header.elev_num());
        }
    }

    Ok(())
}
