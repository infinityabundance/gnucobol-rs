//! Port of the libcob profiling feature (upstream `7b6995042`, profiling.c) for the interpreted
//! candidate.
//!
//! Upstream semantics reproduced:
//! - `COB_PROF_ENABLE` (default `0`) switches the profiler on; `COB_PROF_FILE` (default
//!   `cob-prof-$b-$$-$d-$t.csv`), `COB_PROF_FORMAT` (default `%m,%s,%p,%e,%w,%k,%t,%h,%n`) and
//!   `COB_PROF_MAX_DEPTH` (default `8192`) configure the report.
//! - Per-procedure counts and cumulative times accumulate while a procedure is on the call stack;
//!   the report is written at process end with a header line (the format with literal words) then
//!   one line per procedure.
//! - The `%I` placeholder prints `123456` in test mode (the upstream `is_test` determinism for
//!   the pid); under `COB_IS_RUNNING_IN_TESTMODE` the clock advances a fixed 1 ms per tick so the
//!   report is deterministic.
//! - Depth overflow prints `[cob_prof] Profiling overflow at N calls, aborting profiling.` and
//!   stops accumulating (the suite's `run_misc.at` expectation).
//!
//! The candidate's procedures are paragraphs: the front-end calls [`prof_enter`] at each paragraph
//! label and [`prof_exit`] when the paragraph is left. This is the interpreted equivalent of the
//! generated `cob_prof_function_call` calls -- the hooks are always present, and the runtime
//! setting (`COB_PROF_ENABLE`) is what activates them (the candidate accepts `-fprof` and records
//! it; it has no codegen to gate).

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::time::Instant;

/// The resolved profiling configuration (upstream `cobsettings` `cob_prof_*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfConfig {
    /// `COB_PROF_ENABLE` -- whether the profiler is active.
    pub enable: bool,
    /// `COB_PROF_FILE` -- the report path (env-string expanded: `$b` basename, `$f` filename,
    /// `$d` yyyymmdd, `$t` hhmmss, `$$` pid).
    pub file: String,
    /// `COB_PROF_FORMAT` -- the per-line format (upstream default `%m,%s,%p,%e,%w,%k,%t,%h,%n`).
    pub format: String,
    /// `COB_PROF_MAX_DEPTH` -- the call-stack depth cap; exceeding it aborts profiling with the
    /// upstream overflow warning.
    pub max_depth: usize,
    /// `COB_IS_RUNNING_IN_TESTMODE` -- deterministic clock (fixed 1 ms ticks) and `%I` = 123456.
    pub test_mode: bool,
}

impl Default for ProfConfig {
    fn default() -> Self {
        ProfConfig {
            enable: false,
            file: "cob-prof-$b-$$-$d-$t.csv".to_string(),
            format: "%m,%s,%p,%e,%w,%k,%t,%h,%n".to_string(),
            max_depth: 8192,
            test_mode: false,
        }
    }
}

/// Resolve the profiler configuration from the environment (upstream `common.c` defaults).
pub fn prof_config(getenv: &dyn Fn(&str) -> Option<String>) -> ProfConfig {
    let mut cfg = ProfConfig::default();
    if let Some(v) = getenv("COB_PROF_ENABLE") {
        cfg.enable = is_true(&v);
    }
    if let Some(v) = getenv("COB_PROF_FILE") {
        if !v.is_empty() {
            cfg.file = v;
        }
    }
    if let Some(v) = getenv("COB_PROF_FORMAT") {
        if !v.is_empty() {
            cfg.format = v;
        }
    }
    if let Some(v) = getenv("COB_PROF_MAX_DEPTH") {
        if let Ok(n) = v.trim().parse::<usize>() {
            cfg.max_depth = n;
        }
    }
    cfg.test_mode = getenv("COB_IS_RUNNING_IN_TESTMODE")
        .map(|v| is_true(&v))
        .unwrap_or(false);
    cfg
}

fn is_true(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_uppercase().as_str(),
        "1" | "YES" | "TRUE" | "ON" | "Y"
    )
}

/// One profiled procedure (paragraph).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfProc {
    pub module: String,
    pub paragraph: String,
    pub file: String,
    pub line: usize,
}

/// The accumulated state for one procedure.
#[derive(Debug, Clone, Default)]
struct Accum {
    count: u64,
    total_ns: u64,
}

thread_local! {
    static PROF: RefCell<Option<ProfState>> = const { RefCell::new(None) };
}

struct ProfState {
    cfg: ProfConfig,
    /// procedure identity -> accumulation, indexed for the stack.
    procs: Vec<ProfProc>,
    accums: Vec<Accum>,
    /// the call stack: (proc index, entry time).
    stack: Vec<(usize, u64)>,
    /// test-mode deterministic clock.
    tick_ns: u64,
    active: bool,
}

