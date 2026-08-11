//! Prepared-program persistence (spec 9.6).
//!
//! A **versioned, deterministic, corruption-safe** representation of a [`PreparedProgram`]
//! (super::PreparedProgram) so an already-prepared program can be saved to disk once and loaded
//! for repeated execution WITHOUT reparsing the source (cache-off equivalence is enforced by
//! tests: a loaded program must behave byte-identically to a freshly prepared one).
//!
//! Requirements implemented here:
//! - source hash + expanded-source hash are carried and verified;
//! - dialect, compat stamp, parser/checker version are carried (stale detection);
//! - deterministic serialization (every HashMap is emitted in sorted-key order, so the same
//!   program always produces the same bytes);
//! - corruption detection: the payload carries a trailing SHA-256 that `load` verifies;
//! - atomic persistence: writes to a temp file in the destination directory, fsyncs, then
//!   renames over the target (a concurrent reader sees either the old or the new file, never a
//!   torn write);
//! - no unsafe deserialization: the crate is `#![forbid(unsafe_code)]`; the decoder validates
//!   every length and tag against the buffer before reading;
//! - stale invalidation: `load` refuses a file whose compat stamp or expected source hash
//!   disagrees with the caller's expectation.

use super::*;

/// The on-disk magic + version. Bump `FORMAT_VERSION` on any wire-format change; bump
/// `super::PreparedProgram::compat` (`prepared-v1`) when the *front-end artifacts* change shape.
const MAGIC: &[u8; 16] = b"GNURUST-PREPARED";
const FORMAT_VERSION: u16 = 1;
/// Trailing checksum length (SHA-256 hex).
const CHECKSUM_LEN: usize = 64;

/// A tiny, fully-validating writer (deterministic: no HashMap iteration order leaks).
struct Enc {
    out: Vec<u8>,
}

