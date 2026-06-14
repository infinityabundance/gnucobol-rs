//! Oracle mirror for the native XML PARSE state machine (GNURUST.MLIO.PARSE.1).
//!
//! Reads the GnuCOBOL `XML PARSE` PROCESSING-PROCEDURE trace from stdin -- one `ev=<XML-EVENT>` line per
//! invocation, then a `final=<XML-CODE>` line -- drives the native `mlio::cob_xml_parse` loop over the
//! same document, and checks the event sequence and final `XML-CODE` are byte-identical. Prints
//! `PASS=n FAIL=n`.
//!
//! GnuCOBOL 3.2's XML PARSE is itself unimplemented (it hands back the document and emits
//! START-OF-DOCUMENT -> END-OF-INPUT -> END-OF-DOCUMENT without parsing), so this native reimplementation
//! matches the *authority's* observable behaviour, not an idealised parser.

use gnucobol_rs::mlio::{cob_xml_parse, MlModule, XmlState};
use std::io::Read;

fn rust_trace(doc: &[u8]) -> (Vec<String>, i32) {
    let mut module = MlModule::default(); // xml_mode = XMLNSS (the GnuCOBOL 3.2 default)
    let mut saved: Option<XmlState> = None;
    let mut events = Vec::new();
    loop {
        let (next, ret) = cob_xml_parse(&mut module, saved, doc, None, None, 0);
        saved = next;
        if ret != 0 {
            break;
        }
        // The generated XML PARSE loop performs the PROCESSING PROCEDURE for each ret==0 step.
        events.push(String::from_utf8_lossy(&module.xml_event).into_owned());
    }
    (events, module.xml_code)
}

fn main() {
    // The fixed document the oracle sweep parses (content is irrelevant: GnuCOBOL does not parse it).
    let doc = b"<a x=\"1\">hi<b>z</b></a>";

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    let mut oracle_events = Vec::new();
    let mut oracle_final: i32 = i32::MIN;
    for line in input.lines() {
        let line = line.trim();
        if let Some(ev) = line.strip_prefix("ev=") {
            oracle_events.push(ev.trim().to_string());
        } else if let Some(code) = line.strip_prefix("final=") {
            oracle_final = code.trim().trim_start_matches('+').trim_start_matches('0').parse().unwrap_or(0);
        }
    }

    let (rust_events, rust_final) = rust_trace(doc);

    let mut pass = 0;
    let mut fail = 0;

    if rust_events == oracle_events {
        pass += 1;
        println!("PASS events {rust_events:?}");
    } else {
        fail += 1;
        println!("FAIL events: rust={rust_events:?} oracle={oracle_events:?}");
    }

    if rust_final == oracle_final {
        pass += 1;
        println!("PASS final-code {rust_final}");
    } else {
        fail += 1;
        println!("FAIL final-code: rust={rust_final} oracle={oracle_final}");
    }

    println!("PASS={pass} FAIL={fail}");
    if fail > 0 {
        std::process::exit(1);
    }
}
