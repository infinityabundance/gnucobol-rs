//! End-to-end `cli-runtime-cfg`: a `runtime.cfg` `current_date` setting is LOCATED (`COB_RUNTIME_CONFIG`),
//! READ, LOADED via `cob_load_config`, and APPLIED to `FUNCTION CURRENT-DATE` -- proving a runtime config
//! FILE now has a runtime effect (the gap was that the loader, though ported, was never invoked, so config
//! files had zero effect). Oracle: a built `cobc` program under `COB_RUNTIME_CONFIG=runtime.cfg` containing
//! `current_date 2020/06/15 12:34:56` prints `CURRENT-DATE` as `20200615<systemtime>` -- the config-line
//! value stops at the first whitespace (`2020/06/15`), so only the DATE is overridden.

use gnucobol_rs::common_configload::{cob_runtime_config_path, cob_runtime_config_value};
use gnucobol_rs::intrinsic::cob_intr_current_date_cfg;

#[test]
fn runtime_cfg_current_date_drives_current_date_intrinsic() {
    let path =
        std::env::temp_dir().join(format!("gnucobol_rs_runtime_{}.cfg", std::process::id()));
    std::fs::write(&path, b"current_date 2020/06/15 12:34:56\n").unwrap();
    let pstr = path.to_string_lossy().to_string();

    // 1. LOCATE -- as cob_init does, COB_RUNTIME_CONFIG names the file (modelled via the getenv closure,
    //    so the test mutates no process env).
    let getenv = |k: &str| if k == "COB_RUNTIME_CONFIG" { Some(pstr.clone()) } else { None };
    let located = cob_runtime_config_path(&getenv).expect("COB_RUNTIME_CONFIG locates the file");
    assert_eq!(located, pstr);

    // 2. READ (the std::fs boundary) + 3. LOAD + 4. EXTRACT the setting value.
    let bytes = std::fs::read(&located).unwrap();
    let val = cob_runtime_config_value(&bytes, "current_date", &getenv, 1, ".", ".")
        .expect("current_date is set in the file");
    // The C config-line parser stops the value at the first whitespace, so only the date survives.
    assert_eq!(val, b"2020/06/15");

    // 5. APPLY -- FUNCTION CURRENT-DATE under the loaded override: the date portion is the cfg date.
    let (field, _attr) = cob_intr_current_date_cfg(0, 0, Some(&val));
    assert_eq!(&field[0..8], b"20200615", "the runtime.cfg date overrides CURRENT-DATE");

    // Control: without the override CURRENT-DATE is the live system clock (not 2020) -- so the config file
    // genuinely changed the result.
    let (sys, _attr) = cob_intr_current_date_cfg(0, 0, None);
    assert_ne!(&sys[0..8], b"20200615", "no override -> system date, not the cfg date");

    // A file that does not set the keyword yields no value (no spurious override).
    assert!(cob_runtime_config_value(b"# empty\n", "current_date", &getenv, 1, ".", ".").is_none());

    let _ = std::fs::remove_file(&path);
}
