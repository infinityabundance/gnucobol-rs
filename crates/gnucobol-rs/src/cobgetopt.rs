//! 1:1 port of cobgetopt.c — GnuCOBOL's vendored GNU `getopt_long` (originally GNU C Library / gnulib).
//! The C drives everything through file-static globals (`optind`, `optarg`, `nextchar`, `ordering`,
//! `first_nonopt`/`last_nonopt`, …) and permutes `argv` in place. This port carries that state in an
//! explicit [`CobGetopt`] struct (no global mutable state, `#![forbid(unsafe_code)]`): the `char *nextchar`
//! scan pointer becomes a `(elem, off)` index into the owned `argv`, and the caller-visible `cob_optarg` /
//! `cob_optind` / `cob_opterr` / `cob_optopt` are public fields read after each call.
//!
//! The four C functions have named counterparts: [`CobGetopt::cob_getopt_long_long`] (the scanner),
//! `process_long_option`, `_getopt_initialize`, and `exchange` (the permutation primitive).
#![forbid(unsafe_code)]

/// `struct option`'s `has_arg == no_argument`.
pub const NO_ARGUMENT: i32 = 0;
/// `struct option`'s `has_arg == required_argument`.
pub const REQUIRED_ARGUMENT: i32 = 1;
/// `struct option`'s `has_arg == optional_argument`.
pub const OPTIONAL_ARGUMENT: i32 = 2;

/// A long-option descriptor — the port of `struct option { name, has_arg, flag, val }` (cobgetopt.h).
/// The C `int *flag` (written with `val` and triggering a `0` return) is modelled as an opaque slot id:
/// when such an option matches, the `(slot, val)` pair is appended to [`CobGetopt::flag_writes`] and the
/// scanner returns `0`, exactly as the C writes `*flag = val; return 0`.
#[derive(Clone)]
pub struct OptionDef {
    /// The long-option name (without the leading `--`).
    pub name: Vec<u8>,
    /// `no_argument` / `required_argument` / `optional_argument`.
    pub has_arg: i32,
    /// `None` == the C `flag == NULL` (return `val`); `Some(slot)` == write `val` to that slot, return `0`.
    pub flag: Option<usize>,
    /// The value returned (or written through `flag`).
    pub val: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ordering {
    RequireOrder,
    Permute,
    ReturnInOrder,
}

/// A `char *` into the owned `argv` — element index + byte offset. `off == argv[elem].len()` is the C
/// `'\0'` terminator position.
#[derive(Clone, Copy)]
struct Ptr {
    elem: usize,
    off: usize,
}

/// The `getopt_long` scanner state (the cobgetopt.c file-static globals) plus the owned, permutable `argv`.
pub struct CobGetopt {
    argv: Vec<Vec<u8>>,
    argc: i32,
    optstring: Vec<u8>,
    longopts: Vec<OptionDef>,
    long_only: i32,

    /// `cob_optarg` — the argument value of the option just returned (`None` == NULL).
    pub optarg: Option<Vec<u8>>,
    /// `cob_optind` — index of the next `argv` element to scan (1003.2 requires it start at 1).
    pub optind: i32,
    /// `cob_opterr` — non-zero prints the diagnostic for an unrecognised option.
    pub opterr: i32,
    /// `cob_optopt` — the option character found to be unrecognised.
    pub optopt: i32,
    /// `*longind` — the index in `longopts` of the long option found by the most recent call.
    pub longind: i32,
    /// The `*flag = val` writes a long option with a non-NULL `flag` requested (slot id, value).
    pub flag_writes: Vec<(usize, i32)>,

    nextchar: Option<Ptr>,
    getopt_initialized: bool,
    ordering: Ordering,
    first_nonopt: i32,
    last_nonopt: i32,
}

impl CobGetopt {
    /// Build a scanner over `argv` (element 0 is the program name) for `optstring` and `longopts`.
    /// `long_only` mirrors the `getopt_long_only` flag. Mirrors the initial values of the C globals
    /// (`optind = 1`, `opterr = 1`, `optopt = '?'`).
    pub fn new(
        argv: Vec<Vec<u8>>,
        optstring: &[u8],
        longopts: Vec<OptionDef>,
        long_only: i32,
    ) -> Self {
        CobGetopt {
            argc: argv.len() as i32,
            argv,
            optstring: optstring.to_vec(),
            longopts,
            long_only,
            optarg: None,
            optind: 1,
            opterr: 1,
            optopt: b'?' as i32,
            longind: -1,
            flag_writes: Vec::new(),
            nextchar: None,
            getopt_initialized: false,
            ordering: Ordering::Permute,
            first_nonopt: 0,
            last_nonopt: 0,
        }
    }

