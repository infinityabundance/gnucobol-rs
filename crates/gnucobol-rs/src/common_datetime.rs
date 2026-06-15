//! Port of the **pure** date/time field-derivation and ACCEPT formatters of `libcob/common.c`.
//!
//! The COBOL `ACCEPT ... FROM DATE / DAY / TIME / DAY-OF-WEEK` statements are serviced in libcob by a
//! family of `cob_accept_*` functions. Each one reads the system clock via `cob_get_current_datetime`
//! (the impure boundary), assembles a fixed-width unsigned integer from the relevant `struct cob_time`
//! fields, and `cob_move`s that value into the receiving field. Because the integer is built with a known,
//! constant digit count and then moved into a numeric/group field, the canonical observable result is a
//! **fixed-width, zero-padded decimal string**.
//!
//! This module models the `struct cob_time` (as [`CobTime`]) and ports the field-derivation that is pure
//! given the raw broken-down local-time components, plus each `cob_accept_*` formatter as a pure function
//! `&CobTime -> Vec<u8>` returning exactly those canonical digit bytes. The system-clock read and the
//! receiving-field copy/truncation are the caller's concern.

#![forbid(unsafe_code)]

/// Port of libcob's `struct cob_time` (common.h). All fields are the broken-down local-time components
/// as filled by `set_cob_time_from_localtime`.
///
/// Field ranges (per the C header comments):
/// - `year`: [1900-9999]
/// - `month`: [1-12], 1 = Jan
/// - `day_of_month`: [1-31]
/// - `day_of_week`: [1-7], 1 = Monday ... 7 = Sunday
/// - `day_of_year`: [1-366]
/// - `hour`: [0-23]
/// - `minute`: [0-59]
/// - `second`: [0-60] (1 leap second)
/// - `nanosecond`: nanoseconds
/// - `offset_known`: nonzero if `utc_offset` is meaningful
/// - `utc_offset`: minutes east of UTC
/// - `is_daylight_saving_time`: [-1/0/1]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CobTime {
    pub year: i32,
    pub month: i32,
    pub day_of_month: i32,
    pub day_of_week: i32,
    pub day_of_year: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub nanosecond: i32,
    pub offset_known: i32,
    pub utc_offset: i32,
    pub is_daylight_saving_time: i32,
}

/// Port of `common.c:one_indexed_day_of_week_from_monday` -- convert a zero-indexed-from-Sunday weekday
/// (C `struct tm`'s `tm_wday`, 0 = Sunday) into the COBOL one-indexed-from-Monday weekday (1 = Monday ...
/// 7 = Sunday).
///
/// ```text
/// return ((zero_indexed_from_sunday + 6) % 7) + 1;
/// ```
pub fn one_indexed_day_of_week_from_monday(zero_indexed_from_sunday: i32) -> i32 {
    ((zero_indexed_from_sunday + 6) % 7) + 1
}

