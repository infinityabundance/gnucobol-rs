//! Independent expected-output calculators (spec 8.3).
//!
//! Every workload's expected output is computed here from the same generated data the COBOL
//! program reads -- with integer-exact decimal arithmetic (cents/hundredths) and the exact
//! edited-field widths the programs emit. The candidate never generates its own expected output.

use crate::gen;

/// Right-align an integer into `width` chars (the Z-suppression / 9 picture width).
fn pad(n: i128, width: usize) -> String {
    format!("{n:>width$}")
}

/// Format integer cents into the edited form PIC Z(a)9.99 (a+3 chars: a Z's + 1 integer digit
/// + ".99").
fn cents_edited(cents: i128, zs: usize) -> String {
    let whole = cents / 100;
    let frac = (cents % 100).abs();
    format!("{}.{:02}", pad(whole, zs + 1), frac)
}

pub fn payroll(n: usize, scale: &str) -> (String, String, String) {
    let (lines, recs) = gen::payroll(n, gen::seed_for("payroll", scale));
    let mut out = String::new();
    let mut tg: i128 = 0;
    let mut tt: i128 = 0;
    let mut tn: i128 = 0;
    for (i, (hours, rate, exempt)) in recs.iter().enumerate() {
        // COBOL ROUNDED = decimal half-away-from-zero; integer rounding is exact for these
        // positive values: (x + 50) / 100
        let gross = (*hours as i128 * *rate as i128 + 50) / 100;
        let pct = if *exempt { 10 } else { 22 };
        let tax = (gross * pct + 50) / 100;
        let net = gross - tax;
        tg += gross;
        tt += tax;
        tn += net;
        out.push_str(&format!(
            "E{:04} {} {} {}\n",
            i % 10_000,
            cents_edited(gross, 8),
            cents_edited(tax, 8),
            cents_edited(net, 8)
        ));
    }
    out.push_str(&format!(
        "TOTALS {:09} {}\n",
        recs.len(),
        cents_edited(tg, 11)
    ));
    out.push_str(&format!("TAX {}\n", cents_edited(tt, 11)));
    out.push_str(&format!("NET {}\n", cents_edited(tn, 11)));
    (lines.join("\n") + "\n", out, String::new())
}

pub fn invoice(n: usize, scale: &str) -> (String, String, String) {
    let (lines, recs) = gen::invoice(n, gen::seed_for("invoice", scale));
    let mut out = String::new();
    let mut tt: i128 = 0;
    let mut ttax: i128 = 0;
    for (i, (qty, price, disc)) in recs.iter().enumerate() {
        let line_total = (*qty as i128) * (*price as i128);
        let disc_c = (line_total * *disc as i128 + 50) / 100;
        let taxable = line_total - disc_c;
        let tax = (taxable * 14 + 50) / 100;
        tt += line_total;
        ttax += tax;
        out.push_str(&format!(
            "{:04} {} {} {} {}\n",
            i % 10_000,
            pad(line_total, 16),
            pad(disc_c, 16),
            pad(taxable, 16),
            pad(tax, 16)
        ));
    }
    out.push_str(&format!(
        "TOTAL {:09} {} {}\n",
        recs.len(),
        pad(tt, 16),
        pad(ttax, 16)
    ));
    (lines.join("\n") + "\n", out, String::new())
}

pub fn seqfile(n: usize, scale: &str) -> (String, String, String) {
    let (lines, recs) = gen::seqfile(n, gen::seed_for("seqfile", scale));
    let mut out = String::new();
    let mut balance: i128 = 0;
    let mut vs: i128 = 0;
    let mut is_: i128 = 0;
    let mut vn = 0i128;
    let mut inn = 0i128;
    for (i, (amount, ok)) in recs.iter().enumerate() {
        if *ok {
            vs += *amount as i128;
            vn += 1;
            balance += *amount as i128;
        } else {
            is_ += *amount as i128;
            inn += 1;
        }
        out.push_str(&format!("K{:07} {:012} {}\n", i, amount, pad(balance, 16)));
    }
    out.push_str(&format!("VALID {:09} {}\n", vn, pad(vs, 16)));
    out.push_str(&format!("INVALID {:09} {}\n", inn, pad(is_, 16)));
    (lines.join("\n") + "\n", out, String::new())
}

pub fn tables(n: usize, scale: &str) -> (String, String, String) {
    let (lines, recs) = gen::tables(n, gen::seed_for("tables", scale));
    // the program SORTs by v1 then loads the table; the searched keys are the v1 of every
    // 10th SORTED row (the program's SEARCH ALL always finds them: the key is IN the table)
    let mut sorted: Vec<(i64, i64, i64)> = recs.clone();
    sorted.sort_by_key(|(_, v1, _)| *v1);
    let rows = sorted.len() as i128;
    let tot_v1: i128 = sorted.iter().map(|(_, v1, _)| *v1 as i128).sum();
    let tot_v2: i128 = sorted.iter().map(|(_, _, v2)| *v2 as i128).sum();
    let mut found_n = 0i128;
    let mut fv1: i128 = 0;
    let mut fv2: i128 = 0;
    for (idx, (_id, v1, v2)) in sorted.iter().enumerate() {
        if (idx + 1) % 10 == 0 {
            found_n += 1;
            fv1 += *v1 as i128;
            fv2 += *v2 as i128;
        }
    }
    let out = format!(
        "FOUND {:09} {} {}\nMISSED {:09}\nTABLE {:09} {} {}\n",
        found_n,
        pad(fv1, 15),
        pad(fv2, 15),
        0,
        rows,
        pad(tot_v1, 15),
        pad(tot_v2, 15)
    );
    (lines.join("\n") + "\n", out, String::new())
}