    /// The permuted `argv` after scanning (the C permutes the caller's array in place).
    pub fn argv(&self) -> &[Vec<u8>] {
        &self.argv
    }

    // --- pointer helpers over the owned argv -------------------------------------------------------

    /// The byte at `p` (the C `*p`); `0` at the element terminator.
    fn at(&self, p: Ptr) -> u8 {
        self.argv[p.elem].get(p.off).copied().unwrap_or(0)
    }

    /// `&p[..]` up to the element terminator (the C string starting at `p`).
    fn rest(&self, p: Ptr) -> &[u8] {
        let e = &self.argv[p.elem];
        if p.off <= e.len() {
            &e[p.off..]
        } else {
            &[]
        }
    }

    fn elem(&self, i: i32) -> &[u8] {
        &self.argv[i as usize]
    }

    fn prog(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(self.elem(0))
    }

    /// `exchange (argv)` (cobgetopt.c:184): swap the two adjacent `argv` subsequences
    /// `[first_nonopt,last_nonopt)` (skipped non-options) and `[last_nonopt,optind)` (options processed
    /// since), so that after scanning the options precede the non-options.
    fn exchange(&mut self) {
        let mut bottom = self.first_nonopt;
        let middle = self.last_nonopt;
        let mut top = self.optind;

        // Exchange the shorter segment with the far end of the longer segment, repeatedly, until the two
        // parts are in place (the C's `bottom`/`top` are loop-local; `middle` never moves).
        while top > middle && middle > bottom {
            if top - middle > middle - bottom {
                // Bottom segment is the short one.
                let len = middle - bottom;
                for i in 0..len {
                    self.argv.swap(
                        (bottom + i) as usize,
                        (top - (middle - bottom) + i) as usize,
                    );
                }
                top -= len;
            } else {
                // Top segment is the short one.
                let len = top - middle;
                for i in 0..len {
                    self.argv.swap((bottom + i) as usize, (middle + i) as usize);
                }
                bottom += len;
            }
        }

        // Update records for the slots the non-options now occupy.
        self.first_nonopt += self.optind - self.last_nonopt;
        self.last_nonopt = self.optind;
    }

    /// `_getopt_initialize (optstring)` (cobgetopt.c:440): first-call setup — pick the ordering from a
    /// leading `-`/`+`, `POSIXLY_CORRECT`, or default PERMUTE, reset the non-option window, and return the
    /// optstring advanced past any leading `-`/`+`.
    fn _getopt_initialize(&mut self) {
        // cob_common_init(NULL) in the C is a no-op for the scanner's observable behaviour.
        if self.optind == 0 {
            self.optind = 1;
        }
        self.first_nonopt = self.optind;
        self.last_nonopt = self.optind;
        self.nextchar = None;

        if self.optstring.first() == Some(&b'-') {
            self.ordering = Ordering::ReturnInOrder;
            self.optstring.remove(0);
        } else if self.optstring.first() == Some(&b'+') {
            self.ordering = Ordering::RequireOrder;
            self.optstring.remove(0);
        } else if std::env::var_os("POSIXLY_CORRECT").is_some() {
            self.ordering = Ordering::RequireOrder;
        } else {
            self.ordering = Ordering::Permute;
        }
        self.getopt_initialized = true;
    }