impl Enc {
    fn new() -> Self {
        Enc { out: Vec::new() }
    }
    fn bytes(&mut self, b: &[u8]) {
        self.out.extend_from_slice(b);
    }
    fn u8(&mut self, v: u8) {
        self.out.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.out.extend_from_slice(&v.to_be_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.out.extend_from_slice(&v.to_be_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.out.extend_from_slice(&v.to_be_bytes());
    }
    fn bool(&mut self, v: bool) {
        self.out.push(if v { 1 } else { 0 });
    }
    fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.out.extend_from_slice(s.as_bytes());
    }
    fn opt_str(&mut self, s: &Option<String>) {
        match s {
            Some(s) => {
                self.u8(1);
                self.str(s);
            }
            None => self.u8(0),
        }
    }
    fn opt_tok(&mut self, t: &Option<Tok>) {
        match t {
            Some(t) => {
                self.u8(1);
                self.tok(t);
            }
            None => self.u8(0),
        }
    }
    fn str_vec(&mut self, v: &[String]) {
        self.u32(v.len() as u32);
        for s in v {
            self.str(s);
        }
    }
    fn usize(&mut self, v: usize) {
        self.u64(v as u64);
    }
    fn tok(&mut self, t: &Tok) {
        match t {
            Tok::Word(w) => {
                self.u8(0);
                self.str(w);
            }
            Tok::Str(b) => {
                self.u8(1);
                self.u32(b.len() as u32);
                self.bytes(b);
            }
            Tok::AllLiteral(b) => {
                self.u8(2);
                self.u32(b.len() as u32);
                self.bytes(b);
            }
            Tok::Dot => self.u8(3),
        }
    }
    fn tok_vec(&mut self, v: &[Tok]) {
        self.u32(v.len() as u32);
        for t in v {
            self.tok(t);
        }
    }
    fn cond_val(&mut self, c: &CondVal) {
        match c {
            CondVal::Single(s) => {
                self.u8(0);
                self.str(s);
            }
            CondVal::Range(a, b) => {
                self.u8(1);
                self.str(a);
                self.str(b);
            }
        }
    }
    fn usage(&mut self, u: Option<Usage>) {
        match u {
            None => self.u8(0),
            Some(Usage::Display) => self.u8(1),
            Some(Usage::Comp3) => self.u8(2),
            Some(Usage::Comp) => self.u8(3),
            Some(Usage::Comp5) => self.u8(4),
            Some(Usage::CompX) => self.u8(5),
            Some(Usage::Comp6) => self.u8(6),
            Some(Usage::CompNative(w)) => {
                self.u8(7);
                self.u8(w);
            }
        }
    }
    fn file_org(&mut self, o: &FileOrg) {
        self.u8(match o {
            FileOrg::LineSequential => 0,
            FileOrg::Sequential => 1,
            FileOrg::Relative => 2,
            FileOrg::Indexed => 3,
            FileOrg::Sort => 4,
        });
    }
    fn gtype(&mut self, g: &GType) {
        match g {
            GType::ReportHeading => self.u8(0),
            GType::PageHeading => self.u8(1),
            GType::Detail => self.u8(2),
            GType::ControlHeading(s) => {
                self.u8(3);
                self.str(s);
            }
            GType::ControlFooting(s) => {
                self.u8(4);
                self.str(s);
            }
            GType::PageFooting => self.u8(5),
            GType::ReportFooting => self.u8(6),
        }
    }
    fn line_spec(&mut self, l: &LineSpec) {
        match l {
            LineSpec::Abs(n) => {
                self.u8(0);
                self.usize(*n);
            }
            LineSpec::Plus(n) => {
                self.u8(1);
                self.usize(*n);
            }
        }
    }
    fn relem(&mut self, e: &RElem) {
        self.usize(e.column);
        self.str(&e.pic);
        self.opt_str(&e.source);
        self.opt_tok(&e.value);
        self.opt_str(&e.sum);
    }
    fn rline(&mut self, l: &RLine) {
        self.line_spec(&l.spec);
        self.u32(l.elems.len() as u32);
        for e in &l.elems {
            self.relem(e);
        }
    }
    fn rgroup(&mut self, g: &RGroup) {
        self.opt_str(&g.name);
        self.gtype(&g.gtype);
        self.u32(g.lines.len() as u32);
        for l in &g.lines {
            self.rline(l);
        }
    }
    fn report_def(&mut self, r: &ReportDef) {
        self.str(&r.file);
        self.usize(r.page_limit);
        self.usize(r.heading);
        self.usize(r.first_detail);
        self.usize(r.footing);
        self.str_vec(&r.controls);
        self.u32(r.groups.len() as u32);
        for g in &r.groups {
            self.rgroup(g);
        }
    }
    fn file_def(&mut self, f: &FileDef) {
        self.str(&f.name);
        self.str(&f.assign);
        self.str_vec(&f.records);
        self.opt_str(&f.status);
        self.file_org(&f.org);
        self.opt_str(&f.rel_key);
        self.opt_str(&f.record_key);
        self.opt_str(&f.varying_dep);
        self.bool(f.access_random);
    }
    fn prog_item(&mut self, p: &ProgItem) {
        self.u16(p.level);
        self.str(&p.name);
        self.str(&p.pic);
        self.opt_tok(&p.value);
        self.usize(p.occurs);
        self.opt_str(&p.redefines);
        match &p.condition {
            Some((parent, values, false_value)) => {
                self.u8(1);
                self.str(parent);
                self.u32(values.len() as u32);
                for c in values {
                    self.cond_val(c);
                }
                self.opt_str(false_value);
            }
            None => self.u8(0),
        }
        self.str_vec(&p.indexed_by);
        self.usage(p.usage);
        self.bool(p.sign.0);
        self.bool(p.sign.1);
        self.u16(p.extra_flags);
        match p.float_kind {
            Some(k) => {
                self.u8(1);
                self.u16(k);
            }
            None => self.u8(0),
        }
        self.opt_str(&p.odo_counter);
        match &p.renames {
            Some((a, b)) => {
                self.u8(1);
                self.str(a);
                self.str(b);
            }
            None => self.u8(0),
        }
        self.bool(p.sync);
        self.bool(p.external);
        match p.occurs_key {
            Some(asc) => {
                self.u8(1);
                self.bool(asc);
            }
            None => self.u8(0),
        }
    }
    fn prog_item_vec(&mut self, v: &[ProgItem]) {
        self.u32(v.len() as u32);
        for p in v {
            self.prog_item(p);
        }
    }
    fn program_def(&mut self, p: &ProgramDef) {
        self.prog_item_vec(&p.ws);
        self.prog_item_vec(&p.linkage);
        self.str_vec(&p.using);
        self.u32(p.files.len() as u32);
        for f in &p.files {
            self.file_def(f);
        }
        // deterministic: sorted report names
        let mut names: Vec<&String> = p.reports.keys().collect();
        names.sort();
        self.u32(names.len() as u32);
        for n in names {
            self.str(n);
            self.report_def(&p.reports[n]);
        }
        self.tok_vec(&p.proc_toks);
        self.u32(p.proc_lines.len() as u32);
        for l in &p.proc_lines {
            self.usize(*l);
        }
        self.bool(p.is_initial);
        self.bool(p.is_prototype);
    }
    fn dialect(&mut self, d: &crate::dialect::Dialect) {
        self.u8(match d.binary_size {
            crate::dialect::BinarySize::Cob1248 => 0,
            crate::dialect::BinarySize::Cob248 => 1,
            crate::dialect::BinarySize::Cob1to8 => 2,
        });
        self.bool(d.binary_truncate);
        self.bool(d.complex_odo);
        self.bool(d.odoslide);
        match d.defaultbyte {
            crate::dialect::DefaultByte::Init => {
                self.u8(0);
                self.u8(0);
            }
            crate::dialect::DefaultByte::Fill(b) => {
                self.u8(1);
                self.u8(b);
            }
        }
        self.bool(d.move_ibm);
        self.bool(d.init_justify);
    }
    fn switch_env(&mut self, s: &SwitchEnv) {
        let states = s.states.borrow();
        for st in states.iter() {
            self.bool(*st);
        }
        let mut conds: Vec<(&String, &(usize, bool))> = s.conds.iter().collect();
        conds.sort_by(|a, b| a.0.cmp(b.0));
        self.u32(conds.len() as u32);
        for (k, (idx, on)) in conds {
            self.str(k);
            self.usize(*idx);
            self.bool(*on);
        }
        let mut mons: Vec<(&String, &usize)> = s.mnemonics.iter().collect();
        mons.sort_by(|a, b| a.0.cmp(b.0));
        self.u32(mons.len() as u32);
        for (k, idx) in mons {
            self.str(k);
            self.usize(*idx);
        }
    }
    /// Deterministic program-map emission (sorted program names).
    fn program_map(&mut self, m: &HashMap<String, ProgramDef>) {
        let mut names: Vec<&String> = m.keys().collect();
        names.sort();
        self.u32(names.len() as u32);
        for n in names {
            self.str(n);
            self.program_def(&m[n]);
        }
    }
    /// Deterministic string-map emission (sorted keys) for a value type with a known encoder.
    fn str_map(&mut self, m: &HashMap<String, String>) {
        let mut keys: Vec<&String> = m.keys().collect();
        keys.sort();
        self.u32(keys.len() as u32);
        for k in keys {
            self.str(k);
            self.str(&m[k]);
        }
    }
    fn str_map_filedef(&mut self, m: &HashMap<String, FileDef>) {
        let mut keys: Vec<&String> = m.keys().collect();
        keys.sort();
        self.u32(keys.len() as u32);
        for k in keys {
            self.str(k);
            self.file_def(&m[k]);
        }
    }
    fn str_map_reportdef(&mut self, m: &HashMap<String, ReportDef>) {
        let mut keys: Vec<&String> = m.keys().collect();
        keys.sort();
        self.u32(keys.len() as u32);
        for k in keys {
            self.str(k);
            self.report_def(&m[k]);
        }
    }
}

/// A fully-validating reader: every length is bounds-checked against the buffer before use.
struct Dec<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Dec<'a> {
    fn new(buf: &'a [u8]) -> Result<Self, String> {
        if buf.len() < MAGIC.len() + 2 {
            return Err("prepared file too short".into());
        }
        let d = Dec { buf, pos: 0 };
        Ok(d)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.pos + n > self.buf.len() {
            return Err("prepared file truncated".into());
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, String> {
        let b = self.take(8)?;
        Ok(u64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
    fn bool(&mut self) -> Result<bool, String> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("invalid bool tag".into()),
        }
    }
    fn str(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        let b = self.take(n)?;
        String::from_utf8(b.to_vec()).map_err(|_| "invalid utf-8 in prepared file".into())
    }
    fn opt_str(&mut self) -> Result<Option<String>, String> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.str()?)),
            _ => Err("invalid optional-string tag".into()),
        }
    }
    fn opt_tok(&mut self) -> Result<Option<Tok>, String> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.tok()?)),
            _ => Err("invalid optional-token tag".into()),
        }
    }
    fn str_vec(&mut self) -> Result<Vec<String>, String> {
        let n = self.u32()? as usize;
        let mut v = Vec::with_capacity(n.min(1 << 20));
        for _ in 0..n {
            v.push(self.str()?);
        }
        Ok(v)
    }
    fn usize(&mut self) -> Result<usize, String> {
        Ok(self.u64()? as usize)
    }
    fn tok(&mut self) -> Result<Tok, String> {
        match self.u8()? {
            0 => Ok(Tok::Word(self.str()?)),
            1 => {
                let n = self.u32()? as usize;
                Ok(Tok::Str(self.take(n)?.to_vec()))
            }
            2 => {
                let n = self.u32()? as usize;
                Ok(Tok::AllLiteral(self.take(n)?.to_vec()))
            }
            3 => Ok(Tok::Dot),
            _ => Err("invalid token tag".into()),
        }
    }
    fn tok_vec(&mut self) -> Result<Vec<Tok>, String> {
        let n = self.u32()? as usize;
        let mut v = Vec::with_capacity(n.min(1 << 20));
        for _ in 0..n {
            v.push(self.tok()?);
        }
        Ok(v)
    }
    fn cond_val(&mut self) -> Result<CondVal, String> {
        match self.u8()? {
            0 => Ok(CondVal::Single(self.str()?)),
            1 => Ok(CondVal::Range(self.str()?, self.str()?)),
            _ => Err("invalid cond-value tag".into()),
        }
    }
    fn usage(&mut self) -> Result<Option<Usage>, String> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(Usage::Display)),
            2 => Ok(Some(Usage::Comp3)),
            3 => Ok(Some(Usage::Comp)),
            4 => Ok(Some(Usage::Comp5)),
            5 => Ok(Some(Usage::CompX)),
            6 => Ok(Some(Usage::Comp6)),
            7 => {
                let w = self.u8()?;
                Ok(Some(Usage::CompNative(w)))
            }
            _ => Err("invalid usage tag".into()),
        }
    }
    fn file_org(&mut self) -> Result<FileOrg, String> {
        match self.u8()? {
            0 => Ok(FileOrg::LineSequential),
            1 => Ok(FileOrg::Sequential),
            2 => Ok(FileOrg::Relative),
            3 => Ok(FileOrg::Indexed),
            4 => Ok(FileOrg::Sort),
            _ => Err("invalid file-org tag".into()),
        }
    }
    fn gtype(&mut self) -> Result<GType, String> {
        match self.u8()? {
            0 => Ok(GType::ReportHeading),
            1 => Ok(GType::PageHeading),
            2 => Ok(GType::Detail),
            3 => Ok(GType::ControlHeading(self.str()?)),
            4 => Ok(GType::ControlFooting(self.str()?)),
            5 => Ok(GType::PageFooting),
            6 => Ok(GType::ReportFooting),
            _ => Err("invalid group-type tag".into()),
        }
    }
    fn line_spec(&mut self) -> Result<LineSpec, String> {
        match self.u8()? {
            0 => Ok(LineSpec::Abs(self.usize()?)),
            1 => Ok(LineSpec::Plus(self.usize()?)),
            _ => Err("invalid line-spec tag".into()),
        }
    }
    fn relem(&mut self) -> Result<RElem, String> {
        Ok(RElem {
            column: self.usize()?,
            pic: self.str()?,
            source: self.opt_str()?,
            value: self.opt_tok()?,
            sum: self.opt_str()?,
        })
    }
    fn rline(&mut self) -> Result<RLine, String> {
        let spec = self.line_spec()?;
        let n = self.u32()? as usize;
        let mut elems = Vec::with_capacity(n.min(1 << 16));
        for _ in 0..n {
            elems.push(self.relem()?);
        }
        Ok(RLine { spec, elems })
    }
    fn rgroup(&mut self) -> Result<RGroup, String> {
        let name = self.opt_str()?;
        let gtype = self.gtype()?;
        let n = self.u32()? as usize;
        let mut lines = Vec::with_capacity(n.min(1 << 16));
        for _ in 0..n {
            lines.push(self.rline()?);
        }
        Ok(RGroup { name, gtype, lines })
    }
    fn report_def(&mut self) -> Result<ReportDef, String> {
        let file = self.str()?;
        let page_limit = self.usize()?;
        let heading = self.usize()?;
        let first_detail = self.usize()?;
        let footing = self.usize()?;
        let controls = self.str_vec()?;
        let n = self.u32()? as usize;
        let mut groups = Vec::with_capacity(n.min(1 << 16));
        for _ in 0..n {
            groups.push(self.rgroup()?);
        }
        Ok(ReportDef {
            file,
            page_limit,
            heading,
            first_detail,
            footing,
            controls,
            groups,
        })
    }
    fn file_def(&mut self) -> Result<FileDef, String> {
        Ok(FileDef {
            name: self.str()?,
            assign: self.str()?,
            records: self.str_vec()?,
            status: self.opt_str()?,
            org: self.file_org()?,
            rel_key: self.opt_str()?,
            record_key: self.opt_str()?,
            varying_dep: self.opt_str()?,
            access_random: self.bool()?,
        })
    }
    fn prog_item(&mut self) -> Result<ProgItem, String> {
        let level = self.u16()?;
        let name = self.str()?;
        let pic = self.str()?;
        let value = self.opt_tok()?;
        let occurs = self.usize()?;
        let redefines = self.opt_str()?;
        let condition = match self.u8()? {
            0 => None,
            1 => {
                let parent = self.str()?;
                let n = self.u32()? as usize;
                let mut values = Vec::with_capacity(n.min(1 << 16));
                for _ in 0..n {
                    values.push(self.cond_val()?);
                }
                let false_value = self.opt_str()?;
                Some((parent, values, false_value))
            }
            _ => return Err("invalid condition tag".into()),
        };
        let indexed_by = self.str_vec()?;
        let usage = self.usage()?;
        let sign = (self.bool()?, self.bool()?);
        let extra_flags = self.u16()?;
        let float_kind = match self.u8()? {
            0 => None,
            1 => Some(self.u16()?),
            _ => return Err("invalid float-kind tag".into()),
        };
        let odo_counter = self.opt_str()?;
        let renames = match self.u8()? {
            0 => None,
            1 => Some((self.str()?, self.str()?)),
            _ => return Err("invalid renames tag".into()),
        };
        let sync = self.bool()?;
        let external = self.bool()?;
        let occurs_key = match self.u8()? {
            0 => None,
            1 => Some(self.bool()?),
            _ => return Err("invalid occurs-key tag".into()),
        };
        Ok(ProgItem {
            level,
            name,
            pic,
            value,
            occurs,
            redefines,
            condition,
            indexed_by,
            usage,
            sign,
            extra_flags,
            float_kind,
            odo_counter,
            renames,
            sync,
            external,
            occurs_key,
        })
    }
    fn prog_item_vec(&mut self) -> Result<Vec<ProgItem>, String> {
        let n = self.u32()? as usize;
        let mut v = Vec::with_capacity(n.min(1 << 20));
        for _ in 0..n {
            v.push(self.prog_item()?);
        }
        Ok(v)
    }
    fn program_def(&mut self) -> Result<ProgramDef, String> {
        let ws = self.prog_item_vec()?;
        let linkage = self.prog_item_vec()?;
        let using = self.str_vec()?;
        let nf = self.u32()? as usize;
        let mut files = Vec::with_capacity(nf.min(1 << 16));
        for _ in 0..nf {
            files.push(self.file_def()?);
        }
        let nr = self.u32()? as usize;
        let mut reports = HashMap::new();
        for _ in 0..nr {
            let name = self.str()?;
            reports.insert(name, self.report_def()?);
        }
        let proc_toks = self.tok_vec()?;
        let nl = self.u32()? as usize;
        let mut proc_lines = Vec::with_capacity(nl.min(1 << 20));
        for _ in 0..nl {
            proc_lines.push(self.usize()?);
        }
        let is_initial = self.bool()?;
        let is_prototype = self.bool()?;
        Ok(ProgramDef {
            ws,
            linkage,
            using,
            files,
            reports,
            proc_toks,
            proc_lines,
            is_initial,
            is_prototype,
        })
    }
    fn dialect(&mut self) -> Result<crate::dialect::Dialect, String> {
        let binary_size = match self.u8()? {
            0 => crate::dialect::BinarySize::Cob1248,
            1 => crate::dialect::BinarySize::Cob248,
            2 => crate::dialect::BinarySize::Cob1to8,
            _ => return Err("invalid binary-size tag".into()),
        };
        let binary_truncate = self.bool()?;
        let complex_odo = self.bool()?;
        let odoslide = self.bool()?;
        let defaultbyte = match self.u8()? {
            0 => {
                let _ = self.u8()?;
                crate::dialect::DefaultByte::Init
            }
            1 => crate::dialect::DefaultByte::Fill(self.u8()?),
            _ => return Err("invalid defaultbyte tag".into()),
        };
        let move_ibm = self.bool()?;
        let init_justify = self.bool()?;
        Ok(crate::dialect::Dialect {
            binary_size,
            binary_truncate,
            complex_odo,
            odoslide,
            defaultbyte,
            move_ibm,
            init_justify,
        })
    }
    fn switch_env(&mut self) -> Result<SwitchEnv, String> {
        let mut states = [false; crate::common_misc::COB_SWITCH_COUNT];
        for st in states.iter_mut() {
            *st = self.bool()?;
        }
        let nc = self.u32()? as usize;
        let mut conds = HashMap::new();
        for _ in 0..nc {
            let k = self.str()?;
            let idx = self.usize()?;
            let on = self.bool()?;
            conds.insert(k, (idx, on));
        }
        let nm = self.u32()? as usize;
        let mut mnemonics = HashMap::new();
        for _ in 0..nm {
            let k = self.str()?;
            let idx = self.usize()?;
            mnemonics.insert(k, idx);
        }
        Ok(SwitchEnv {
            states: std::cell::RefCell::new(states),
            conds,
            mnemonics,
        })
    }
    fn program_map(&mut self) -> Result<HashMap<String, ProgramDef>, String> {
        let n = self.u32()? as usize;
        let mut m = HashMap::new();
        for _ in 0..n {
            let name = self.str()?;
            m.insert(name, self.program_def()?);
        }
        Ok(m)
    }
    fn str_map_string(&mut self) -> Result<HashMap<String, String>, String> {
        let n = self.u32()? as usize;
        let mut m = HashMap::new();
        for _ in 0..n {
            let k = self.str()?;
            m.insert(k, self.str()?);
        }
        Ok(m)
    }
    fn str_map_filedef(&mut self) -> Result<HashMap<String, FileDef>, String> {
        let n = self.u32()? as usize;
        let mut m = HashMap::new();
        for _ in 0..n {
            let k = self.str()?;
            m.insert(k, self.file_def()?);
        }
        Ok(m)
    }
    fn str_map_reportdef(&mut self) -> Result<HashMap<String, ReportDef>, String> {
        let n = self.u32()? as usize;
        let mut m = HashMap::new();
        for _ in 0..n {
            let k = self.str()?;
            m.insert(k, self.report_def()?);
        }
        Ok(m)
    }
}