impl CobTime {
    /// Port of the **pure** field-derivation in `common.c:set_cob_time_from_localtime` -- given the raw
    /// broken-down local-time components (as produced by C's `localtime` into a `struct tm`), derive the
    /// `cob_time` fields.
    ///
    /// The C code reads `tmptr` (a `struct tm`) and assigns:
    /// ```text
    /// cb_time->year         = tmptr->tm_year + 1900;
    /// cb_time->month        = tmptr->tm_mon + 1;
    /// cb_time->day_of_month = tmptr->tm_mday;
    /// cb_time->day_of_week  = one_indexed_day_of_week_from_monday (tmptr->tm_wday);
    /// cb_time->day_of_year  = tmptr->tm_yday + 1;
    /// cb_time->hour         = tmptr->tm_hour;
    /// cb_time->minute       = tmptr->tm_min;
    /// if (tmptr->tm_sec >= 60) tmptr->tm_sec = 59;   /* leap seconds */
    /// cb_time->second       = tmptr->tm_sec;
    /// cb_time->nanosecond   = 0;
    /// cb_time->is_daylight_saving_time = tmptr->tm_isdst;
    /// ```
    /// `nanosecond` and the `utc_offset` / `offset_known` pair are filled separately (the latter via the
    /// platform timezone APIs -- the impure boundary), so this constructor sets `nanosecond = 0`,
    /// `offset_known = 0`, `utc_offset = 0`.
    ///
    /// Parameters mirror the `struct tm` members read above: `tm_year` (years since 1900), `tm_mon`
    /// (0-11), `tm_mday` (1-31), `tm_wday` (0 = Sunday), `tm_yday` (0-365), `tm_hour`, `tm_min`, `tm_sec`,
    /// `tm_isdst`.
    #[allow(clippy::too_many_arguments)]
    pub fn from_tm_fields(
        tm_year: i32,
        tm_mon: i32,
        tm_mday: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_hour: i32,
        tm_min: i32,
        tm_sec: i32,
        tm_isdst: i32,
    ) -> CobTime {
        // Leap seconds ? (C clamps tm_sec >= 60 down to 59)
        let second = if tm_sec >= 60 { 59 } else { tm_sec };
        CobTime {
            year: tm_year + 1900,
            month: tm_mon + 1,
            day_of_month: tm_mday,
            day_of_week: one_indexed_day_of_week_from_monday(tm_wday),
            day_of_year: tm_yday + 1,
            hour: tm_hour,
            minute: tm_min,
            second,
            nanosecond: 0,
            offset_known: 0,
            utc_offset: 0,
            is_daylight_saving_time: tm_isdst,
        }
    }
}

/// Format an unsigned value as exactly `width` zero-padded base-10 ASCII digits (the canonical result of
/// `cob_move`ing a `width`-digit `COB_TYPE_NUMERIC_BINARY` value into a display/group receiver). If the
/// value needs more than `width` digits it is reduced modulo `10^width`, mirroring the digit-count
/// truncation of the libcob `cob_field` (`digits` = `width`).
fn fixed_width_digits(value: u64, width: usize) -> Vec<u8> {
    let mut out = vec![0x30u8; width]; // '0'
    let mut v = value;
    let mut i = width;
    while i > 0 {
        i -= 1;
        out[i] = 0x30u8 + (v % 10) as u8;
        v /= 10;
    }
    out
}

/// Port of `common.c:cob_accept_date` -- ACCEPT ... FROM DATE. Returns the 6 bytes "YYMMDD".
///
/// C builds `val = day_of_month + month*100 + (year % 100)*10000` as a 6-digit numeric, then moves it
/// into `f`. The canonical formatted result is the 6-digit zero-padded decimal of that value.
pub fn cob_accept_date(t: &CobTime) -> Vec<u8> {
    let val = t.day_of_month as u64 + t.month as u64 * 100 + (t.year % 100) as u64 * 10000;
    fixed_width_digits(val, 6)
}

/// Port of `common.c:cob_accept_date_yyyymmdd` -- ACCEPT ... FROM DATE YYYYMMDD. Returns the 8 bytes
/// "YYYYMMDD".
///
/// C builds `val = day_of_month + month*100 + year*10000` as an 8-digit numeric, then moves it into `f`.
pub fn cob_accept_date_yyyymmdd(t: &CobTime) -> Vec<u8> {
    let val = t.day_of_month as u64 + t.month as u64 * 100 + t.year as u64 * 10000;
    fixed_width_digits(val, 8)
}

/// Port of `common.c:cob_accept_day` -- ACCEPT ... FROM DAY. Returns the 5 bytes "YYDDD" (year-of-century
/// + day-of-year).
///
/// C builds `val = day_of_year + (year % 100)*1000` as a 5-digit numeric, then moves it into `f`.
pub fn cob_accept_day(t: &CobTime) -> Vec<u8> {
    let val = t.day_of_year as u64 + (t.year % 100) as u64 * 1000;
    fixed_width_digits(val, 5)
}

