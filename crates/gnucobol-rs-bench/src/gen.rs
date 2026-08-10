//! Deterministic data generation for the performance corpus (spec 8.2).
//!
//! Every workload's input is produced by a seeded generator; expected outputs are computed
//! independently in [`expected`] (never by the candidate). All inputs are integer-exact
//! (hundredths / cents) in fixed contiguous columns, matching the COBOL programs' record
//! layouts exactly -- no embedded decimal points, no separators.

/// Deterministic xorshift64 PRNG.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.max(1))
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// Uniform in [0, n).
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// Scale -> record count (spec 8.4: the workload scales the intended operation).
pub fn record_count(scale: &str) -> usize {
    match scale {
        "small" => 500,
        "medium" => 5_000,
        "large" => 50_000,
        "stress" => 500_000,
        other => panic!("unknown scale {other:?} (small | medium | large | stress)"),
    }
}

pub fn seed_for(workload: &str, scale: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in format!("{workload}/{scale}").bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

/// Payroll: `E<id4> <hours6> <rate6> <status1>` -- hours in hundredths, rate in cents.
pub fn payroll(n: usize, seed: u64) -> (Vec<String>, Vec<(i64, i64, bool)>) {
    let mut r = Rng::new(seed);
    let mut lines = Vec::with_capacity(n);
    let mut recs = Vec::with_capacity(n);
    for i in 0..n {
        let id = format!("E{:04}", i % 10_000);
        let hours = 4000 + r.below(4001) as i64; // hundredths: 40.00..80.00
        let rate = 800 + r.below(9201) as i64; // cents: $8.00..$99.99
        let exempt = r.below(10) == 0;
        let status = if exempt { "E" } else { "N" };
        let line = format!("{id}{hours:06}{rate:06}{status}");
        lines.push(line);
        recs.push((hours, rate, exempt));
    }
    (lines, recs)
}

/// Invoice: `<item4><qty4><price8><disc2>` contiguous (price in cents, disc percent).
pub fn invoice(n: usize, seed: u64) -> (Vec<String>, Vec<(i64, i64, i64)>) {
    let mut r = Rng::new(seed);
    let mut lines = Vec::with_capacity(n);
    let mut recs = Vec::with_capacity(n);
    for i in 0..n {
        let qty = 1 + r.below(100) as i64;
        let price = 100 + r.below(100_000) as i64;
        let disc = r.below(21) as i64;
        let line = format!("{:04}{:04}{:08}{:02}", i % 10_000, qty, price, disc);
        lines.push(line);
        recs.push((qty, price, disc));
    }
    (lines, recs)
}

/// Sequential-file batch: `K<key7><amount12><code2>` contiguous (key 8 chars total, code 2).
pub fn seqfile(n: usize, seed: u64) -> (Vec<String>, Vec<(i64, bool)>) {
    let mut r = Rng::new(seed);
    let mut lines = Vec::with_capacity(n);
    let mut recs = Vec::with_capacity(n);
    for i in 0..n {
        let key = format!("K{:07}", i);
        let amount = r.below(1_000_000) as i64;
        let ok = r.below(10) != 0;
        let code = if ok { "OK" } else { "NO" };
        let line = format!("{key}{amount:012}{code}");
        lines.push(line);
        recs.push((amount, ok));
    }
    (lines, recs)
}

/// Tables: `<id5><v1 6><v2 6>` contiguous. v1 is a UNIQUE scrambled permutation of 0..n
/// (SEARCH ALL requires a strictly sortable key; the program SORTs before searching).
pub fn tables(n: usize, seed: u64) -> (Vec<String>, Vec<(i64, i64, i64)>) {
    let mut r = Rng::new(seed);
    let mut lines = Vec::with_capacity(n);
    let mut recs = Vec::with_capacity(n);
    for i in 0..n {
        let id = (i as i64) % 1000;
        // 7919 is coprime to 500000: v1 is unique for n <= 500000 and scrambled
        let v1 = ((i as i64) * 7919) % 500_000;
        let v2 = r.below(10_000) as i64;
        let line = format!("{id:05}{v1:06}{v2:06}");
        lines.push(line);
        recs.push((id, v1, v2));
    }
    (lines, recs)
}

/// Strings: `a,b,c` comma-delimited (the program UNSTRINGs).
pub fn strings(n: usize, seed: u64) -> (Vec<String>, Vec<(String, String, String)>) {
    let mut r = Rng::new(seed);
    let mut lines = Vec::with_capacity(n);
    let mut recs = Vec::with_capacity(n);
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    for _ in 0..n {
        let mk = |r: &mut Rng, len: usize| -> String {
            (0..len)
                .map(|_| CHARS[r.below(CHARS.len() as u64) as usize] as char)
                .collect()
        };
        let a = mk(&mut r, 8);
        let b = mk(&mut r, 4);
        let c = format!("{:03}", r.below(1000));
        let line = format!("{a},{b},{c}");
        lines.push(line);
        recs.push((a, b, c));
    }
    (lines, recs)
}

/// Float: `<a5><b5>` contiguous integers (the program converts to COMP-1/COMP-2).
pub fn floatwork(n: usize, seed: u64) -> (Vec<String>, Vec<(f64, f64)>) {
    let mut r = Rng::new(seed);
    let mut lines = Vec::with_capacity(n);
    let mut recs = Vec::with_capacity(n);
    for _ in 0..n {
        let a = r.below(100_000) as f64;
        let b = r.below(100_000) as f64;
        let line = format!("{:05}{:05}", a as u64, b as u64);
        lines.push(line);
        recs.push((a, b));
    }
    (lines, recs)
}

/// Report: `D<dept1><amount10>` contiguous.
pub fn reportwork(n: usize, seed: u64) -> (Vec<String>, Vec<(i64, i64)>) {
    let mut r = Rng::new(seed);
    let mut lines = Vec::with_capacity(n);
    let mut recs = Vec::with_capacity(n);
    for i in 0..n {
        let dept = (i as i64) % 5;
        let amount = r.below(100_000) as i64;
        let line = format!("D{dep}{amount:010}", dep = dept);
        lines.push(line);
        recs.push((dept, amount));
    }
    (lines, recs)
}

/// Relative-file workload: `<key10><payload10>` contiguous.
pub fn relativework(n: usize, seed: u64) -> (Vec<String>, Vec<(i64, i64)>) {
    let mut r = Rng::new(seed);
    let mut lines = Vec::with_capacity(n);
    let mut recs = Vec::with_capacity(n);
    for i in 0..n {
        let key = (i as i64) + 1;
        let payload = r.below(1_000_000) as i64;
        let line = format!("{key:010}{payload:010}");
        lines.push(line);
        recs.push((key, payload));
    }
    (lines, recs)
}
