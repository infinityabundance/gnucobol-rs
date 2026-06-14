//! Rust mirror for the verb-precondition sweep (`GNURUST.FILEIO.VERB.1`). Reads `<tag>=<status>` (the
//! oracle FILE STATUS when a verb is attempted in the wrong mode) and compares to the precondition
//! decision of [`gnucobol_rs::fileio`]'s `cob_*` verbs. PASS=n FAIL=n.
use gnucobol_rs::fileio::{
    cob_delete, cob_read, cob_read_next, cob_rewrite, cob_write, AccessMode, Organization, OpenMode,
};
use std::io::BufRead;

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let oracle = oracle.trim();
        // each scenario mirrors the exact (open, access, ...) the oracle program uses; a passed
        // precondition (None) would proceed to the handler, shown here as "00".
        let mine = match tag {
            "w_input_seq" => cob_write(OpenMode::Input, AccessMode::Sequential, 4, 4, 4),
            "r_output" => cob_read(OpenMode::Output, false, false, false, false, false, false),
            "rw_input" => cob_rewrite(OpenMode::Input, AccessMode::Sequential, Organization::Sequential, true, 4, 4),
            "rw_io_noread" => cob_rewrite(OpenMode::Io, AccessMode::Sequential, Organization::Sequential, false, 4, 4),
            "del_input" => cob_delete(OpenMode::Input, AccessMode::Random, false),
            "w_input_rel" => cob_write(OpenMode::Input, AccessMode::Random, 4, 4, 4),
            "rn_output" => cob_read_next(OpenMode::Output, false, false, false, false, false),
            _ => continue,
        }
        .unwrap_or("00");
        if mine == oracle {
            pass += 1;
        } else {
            println!("{tag} FAIL mine={mine} oracle={oracle}");
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