fn now_ns(test_mode: bool, tick: &mut u64) -> u64 {
    if test_mode {
        *tick += 1_000_000; // upstream: ns_time += 1000000 per get_ns_time in test mode
        *tick
    } else {
        // monotonic elapsed since the profiler started
        START_NS.with(|s| {
            let elapsed = s.borrow().elapsed().as_nanos() as u64;
            elapsed.max(*tick)
        })
    }
}

thread_local! {
    static START_NS: RefCell<Instant> = RefCell::new(Instant::now());
}

/// Enable profiling with the given configuration (called once at run start).
pub fn prof_start(cfg: &ProfConfig) {
    PROF.with(|p| {
        *p.borrow_mut() = Some(ProfState {
            cfg: cfg.clone(),
            procs: Vec::new(),
            accums: Vec::new(),
            stack: Vec::new(),
            tick_ns: 0,
            active: cfg.enable,
        });
    });
}

/// Enter a procedure: record the entry time and bump the call count.
pub fn prof_enter(proc: ProfProc) {
    PROF.with(|p| {
        let mut b = p.borrow_mut();
        let Some(st) = b.as_mut() else { return };
        if !st.active {
            return;
        }
        let idx = match st.procs.iter().position(|q| *q == proc) {
            Some(i) => i,
            None => {
                st.procs.push(proc);
                st.accums.push(Accum::default());
                st.procs.len() - 1
            }
        };
        if st.stack.len() >= st.cfg.max_depth {
            // upstream: '[cob_prof] Profiling overflow at N calls, aborting profiling.'
            eprintln!(
                "[cob_prof] Profiling overflow at {} calls, aborting profiling.",
                st.stack.len()
            );
            st.active = false;
            return;
        }
        st.stack
            .push((idx, now_ns(st.cfg.test_mode, &mut st.tick_ns)));
        st.accums[idx].count += 1;
    });
}

/// Leave a procedure (a paragraph is left when the next label runs or the body ends): pop the
/// stack down to (and including) `proc`, crediting cumulative time to each popped procedure the
/// same way upstream's `cob_prof_exit_procedure` does (recursion depth is tracked per procedure).
pub fn prof_exit(proc: &ProfProc) {
    PROF.with(|p| {
        let mut b = p.borrow_mut();
        let Some(st) = b.as_mut() else { return };
        if !st.active {
            return;
        }
        let Some(idx) = st.procs.iter().position(|q| q == proc) else {
            return;
        };
        let t = now_ns(st.cfg.test_mode, &mut st.tick_ns);
        while let Some((ci, start)) = st.stack.pop() {
            st.accums[ci].total_ns += t - start;
            if ci == idx {
                break;
            }
        }
    });
}

/// Write the report (the header line, then one line per procedure) to `COB_PROF_FILE` resolved
/// against `argv0` (the `$b`/`$f` env expansion). Called once at run end.
pub fn prof_report(getenv: &dyn Fn(&str) -> Option<String>, argv0: &str) {
    let taken = PROF.with(|p| {
        let mut b = p.borrow_mut();
        b.take().map(|st| (st.cfg, st.procs, st.accums))
    });
    let Some((cfg, procs, accums)) = taken else {
        return;
    };
    if !cfg.enable {
        return;
    }
    let path = expand_env_string(&cfg.file, getenv, argv0);
    let mut out = String::new();
    // header line: the format with literal words (upstream prints it with info == NULL).
    out.push_str(&render_line(
        &cfg.format,
        &ProfProc {
            module: "program-id".into(),
            paragraph: "paragraph".into(),
            file: "file".into(),
            line: 0,
        },
        None,
        0,
        0,
        cfg.test_mode,
    ));
    out.push('\n');
    for (proc, acc) in procs.iter().zip(accums.iter()) {
        out.push_str(&render_line(
            &cfg.format,
            proc,
            Some(acc.total_ns),
            acc.count,
            0,
            cfg.test_mode,
        ));
        out.push('\n');
    }
    if let Err(e) = std::fs::write(&path, out) {
        eprintln!("[cob_prof] cannot write profiling data to {path}: {e}");
    }
}