    /// `process_long_option (...)` (cobgetopt.c:247): match the argument at `nextchar` against `longopts`
    /// (exact, else unique abbreviation), consume it and its argument, and return the value to return from
    /// the scanner — or `-1` when (under `long_only`) it is not actually a long option.
    #[allow(clippy::too_many_arguments)]
    fn process_long_option(&mut self, long_only: i32, print_errors: bool, prefix: &str) -> i32 {
        let nc = self
            .nextchar
            .expect("process_long_option requires nextchar");
        // nameend: scan to '=' or end.
        let mut nameend_off = nc.off;
        loop {
            let b = self.argv[nc.elem].get(nameend_off).copied().unwrap_or(0);
            if b == 0 || b == b'=' {
                break;
            }
            nameend_off += 1;
        }
        let namelen = nameend_off - nc.off;
        let name_bytes: Vec<u8> = self.argv[nc.elem][nc.off..nameend_off].to_vec();

        let mut pfound: Option<usize> = None;
        let mut option_index = 0usize;
        let n_options = self.longopts.len();

        // First look for an exact match.
        for (i, p) in self.longopts.iter().enumerate() {
            if p.name.len() >= namelen
                && p.name[..namelen] == name_bytes[..]
                && namelen == p.name.len()
            {
                pfound = Some(i);
                option_index = i;
                break;
            }
        }

        if pfound.is_none() {
            // Look for abbreviations.
            let mut ambig_set: Option<Vec<bool>> = None;
            let mut ambig_fallback = false;
            let mut indfound: i32 = -1;

            for (i, p) in self.longopts.iter().enumerate() {
                if p.name.len() >= namelen && p.name[..namelen] == name_bytes[..] {
                    match pfound {
                        None => {
                            // First nonexact match found.
                            pfound = Some(i);
                            indfound = i as i32;
                        }
                        Some(pf_idx) => {
                            // Second or later nonexact match: an ambiguity unless the entries are
                            // equivalent (and not long_only). C nests `if(...) { if(!ambig_fallback) }`.
                            let pf = &self.longopts[pf_idx];
                            if (long_only != 0
                                || pf.has_arg != p.has_arg
                                || pf.flag != p.flag
                                || pf.val != p.val)
                                && !ambig_fallback
                            {
                                if !print_errors {
                                    ambig_fallback = true;
                                } else if ambig_set.is_none() {
                                    let mut set = vec![false; n_options];
                                    set[indfound as usize] = true;
                                    ambig_set = Some(set);
                                }
                                if let Some(set) = ambig_set.as_mut() {
                                    set[i] = true;
                                }
                            }
                        }
                    }
                }
            }

            if ambig_set.is_some() || ambig_fallback {
                if print_errors {
                    if ambig_fallback {
                        eprintln!(
                            "{}: option '{}{}' is ambiguous",
                            self.prog(),
                            prefix,
                            String::from_utf8_lossy(&name_bytes)
                        );
                    } else {
                        let set = ambig_set.as_ref().unwrap();
                        eprint!(
                            "{}: option '{}{}' is ambiguous; possibilities:",
                            self.prog(),
                            prefix,
                            String::from_utf8_lossy(&name_bytes)
                        );
                        for (oi, on) in self.longopts.iter().enumerate() {
                            if set[oi] {
                                eprint!(" '{}{}'", prefix, String::from_utf8_lossy(&on.name));
                            }
                        }
                        eprintln!();
                    }
                }
                // nextchar += strlen(nextchar)
                let end = self.argv[nc.elem].len();
                self.nextchar = Some(Ptr {
                    elem: nc.elem,
                    off: end,
                });
                self.optind += 1;
                self.optopt = 0;
                return b'?' as i32;
            }

            option_index = indfound as usize;
        }

        if pfound.is_none() {
            // Can't find it as a long option.
            let argv_optind1 = self.elem(self.optind).get(1).copied().unwrap_or(0);
            let first = self.at(nc);
            let in_optstring = self.optstring.contains(&first);
            if long_only == 0 || argv_optind1 == b'-' || !in_optstring {
                if print_errors {
                    eprintln!(
                        "{}: unrecognized option '{}{}'",
                        self.prog(),
                        prefix,
                        String::from_utf8_lossy(self.rest(nc))
                    );
                }
                self.nextchar = None;
                self.optind += 1;
                self.optopt = 0;
                return b'?' as i32;
            }
            // Otherwise interpret it as a short option.
            return -1;
        }

        let pf_idx = pfound.unwrap();
        // We have found a matching long option.  Consume it.
        self.optind += 1;
        self.nextchar = None;

        let has_eq = self.argv[nc.elem].get(nameend_off).copied().unwrap_or(0) != 0; // '=' present
        let (p_has_arg, p_val, p_flag, p_name) = {
            let p = &self.longopts[pf_idx];
            (p.has_arg, p.val, p.flag, p.name.clone())
        };

        if has_eq {
            if p_has_arg != 0 {
                self.optarg = Some(self.argv[nc.elem][nameend_off + 1..].to_vec());
            } else {
                if print_errors {
                    eprintln!(
                        "{}: option '{}{}' doesn't allow an argument",
                        self.prog(),
                        prefix,
                        String::from_utf8_lossy(&p_name)
                    );
                }
                self.optopt = p_val;
                return b'?' as i32;
            }
        } else if p_has_arg == 1 {
            if self.optind < self.argc {
                self.optarg = Some(self.elem(self.optind).to_vec());
                self.optind += 1;
            } else {
                if print_errors {
                    eprintln!(
                        "{}: option '{}{}' requires an argument",
                        self.prog(),
                        prefix,
                        String::from_utf8_lossy(&p_name)
                    );
                }
                self.optopt = p_val;
                return if self.optstring.first() == Some(&b':') {
                    b':' as i32
                } else {
                    b'?' as i32
                };
            }
        }

        self.longind = option_index as i32;
        if let Some(slot) = p_flag {
            self.flag_writes.push((slot, p_val));
            return 0;
        }
        p_val
    }

