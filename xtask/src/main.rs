//! gnucobol-rs governance tooling in Rust (replaces the lab/*.py scripts). Subcommands mirror the former
//! Python tools: `<tool> generate|check`. Run from the repo root (cwd) or set GNURUST_ROOT.
use std::process::exit;

mod ladder;
mod kani_fuzz;
mod gap;
mod coverage;
mod corpus;
mod dialect;
mod receipt;
mod trust5;
mod support;
mod trust4;
mod docs;
mod misc;
mod lineage;
mod release;
mod sweeps;
mod atlas;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tool = args.get(1).map(String::as_str).unwrap_or("");
    let cmd = args.get(2).map(String::as_str).unwrap_or("check");
    let root = std::env::var("GNURUST_ROOT").unwrap_or_else(|_| ".".into());
    if tool == "release" { std::process::exit(release::run_main(&args)); }
    let code = match tool {
        "ladder" => ladder::run(cmd, &root),
        "kani-fuzz" => kani_fuzz::run(cmd, &root),
        "gap" => gap::run(cmd, &root),
        "coverage" => coverage::run(cmd, &root),
        "corpus" => corpus::run(cmd, &root),
        "dialect" => dialect::run(cmd, &root),
        "receipt" => receipt::run(cmd, &root),
        "trust5" => trust5::run(cmd, &root),
        "support" => support::run(cmd, &root),
        "trust4" => trust4::run(cmd, &root),
        "docs" => docs::run(cmd, &root),
        "gcodes" => misc::gcodes(&root),
        "atlas-check" => misc::atlas_check(&root),
        "sweep-join" => misc::sweep_join(args.get(2).map(String::as_str).unwrap_or(""), args.get(3).map(String::as_str).unwrap_or("")),
        "sweep-extract" => sweeps::extract(args.get(2).map(String::as_str).unwrap_or(""), args.get(3).map(String::as_str).unwrap_or(""), args.get(4).map(String::as_str).unwrap_or("")),
        "sweep-remainder" => sweeps::remainder(args.get(2).map(String::as_str).unwrap_or(""), args.get(3).map(String::as_str).unwrap_or("")),
        "sweep-ordchar" => sweeps::ordchar(args.get(2).map(String::as_str).unwrap_or(""), args.get(3).map(String::as_str).unwrap_or("")),
        "atlas-file-status" => atlas::file_status(&root),
        "atlas-intrinsic" => atlas::intrinsic(&root),
        "lineage" => lineage::run(cmd, &root),
        _ => {
            eprintln!("usage: xtask <ladder> <generate|check>");
            2
        }
    };
    exit(code);
}