/// Port of `common.c:cob_accept_day_yyyyddd` -- ACCEPT ... FROM DAY YYYYDDD. Returns the 7 bytes
/// "YYYYDDD" (full year + day-of-year).
///
/// C builds `val = day_of_year + year*1000` as a 7-digit numeric, then moves it into `f`.
pub fn cob_accept_day_yyyyddd(t: &CobTime) -> Vec<u8> {
    let val = t.day_of_year as u64 + t.year as u64 * 1000;
    fixed_width_digits(val, 7)
}

/// Port of `common.c:cob_accept_day_of_week` -- ACCEPT ... FROM DAY-OF-WEEK. Returns the single byte
/// `day_of_week + '0'` (Monday = '1' ... Sunday = '7').
///
/// C computes `day = (unsigned char)(time.day_of_week + '0')` and moves the single digit into `f`.
/// (`'0'` is `0x30`.)
pub fn cob_accept_day_of_week(t: &CobTime) -> Vec<u8> {
    vec![(t.day_of_week + 0x30) as u8]
}

/// Port of `common.c:cob_accept_time` -- ACCEPT ... FROM TIME. Returns the 8 bytes "HHMMSShh" (hours,
/// minutes, seconds, hundredths of a second).
///
/// C builds an 8-digit numeric:
/// ```text
/// val = (nanosecond / 10000000)   /* hundredths */
///     + second * 100
///     + minute * 10000
///     + hour   * 1000000;
/// ```
/// then moves it into `f`. Note: in C the receiving field's `size` selects `DTR_FULL` vs
/// `DTR_TIME_NO_NANO` (i.e. whether nanoseconds are read at all); that is the clock-read boundary. Given a
/// populated `CobTime`, the formatted value is purely the 8-digit decimal above.
pub fn cob_accept_time(t: &CobTime) -> Vec<u8> {
    let val = (t.nanosecond / 10000000) as u64
        + t.second as u64 * 100
        + t.minute as u64 * 10000
        + t.hour as u64 * 1000000;
    fixed_width_digits(val, 8)
}