    /// `cob_getopt_long_long (...)` (cobgetopt.c:536): return the next option character (or `0` for a
    /// flag-setting long option), `1` for a non-option under RETURN_IN_ORDER, or `-1` when scanning is
    /// done. Reads/updates `optarg`, `optind`, `optopt`, `longind` between calls.
    pub fn cob_getopt_long_long(&mut self) -> i32 {
        let mut print_errors = self.opterr != 0;

        if self.argc < 1 {
            return -1;
        }

        self.optarg = None;

        if self.optind == 0 || !self.getopt_initialized {
            self._getopt_initialize();
        } else if self.optstring.first() == Some(&b'-') || self.optstring.first() == Some(&b'+') {
            self.optstring.remove(0);
        }

        if self.optstring.first() == Some(&b':') {
            print_errors = false;
        }

        // NONOPTION_P(i): argv[i][0] != '-' || argv[i][1] == '\0'
        let nonoption_p = |s: &Self, i: i32| -> bool {
            let e = s.elem(i);
            e.first().copied().unwrap_or(0) != b'-' || e.get(1).copied().unwrap_or(0) == 0
        };

        let nc_exhausted = match self.nextchar {
            None => true,
            Some(p) => self.at(p) == 0,
        };

        if nc_exhausted {
            // Advance to the next ARGV-element.
            if self.last_nonopt > self.optind {
                self.last_nonopt = self.optind;
            }
            if self.first_nonopt > self.optind {
                self.first_nonopt = self.optind;
            }

            if self.ordering == Ordering::Permute {
                if self.first_nonopt != self.last_nonopt && self.last_nonopt != self.optind {
                    self.exchange();
                } else if self.last_nonopt != self.optind {
                    self.first_nonopt = self.optind;
                }

                while self.optind < self.argc && nonoption_p(self, self.optind) {
                    self.optind += 1;
                }
                self.last_nonopt = self.optind;
            }

            // The special ARGV-element '--'.
            if self.optind != self.argc && self.elem(self.optind) == b"--" {
                self.optind += 1;

                if self.first_nonopt != self.last_nonopt && self.last_nonopt != self.optind {
                    self.exchange();
                } else if self.first_nonopt == self.last_nonopt {
                    self.first_nonopt = self.optind;
                }
                self.last_nonopt = self.argc;

                self.optind = self.argc;
            }

            // If we have done all the ARGV-elements, stop.
            if self.optind == self.argc {
                if self.first_nonopt != self.last_nonopt {
                    self.optind = self.first_nonopt;
                }
                return -1;
            }

            // A non-option that was not permuted.
            if nonoption_p(self, self.optind) {
                if self.ordering == Ordering::RequireOrder {
                    return -1;
                }
                self.optarg = Some(self.elem(self.optind).to_vec());
                self.optind += 1;
                return 1;
            }

            // Another option-ARGV-element; maybe a long option.
            if !self.longopts.is_empty() {
                if self.elem(self.optind).get(1).copied().unwrap_or(0) == b'-' {
                    // "--foo" is always a long option.
                    self.nextchar = Some(Ptr {
                        elem: self.optind as usize,
                        off: 2,
                    });
                    return self.process_long_option(self.long_only, print_errors, "--");
                }

                let oi = self.optind as usize;
                let c2 = self.argv[oi].get(2).copied().unwrap_or(0);
                let c1 = self.argv[oi].get(1).copied().unwrap_or(0);
                if self.long_only != 0 && (c2 != 0 || !self.optstring.contains(&c1)) {
                    self.nextchar = Some(Ptr { elem: oi, off: 1 });
                    let code = self.process_long_option(self.long_only, print_errors, "-");
                    if code != -1 {
                        return code;
                    }
                }
            }

            // Not a long option.  Skip the initial punctuation.
            self.nextchar = Some(Ptr {
                elem: self.optind as usize,
                off: 1,
            });
        }

        // Look at and handle the next short option-character.
        let mut nc = self.nextchar.unwrap();
        let c = self.at(nc);
        nc.off += 1;
        self.nextchar = Some(nc);

        // Increment optind when we start to process its last character.
        if self.at(nc) == 0 {
            self.optind += 1;
        }

        let temp = self.optstring.iter().position(|&x| x == c);
        let mut c = c as i32;

        if temp.is_none() || c == b':' as i32 || c == b';' as i32 {
            if print_errors {
                eprintln!("{}: invalid option -- '{}'", self.prog(), c as u8 as char);
            }
            self.optopt = c;
            return b'?' as i32;
        }
        let ti = temp.unwrap();
        let t1 = self.optstring.get(ti + 1).copied().unwrap_or(0);
        let t2 = self.optstring.get(ti + 2).copied().unwrap_or(0);

        // Treat POSIX -W foo the same as long option --foo.
        if self.optstring[ti] == b'W' && t1 == b';' && !self.longopts.is_empty() {
            // This is an option that requires an argument.
            let wptr: Ptr;
            if self.at(self.nextchar.unwrap()) != 0 {
                wptr = self.nextchar.unwrap();
            } else if self.optind == self.argc {
                if print_errors {
                    eprintln!(
                        "{}: option requires an argument -- '{}'",
                        self.prog(),
                        c as u8 as char
                    );
                }
                self.optopt = c;
                return if self.optstring.first() == Some(&b':') {
                    b':' as i32
                } else {
                    b'?' as i32
                };
            } else {
                wptr = Ptr {
                    elem: self.optind as usize,
                    off: 0,
                };
            }
            self.nextchar = Some(wptr);
            self.optarg = None;
            return self.process_long_option(0, print_errors, "-W ");
        }

        if t1 == b':' {
            if t2 == b':' {
                // Accepts an argument optionally.
                if self.at(self.nextchar.unwrap()) != 0 {
                    self.optarg = Some(self.rest(self.nextchar.unwrap()).to_vec());
                    self.optind += 1;
                } else {
                    self.optarg = None;
                }
                self.nextchar = None;
            } else {
                // Requires an argument.
                if self.at(self.nextchar.unwrap()) != 0 {
                    self.optarg = Some(self.rest(self.nextchar.unwrap()).to_vec());
                    self.optind += 1;
                } else if self.optind == self.argc {
                    if print_errors {
                        eprintln!(
                            "{}: option requires an argument -- '{}'",
                            self.prog(),
                            c as u8 as char
                        );
                    }
                    self.optopt = c;
                    c = if self.optstring.first() == Some(&b':') {
                        b':' as i32
                    } else {
                        b'?' as i32
                    };
                } else {
                    self.optarg = Some(self.elem(self.optind).to_vec());
                    self.optind += 1;
                }
                self.nextchar = None;
            }
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<Vec<u8>> {
        std::iter::once("prog")
            .chain(args.iter().copied())
            .map(|s| s.as_bytes().to_vec())
            .collect()
    }
    fn long(name: &str, has_arg: i32, val: i32) -> OptionDef {
        OptionDef {
            name: name.as_bytes().to_vec(),
            has_arg,
            flag: None,
            val,
        }
    }
    /// Drive to completion, returning `(ret, optarg, optopt)` per call (the byte court's tuple).
    fn run(g: &mut CobGetopt) -> Vec<(i32, Option<String>, i32)> {
        let mut seq = Vec::new();
        loop {
            let r = g.cob_getopt_long_long();
            seq.push((
                r,
                g.optarg
                    .as_ref()
                    .map(|v| String::from_utf8_lossy(v).into_owned()),
                g.optopt,
            ));
            if r == -1 || seq.len() > 50 {
                break;
            }
        }
        seq
    }

    #[test]
    fn short_required_optional_missing() {
        // -a foo -b   (a needs an arg, b is a flag)
        let mut g = CobGetopt::new(argv(&["-a", "foo", "-b"]), b"a:b", vec![], 0);
        g.opterr = 0;
        let s = run(&mut g);
        assert_eq!(s[0].0, b'a' as i32);
        assert_eq!(s[0].1.as_deref(), Some("foo"));
        assert_eq!(s[1].0, b'b' as i32);
        assert_eq!(s[2].0, -1);

        // -a   with required arg missing -> '?'
        let mut g = CobGetopt::new(argv(&["-a"]), b"a:", vec![], 0);
        g.opterr = 0;
        let s = run(&mut g);
        assert_eq!(s[0].0, b'?' as i32);
        assert_eq!(s[0].2, b'a' as i32);

        // leading ':' optstring -> missing required arg returns ':'
        let mut g = CobGetopt::new(argv(&["-a"]), b":a:", vec![], 0);
        g.opterr = 0;
        assert_eq!(run(&mut g)[0].0, b':' as i32);
    }

    #[test]
    fn long_exact_abbrev_ambiguous_eq() {
        // --foo=hello (required arg via '=')
        let mut g = CobGetopt::new(
            argv(&["--foo=hello"]),
            b"",
            vec![long("foo", REQUIRED_ARGUMENT, 1)],
            0,
        );
        g.opterr = 0;
        let s = run(&mut g);
        assert_eq!(s[0].0, 1);
        assert_eq!(s[0].1.as_deref(), Some("hello"));

        // --fo unique abbreviation of foo
        let mut g = CobGetopt::new(
            argv(&["--fo"]),
            b"",
            vec![long("foo", NO_ARGUMENT, 7), long("zap", NO_ARGUMENT, 8)],
            0,
        );
        g.opterr = 0;
        assert_eq!(run(&mut g)[0].0, 7);

        // --fo ambiguous between foo and fob -> '?'
        let mut g = CobGetopt::new(
            argv(&["--fo"]),
            b"",
            vec![long("foo", NO_ARGUMENT, 1), long("fob", NO_ARGUMENT, 2)],
            0,
        );
        g.opterr = 0;
        assert_eq!(run(&mut g)[0].0, b'?' as i32);
    }

    #[test]
    fn permutation_moves_nonoptions_to_end() {
        // PERMUTE: -a and -b are extracted, x y z permuted to the end; optind lands on first non-option.
        let mut g = CobGetopt::new(argv(&["x", "-a", "y", "-b", "z"]), b"ab", vec![], 0);
        g.opterr = 0;
        let s = run(&mut g);
        assert_eq!(s[0].0, b'a' as i32);
        assert_eq!(s[1].0, b'b' as i32);
        assert_eq!(s[2].0, -1);
        // after scanning, argv[optind..] are the non-options in order.
        let rest: Vec<&[u8]> = g.argv()[g.optind as usize..]
            .iter()
            .map(|v| v.as_slice())
            .collect();
        assert_eq!(rest, vec![&b"x"[..], &b"y"[..], &b"z"[..]]);
    }

    #[test]
    fn require_order_and_return_in_order() {
        // '+' -> REQUIRE_ORDER: stop at the first non-option.
        let mut g = CobGetopt::new(argv(&["x", "-a"]), b"+ab", vec![], 0);
        g.opterr = 0;
        assert_eq!(run(&mut g)[0].0, -1);

        // '-' -> RETURN_IN_ORDER: each non-option comes back as code 1.
        let mut g = CobGetopt::new(argv(&["x", "-a", "y"]), b"-ab", vec![], 0);
        g.opterr = 0;
        let s = run(&mut g);
        assert_eq!(s[0].0, 1);
        assert_eq!(s[0].1.as_deref(), Some("x"));
        assert_eq!(s[1].0, b'a' as i32);
        assert_eq!(s[2].0, 1);
        assert_eq!(s[2].1.as_deref(), Some("y"));
    }

    #[test]
    fn dashdash_terminates_options() {
        let mut g = CobGetopt::new(argv(&["-a", "--", "-b"]), b"ab", vec![], 0);
        g.opterr = 0;
        let s = run(&mut g);
        assert_eq!(s[0].0, b'a' as i32);
        assert_eq!(s[1].0, -1); // -b is past '--', not an option
    }

    #[test]
    fn flag_writes_recorded() {
        // A long option with a non-NULL flag records (slot, val) and returns 0.
        let mut g = CobGetopt::new(
            argv(&["--verbose"]),
            b"",
            vec![OptionDef {
                name: b"verbose".to_vec(),
                has_arg: NO_ARGUMENT,
                flag: Some(3),
                val: 42,
            }],
            0,
        );
        g.opterr = 0;
        let s = run(&mut g);
        assert_eq!(s[0].0, 0);
        assert_eq!(g.flag_writes, vec![(3, 42)]);
    }
}
