//! Rust mirror for the binary-float sweep (both directions).
use gnucobol_rs::float::{decimal_to_f64_trunc, decimal_to_f32_trunc, f64_to_decimal_trunc, dec64_encode, dec128_encode, dec64_decode, dec128_decode, dec_value_to_decimal};
use std::io::{self, BufRead, Write};
fn parse_hex(s: &str, size: usize) -> Vec<u8> { (0..size).map(|i| u8::from_str_radix(&s[2*i..2*i+2], 16).unwrap()).collect() }
fn main() {
    let stdin = io::stdin(); let stdout = io::stdout(); let mut out = stdout.lock();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 12 { continue; }
        let label = f[0];
        let s_type: u32 = f[1].parse().unwrap();
        let s_dig: usize = f[2].parse().unwrap();
        let s_scale: i32 = f[3].parse().unwrap();
        let s_size: usize = f[5].parse().unwrap();
        let s = parse_hex(f[6], s_size);
        let d_type: u32 = f[7].parse().unwrap();
        let d_dig: usize = f[8].parse().unwrap();
        let d_scale: i32 = f[9].parse().unwrap();
        let d_flags: u32 = f[10].parse().unwrap();
        let d_size: usize = f[11].parse().unwrap();
        // helper: decode a DISPLAY source to (mag, scale-from-field)
        let decode_display = |s: &[u8], dig: usize| -> i128 {
            let mut mag: i128 = 0; let mut neg = false;
            for (i, &b) in s.iter().enumerate().take(dig) {
                mag = mag * 10 + (b & 0x0F) as i128;
                if i == dig - 1 && (0x70..=0x79).contains(&b) { neg = true; }
            }
            if neg { -mag } else { mag }
        };
        let render_display = |mag: i128, neg: bool, dig: usize, sz: usize, flags: u32| -> Vec<u8> {
            let modulus = 10i128.pow(dig as u32);
            let mut absm = (mag.unsigned_abs() as i128 % modulus) as u128;
            let mut digits = vec![0u8; dig];
            for slot in digits.iter_mut().rev() { *slot = (absm % 10) as u8; absm /= 10; }
            let mut o: Vec<u8> = digits.iter().map(|d| b'0' + d).collect();
            if flags & 1 != 0 && neg && mag != 0 { if let Some(l) = o.last_mut() { *l |= 0x40; } }
            o.truncate(sz); o
        };
        let bytes: Vec<u8> = if s_type == 16 && (d_type == 22) {
            dec64_encode(decode_display(&s, s_dig), s_scale).to_vec()
        } else if s_type == 16 && (d_type == 23) {
            dec128_encode(decode_display(&s, s_dig), s_scale).to_vec()
        } else if s_type == 22 || s_type == 23 {
            // FLOAT-DECIMAL -> DISPLAY
            let (mag, dscale) = if s_type == 22 {
                dec64_decode(s[..8].try_into().unwrap()).unwrap_or((0,0))
            } else {
                dec128_decode(s[..16].try_into().unwrap()).unwrap_or((0,0))
            };
            let (m, neg) = dec_value_to_decimal(mag.unsigned_abs(), -dscale, mag < 0, d_scale);
            render_display(m, neg, d_dig, d_size, d_flags)
        } else if s_type == 16 {
            // DISPLAY -> FLOAT/DOUBLE (encode)
            let mut mag: i128 = 0; let mut neg = false;
            for (i, &b) in s.iter().enumerate().take(s_dig) {
                mag = mag * 10 + (b & 0x0F) as i128;
                if i == s_dig - 1 && (0x70..=0x79).contains(&b) { neg = true; }
            }
            if neg { mag = -mag; }
            if d_type == 20 { decimal_to_f64_trunc(mag, s_scale).to_le_bytes().to_vec() }
            else { decimal_to_f32_trunc(mag, s_scale).to_le_bytes().to_vec() }
        } else {
            // FLOAT/DOUBLE -> DISPLAY (decode)
            let v: f64 = if s_type == 20 { f64::from_le_bytes(s[..8].try_into().unwrap()) }
                         else { f32::from_le_bytes(s[..4].try_into().unwrap()) as f64 };
            let (mag, neg) = f64_to_decimal_trunc(v, d_scale);
            let modulus = 10i128.pow(d_dig as u32);
            let mut absm = (mag.unsigned_abs() as i128 % modulus) as u128;
            let mut digits = vec![0u8; d_dig];
            for slot in digits.iter_mut().rev() { *slot = (absm % 10) as u8; absm /= 10; }
            let mut o: Vec<u8> = digits.iter().map(|d| b'0' + d).collect();
            if d_flags & 1 != 0 && neg && mag != 0 { if let Some(l) = o.last_mut() { *l |= 0x40; } }
            o.truncate(d_size);
            o
        };
        let _ = writeln!(out, "{label} {}", bytes.iter().map(|b| format!("{b:02x}")).collect::<String>());
    }
}