pub fn strings(n: usize, scale: &str) -> (String, String, String) {
    let (lines, recs) = gen::strings(n, gen::seed_for("strings", scale));
    let mut out = String::new();
    for (a, b, c) in &recs {
        let digits = a.chars().filter(|ch| ch.is_ascii_digit()).count();
        let head3 = &a[..3.min(a.len())];
        out.push_str(&format!("{a}|{b}|{c}|{digits:04}|{head3}\n"));
    }
    (lines.join("\n") + "\n", out, String::new())
}

pub fn floatwork(n: usize, scale: &str) -> (String, String, String) {
    let (lines, recs) = gen::floatwork(n, gen::seed_for("float", scale));
    let mut out = String::new();
    for (a, b) in &recs {
        let s = a + b;
        let p = a * b;
        out.push_str(&format!("{:>9.2} {:>13.2} N\n", s, p));
    }
    (lines.join("\n") + "\n", out, String::new())
}

pub fn reportwork(n: usize, scale: &str) -> (String, String, String) {
    let (lines, recs) = gen::reportwork(n, gen::seed_for("report", scale));
    let mut out = String::new();
    let mut grand: i128 = 0;
    let mut cur_dept = String::new();
    let mut dept_total: i128 = 0;
    for (dept, amount) in &recs {
        let d = format!("D{dep}", dep = dept);
        if d != cur_dept && !cur_dept.is_empty() {
            out.push_str(&format!("{cur_dept}SUBTOTAL {}\n", pad(dept_total, 14)));
        }
        cur_dept = d;
        dept_total += *amount as i128;
        grand += *amount as i128;
    }
    if !cur_dept.is_empty() {
        out.push_str(&format!("{cur_dept}SUBTOTAL {}\n", pad(dept_total, 14)));
    }
    out.push_str(&format!("GRAND {:09} {}\n", recs.len(), pad(grand, 14)));
    (lines.join("\n") + "\n", out, String::new())
}

pub fn relative(n: usize, scale: &str) -> (String, String, String) {
    let (lines, recs) = gen::relativework(n, gen::seed_for("relative", scale));
    let mut written = 0i128;
    let mut updated = 0i128;
    let mut deleted = 0i128;
    let mut trav_n = 0i128;
    let mut trav_sum: i128 = 0;
    for (key, payload) in &recs {
        if key % 2 == 0 {
            written += 1;
            if key % 8 == 0 {
                deleted += 1;
            } else if key % 4 == 0 {
                updated += 1;
            }
            // every surviving written record is traversed (survivor = written, not deleted)
            if key % 8 != 0 {
                trav_n += 1;
                trav_sum += *payload as i128;
            }
        }
    }
    let out = format!(
        "WRITTEN {:09}\nUPDATED {:09}\nDELETED {:09}\nTRAVERSE {:09} {}\n",
        written,
        updated,
        deleted,
        trav_n,
        pad(trav_sum, 16)
    );
    (lines.join("\n") + "\n", out, String::new())
}

pub fn modules(n: usize, _scale: &str) -> (String, String, String) {
    let mut lines = Vec::new();
    for i in 1..=n {
        lines.push(format!("{i:010}"));
    }
    let mut out = String::new();
    for i in 1..=n {
        let res = i * 3 + 1;
        out.push_str(&format!("CALL {:09} {:010}\n", i, res));
    }
    (lines.join("\n") + "\n", out, String::new())
}

pub fn mixed(n: usize, scale: &str) -> (String, String, String) {
    let mut r = gen::Rng::new(gen::seed_for("mixed", scale));
    let mut lines = Vec::new();
    let mut out = String::new();
    let mut sum: i128 = 0;
    let mut rejects = 0i128;
    for i in 1..=n {
        let qty = r.below(101) as i64;
        let price = 100 + r.below(100_000) as i64;
        let dept = 1 + r.below(5) as i64;
        lines.push(format!("K{i:07}{qty:04}{price:08}{dept:02}"));
        if qty == 0 {
            rejects += 1;
            out.push_str(&format!("REJECT K{i:07}\n"));
            continue;
        }
        let line_total = (qty as i128) * (price as i128);
        let pct: i128 = match dept {
            1 => 5,
            2 => 10,
            3 => 15,
            4 => 20,
            _ => 25,
        };
        let disc = (line_total * pct + 50) / 100;
        let net = line_total - disc;
        let surc = (net * 3 + 50) / 100;
        let total = net + surc;
        sum += total;
        out.push_str(&format!("K{i:07} {}\n", pad(total, 14)));
    }
    out.push_str(&format!(
        "DONE {:09} {} REJECTS {:09}\n",
        n - rejects as usize,
        pad(sum, 14),
        rejects
    ));
    (lines.join("\n") + "\n", out, String::new())
}