/// Serialize a prepared program to the versioned wire format (payload + trailing checksum).
pub(crate) fn encode(p: &PreparedProgram) -> Vec<u8> {
    let mut e = Enc::new();
    e.bytes(MAGIC);
    e.u16(FORMAT_VERSION);
    e.str(p.compat);
    e.str(&p.source_hash);
    e.str(&p.expanded_hash);
    e.dialect(&p.dialect);
    e.u8(p.currency);
    e.bool(p.decimal_comma);
    match &p.collation {
        Some(c) => {
            e.u8(1);
            e.bytes(c);
        }
        None => e.u8(0),
    }
    e.str(&p.main_name);
    // probes
    e.u32(p.probes.len() as u32);
    for pr in &p.probes {
        e.str(&pr.phase);
        e.bool(pr.ok);
        e.str(&pr.diagnostic);
    }
    // switch env
    e.switch_env(&p.switches);
    // program map + derived maps (deterministic)
    e.program_map(&p.program_map);
    e.str_map_filedef(&p.file_defs);
    e.str_map(&p.record_files);
    e.str_map_reportdef(&p.reports);
    // trailing checksum over the whole payload so far
    let payload = e.out;
    let sum = crate::sha256::sha256_hex(&payload);
    let mut out = payload;
    out.extend_from_slice(sum.as_bytes());
    out
}