/// Port of `common.c:cob_accept_microsecond_time` -- ACCEPT ... FROM TIME with microsecond precision.
/// Returns the 12 bytes "HHMMSSssssss" (hours, minutes, seconds, microseconds).
///
/// C builds a 12-digit numeric (held in a `cob_u64_t`):
/// ```text
/// val = (nanosecond / 1000)        /* microseconds */
///     + second * 1000000
///     + minute * 100000000
///     + hour   * 10000000000;
/// ```
/// then moves it into `f`.
pub fn cob_accept_microsecond_time(t: &CobTime) -> Vec<u8> {
    let val = (t.nanosecond / 1000) as u64
        + t.second as u64 * 1000000
        + t.minute as u64 * 100000000
        + t.hour as u64 * 10000000000;
    fixed_width_digits(val, 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed reference time: 2026-06-15 14:30:45, with 123456789 ns (-> 12 hundredths, 123456 us),
    /// day_of_year = 166 (2026-06-15), day_of_week = 1 (Monday).
    fn fixed() -> CobTime {
        CobTime {
            year: 2026,
            month: 6,
            day_of_month: 15,
            day_of_week: 1, // Monday
            day_of_year: 166,
            hour: 14,
            minute: 30,
            second: 45,
            nanosecond: 123_456_789,
            offset_known: 1,
            utc_offset: 0,
            is_daylight_saving_time: 0,
        }
    }

    #[test]
    fn one_indexed_dow() {
        // tm_wday: 0=Sun .. 6=Sat  ->  COBOL 1=Mon .. 7=Sun
        assert_eq!(one_indexed_day_of_week_from_monday(0), 7); // Sunday
        assert_eq!(one_indexed_day_of_week_from_monday(1), 1); // Monday
        assert_eq!(one_indexed_day_of_week_from_monday(2), 2); // Tuesday
        assert_eq!(one_indexed_day_of_week_from_monday(6), 6); // Saturday
    }

    #[test]
    fn from_tm_fields_derivation() {
        // 2026-06-15 14:30:45, Monday. localtime: tm_year=126, tm_mon=5, tm_mday=15,
        // tm_wday=1 (Monday), tm_yday=165, tm_hour=14, tm_min=30, tm_sec=45.
        let t = CobTime::from_tm_fields(126, 5, 15, 1, 165, 14, 30, 45, 0);
        assert_eq!(t.year, 2026);
        assert_eq!(t.month, 6);
        assert_eq!(t.day_of_month, 15);
        assert_eq!(t.day_of_week, 1);
        assert_eq!(t.day_of_year, 166);
        assert_eq!(t.hour, 14);
        assert_eq!(t.minute, 30);
        assert_eq!(t.second, 45);
        assert_eq!(t.nanosecond, 0);
        assert_eq!(t.offset_known, 0);
        assert_eq!(t.utc_offset, 0);
        assert_eq!(t.is_daylight_saving_time, 0);
    }

    #[test]
    fn from_tm_fields_leap_second_clamp() {
        // tm_sec == 60 (a leap second) is clamped to 59.
        let t = CobTime::from_tm_fields(126, 5, 15, 1, 165, 23, 59, 60, 0);
        assert_eq!(t.second, 59);
    }

    #[test]
    fn fixed_width_helper() {
        assert_eq!(fixed_width_digits(0, 6), b"000000");
        assert_eq!(fixed_width_digits(45, 6), b"000045");
        assert_eq!(fixed_width_digits(123456, 6), b"123456");
        // overflow reduces modulo 10^width
        assert_eq!(fixed_width_digits(1_234_567, 6), b"234567");
    }

    #[test]
    fn accept_date() {
        // YYMMDD: 26-06-15 -> "260615"
        assert_eq!(cob_accept_date(&fixed()), b"260615");
    }

    #[test]
    fn accept_date_yyyymmdd() {
        // YYYYMMDD: 2026-06-15 -> "20260615"
        assert_eq!(cob_accept_date_yyyymmdd(&fixed()), b"20260615");
    }

    #[test]
    fn accept_day() {
        // YYDDD: 26 + 166 -> "26166"
        assert_eq!(cob_accept_day(&fixed()), b"26166");
    }

    #[test]
    fn accept_day_yyyyddd() {
        // YYYYDDD: 2026 + 166 -> "2026166"
        assert_eq!(cob_accept_day_yyyyddd(&fixed()), b"2026166");
    }

    #[test]
    fn accept_day_of_week() {
        // Monday -> '1'
        assert_eq!(cob_accept_day_of_week(&fixed()), b"1");
        // Sunday -> '7'
        let mut sun = fixed();
        sun.day_of_week = 7;
        assert_eq!(cob_accept_day_of_week(&sun), b"7");
    }

    #[test]
    fn accept_time() {
        // HHMMSShh: 14:30:45, ns=123456789 -> hundredths = 123456789/10000000 = 12 -> "14304512"
        assert_eq!(cob_accept_time(&fixed()), b"14304512");
    }

    #[test]
    fn accept_time_zero_nanos() {
        let mut t = fixed();
        t.nanosecond = 0;
        assert_eq!(cob_accept_time(&t), b"14304500");
    }

    #[test]
    fn accept_microsecond_time() {
        // HHMMSSssssss: 14:30:45, ns=123456789 -> us = 123456789/1000 = 123456 -> "143045123456"
        assert_eq!(cob_accept_microsecond_time(&fixed()), b"143045123456");
    }

    #[test]
    fn accept_microsecond_time_midnight() {
        let t = CobTime {
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
            ..fixed()
        };
        assert_eq!(cob_accept_microsecond_time(&t), b"000000000000");
    }
}