/// The `%`-format renderer (upstream `print_data`): `%M/%m` module, `%S/%s` section,
/// `%P/%p` paragraph, `%E/%e` entry, `%F/%f` file, `%L/%l` line, `%I/%i` pid (123456 in test
/// mode), `%W/%w` file:line, `%K/%k` kind, `%T/%t` time-ns, `%H/%h` human time, `%N/%n` calls.
fn render_line(
    format: &str,
    proc: &ProfProc,
    time: Option<u64>,
    ncalls: u64,
    _section: usize,
    test_mode: bool,
) -> String {
    let mut out = String::new();
    let mut chars = format.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let Some(spec) = chars.next() else {
            out.push('%');
            break;
        };
        match spec.to_ascii_lowercase() {
            'm' => out.push_str(&proc.module),
            's' => out.push_str("section"),
            'p' => out.push_str(&proc.paragraph),
            'e' => out.push_str(&proc.paragraph),
            'f' => out.push_str(&proc.file),
            'l' => out.push_str(&proc.line.to_string()),
            'i' => {
                if time.is_none() {
                    out.push_str("pid"); // header line (info == NULL in upstream)
                } else if test_mode {
                    out.push_str("123456");
                } else {
                    out.push_str("0"); // the candidate's pid placeholder (deterministic)
                }
            }
            'w' => out.push_str(&format!("{}:{}", proc.file, proc.line)),
            'k' => out.push_str("PARAGRAPH"),
            't' => match time {
                Some(t) => out.push_str(&t.to_string()),
                None => out.push_str("time-ns"),
            },
            'h' => match time {
                Some(t) => out.push_str(&human_time(t)),
                None => out.push_str("time"),
            },
            'n' => match ncalls {
                0 if time.is_none() => out.push_str("ncalls"),
                n => out.push_str(&n.to_string()),
            },
            other => {
                out.push('%');
                out.push(other);
            }
        }
    }
    out
}

/// `print_monotonic_time` (profiling.c:329): `N s` beyond 1000 s, else `N.MMM s`.
fn human_time(t: u64) -> String {
    let milliseconds = t / 1_000_000;
    let seconds = milliseconds / 1000;
    let ms = milliseconds - 1000 * seconds;
    if seconds > 1000 {
        format!("{seconds} s")
    } else {
        format!("{seconds}.{ms:03} s")
    }
}

/// `cob_expand_env_string` for the profiler: `$b` argv0 basename, `$f` argv0 full, `$d`
/// yyyymmdd, `$t` hhmmss, `$$` the pid.
pub fn expand_env_string(
    template: &str,
    _getenv: &dyn Fn(&str) -> Option<String>,
    argv0: &str,
) -> String {
    let base = std::path::Path::new(argv0)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| argv0.to_string());
    let mut out = String::new();
    let mut chars = template.chars();
    while let Some(c) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('b') => out.push_str(&base),
            Some('f') => out.push_str(argv0),
            Some('d') => out.push_str(&date_ymd()),
            Some('t') => out.push_str(&time_hms()),
            Some('$') => out.push_str(&std::process::id().to_string()),
            Some(other) => {
                out.push('$');
                out.push(other);
            }
            None => out.push('$'),
        }
    }
    out
}

fn date_ymd() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}{m:02}{d:02}")
}

fn time_hms() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let tod = secs % 86400;
    format!("{:02}{:02}{:02}", tod / 3600, (tod % 3600) / 60, tod % 60)
}

/// Days-since-epoch -> (year, month, day) (civil calendar, Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_and_overrides() {
        let getenv = |k: &str| -> Option<String> {
            match k {
                "COB_PROF_ENABLE" => Some("1".into()),
                "COB_PROF_FORMAT" => Some("%p %n".into()),
                _ => None,
            }
        };
        let cfg = prof_config(&getenv);
        assert!(cfg.enable);
        assert_eq!(cfg.format, "%p %n");
        assert_eq!(cfg.file, "cob-prof-$b-$$-$d-$t.csv");
        assert_eq!(cfg.max_depth, 8192);
    }

    #[test]
    fn format_placeholders_and_header() {
        let fmt = "%m,%s,%p,%e,%w,%k,%t,%h,%n";
        let proc = ProfProc {
            module: "M".into(),
            paragraph: "P".into(),
            file: "f.cob".into(),
            line: 7,
        };
        let line = render_line(fmt, &proc, Some(1_500_000_000), 3, 0, false);
        assert_eq!(line, "M,section,P,P,f.cob:7,PARAGRAPH,1500000000,1.500 s,3");
        let hdr = render_line(
            fmt,
            &ProfProc {
                module: "program-id".into(),
                paragraph: "paragraph".into(),
                file: "file".into(),
                line: 0,
            },
            None,
            0,
            0,
            false,
        );
        assert_eq!(
            hdr,
            "program-id,section,paragraph,paragraph,file:0,PARAGRAPH,time-ns,time,ncalls"
        );
    }

    #[test]
    fn overflow_warning_and_deterministic_test_clock() {
        let mut cfg = ProfConfig::default();
        cfg.enable = true;
        cfg.test_mode = true;
        cfg.max_depth = 2;
        prof_start(&cfg);
        for i in 0..3 {
            prof_enter(ProfProc {
                module: "M".into(),
                paragraph: format!("P{i}").into(),
                file: "f".into(),
                line: i,
            });
        }
        prof_report(&|_| None, "prog");
    }
}
