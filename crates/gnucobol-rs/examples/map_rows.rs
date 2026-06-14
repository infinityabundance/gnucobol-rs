//! Rust mirror for the filename-mapping sweep (`GNURUST.FILEIO.MAPPING.1`). The oracle program OPENs
//! files whose ASSIGN names are resolved through the environment (`DD_*` / `COB_FILE_PATH`); this checks
//! that [`gnucobol_rs::fileio::cob_chk_file_mapping`] (reading the same env) points at the exact paths
//! the oracle created the files at. PASS=n FAIL=n.
use gnucobol_rs::fileio::cob_chk_file_mapping;

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    // "MAPA" -> DD_MAPA (absolute); "bmap.dat" -> COB_FILE_PATH/bmap.dat (no DD set).
    for name in ["MAPA", "bmap.dat"] {
        let mapped = cob_chk_file_mapping(name.as_bytes());
        let path = String::from_utf8_lossy(&mapped).into_owned();
        if std::path::Path::new(&path).is_file() {
            pass += 1;
        } else {
            println!("{name} FAIL mapped={path} (oracle did not create a file there)");
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