/// Deserialize + verify a prepared program. `expected_source_hash` enables stale-invalidation
/// checks (None = accept whatever identity the file carries).
pub(crate) fn decode(
    bytes: &[u8],
    expected_source_hash: Option<&str>,
) -> Result<PreparedProgram, String> {
    if bytes.len() < MAGIC.len() + 2 + CHECKSUM_LEN {
        return Err("prepared file too short".into());
    }
    let (payload, sum) = bytes.split_at(bytes.len() - CHECKSUM_LEN);
    let sum_str = std::str::from_utf8(sum).map_err(|_| "corrupt checksum".to_string())?;
    let expect = crate::sha256::sha256_hex(payload);
    if sum_str != expect {
        return Err("prepared file checksum mismatch (corruption)".to_string());
    }
    let mut d = Dec::new(payload)?;
    let magic = d.take(MAGIC.len())?;
    if magic != MAGIC {
        return Err("not a prepared-program file (bad magic)".to_string());
    }
    let ver = d.u16()?;
    if ver != FORMAT_VERSION {
        return Err(format!(
            "prepared-program format v{ver} unsupported (expected v{FORMAT_VERSION})"
        ));
    }
    let compat = d.str()?;
    if compat != "prepared-v1" {
        return Err(format!(
            "prepared-program compat {compat:?} != prepared-v1 (front-end changed; re-prepare)"
        ));
    }
    let source_hash = d.str()?;
    let expanded_hash = d.str()?;
    if let Some(want) = expected_source_hash {
        if want != source_hash {
            return Err(format!(
                "prepared-program source hash mismatch (stale: file {source_hash}, want {want})"
            ));
        }
    }
    let dialect = d.dialect()?;
    let currency = d.u8()?;
    let decimal_comma = d.bool()?;
    let collation = match d.u8()? {
        0 => None,
        1 => Some(<[u8; 256]>::try_from(d.take(256)?).map_err(|_| "collation length")?),
        _ => return Err("invalid collation tag".into()),
    };
    let main_name = d.str()?;
    let np = d.u32()? as usize;
    let mut probes = Vec::with_capacity(np.min(1 << 16));
    for _ in 0..np {
        let phase = d.str()?;
        let ok = d.bool()?;
        let diagnostic = d.str()?;
        probes.push(PhaseProbe {
            phase,
            ok,
            diagnostic,
        });
    }
    let switches = d.switch_env()?;
    let program_map = d.program_map()?;
    let file_defs = d.str_map_filedef()?;
    let record_files = d.str_map_string()?;
    let reports = d.str_map_reportdef()?;
    // the reader must consume the whole payload (trailing garbage = corrupt)
    if d.pos != payload.len() {
        return Err("trailing garbage after prepared payload".to_string());
    }
    Ok(PreparedProgram {
        source_hash,
        expanded_hash,
        dialect,
        compat: "prepared-v1",
        probes,
        program_map,
        currency,
        decimal_comma,
        switches,
        collation,
        main_name,
        file_defs,
        record_files,
        reports,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PreparedProgram {
        // A program exercising WS items, an FD + records, a report, an 88, an OCCURS table and a
        // performed loop so the codec round-trips every persisted shape.
        let src = concat!(
            "       IDENTIFICATION DIVISION.\n",
            "       PROGRAM-ID. T.\n",
            "       DATA DIVISION.\n",
            "       FILE SECTION.\n",
            "       FD  OUTFILE.\n",
            "       01  OUT-REC PIC X(20).\n",
            "       WORKING-STORAGE SECTION.\n",
            "       01  I PIC 9(3) COMP-3.\n",
            "       01  TBL OCCURS 5 TIMES INDEXED BY IX.\n",
            "           05  ELEM PIC 9(4).\n",
            "       01  FLAG PIC X.\n",
            "           88  DONE VALUE \"Y\".\n",
            "       01  G.\n",
            "           05  A PIC X(4).\n",
            "           05  B REDEFINES A PIC 9(4).\n",
            "       01  F PIC 9(2)V9(2).\n",
            "       01  N PIC S9(5).\n",
            "       REPORT SECTION.\n",
            "       RD  RPT1 CONTROLS FINAL PAGE LIMIT 60.\n",
            "       01  TYPE RH.\n",
            "           05  LINE 1 COLUMN 1 PIC X(5) VALUE \"HEAD\".\n",
            "       01  DET TYPE DETAIL.\n",
            "           05  LINE 1 COLUMN 2 PIC Z(3)9 SOURCE N.\n",
            "       PROCEDURE DIVISION.\n",
            "           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 5\n",
            "               MOVE I TO ELEM (I)\n",
            "           END-PERFORM\n",
            "           DISPLAY N\n",
            "           STOP RUN.\n",
        );
        prepare_program(src, crate::dialect::Dialect::DEFAULT).expect("prepare sample")
    }

    #[test]
    fn round_trip_preserves_behavior() {
        let p = sample();
        let bytes = encode(&p);
        // deterministic: encoding twice gives identical bytes
        assert_eq!(bytes, encode(&p));
        let q = decode(&bytes, Some(&p.source_hash)).expect("decode");
        assert_eq!(q.source_hash, p.source_hash);
        assert_eq!(q.expanded_hash, p.expanded_hash);
        assert_eq!(q.main_name, p.main_name);
        assert_eq!(q.probes.len(), p.probes.len());
        // cache-off equivalence: fresh run == loaded run
        let (o1, _pr1, rc1) = p.run(false).expect("fresh run");
        let (o2, _pr2, rc2) = q.run(false).expect("loaded run");
        assert_eq!(o1, o2);
        assert_eq!(rc1, rc2);
    }

    #[test]
    fn corruption_is_detected() {
        let p = sample();
        let mut bytes = encode(&p);
        let n = bytes.len();
        // flip a payload byte (never the checksum)
        bytes[n - CHECKSUM_LEN - 1] ^= 0xFF;
        assert!(decode(&bytes, Some(&p.source_hash)).is_err());
        // truncation
        let short = &bytes[..bytes.len() - CHECKSUM_LEN - 1];
        assert!(decode(short, Some(&p.source_hash)).is_err());
    }

    #[test]
    fn stale_source_is_rejected() {
        let p = sample();
        let bytes = encode(&p);
        let wrong = crate::sha256::sha256_hex(b"some other source");
        assert!(decode(&bytes, Some(&wrong)).is_err());
        // no expectation -> accepts (identity carried in the file)
        assert!(decode(&bytes, None).is_ok());
    }

    #[test]
    fn magic_and_version_are_checked() {
        let p = sample();
        let mut bytes = encode(&p);
        bytes[0] ^= 0xFF;
        assert!(decode(&bytes, None).is_err());
        bytes[0] ^= 0xFF;
        // corrupt the format version (byte MAGIC.len()..+2)
        bytes[MAGIC.len()] = 99;
        assert!(decode(&bytes, None).is_err());
    }

    #[test]
    fn atomic_save_and_load_round_trip() {
        let p = sample();
        let dir =
            std::env::temp_dir().join(format!("gnucobol-rs-prepared-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("prepared.bin");
        p.save(&path).expect("save");
        let q = PreparedProgram::load(&path, Some(&p.source_hash)).expect("load");
        let (o1, _p1, rc1) = p.run(false).expect("fresh");
        let (o2, _p2, rc2) = q.run(false).expect("loaded");
        assert_eq!(o1, o2);
        assert_eq!(rc1, rc2);
        // corruption on disk is caught at load
        let mut bytes = std::fs::read(&path).expect("read");
        let n = bytes.len();
        bytes[n - CHECKSUM_LEN - 1] ^= 0x01;
        std::fs::write(&path, &bytes).expect("corrupt");
        assert!(PreparedProgram::load(&path, Some(&p.source_hash)).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
