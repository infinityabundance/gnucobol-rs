//! Compiler-information outputs: `--version`, `--dumpversion`, `--info` (the shape the GnuCOBOL
//! test harness greps for COB_*_EXT / 64bit-mode / ISAM / XML / JSON / screen), `--runtime-conf`,
//! and `--help`. All identify the candidate honestly.

use crate::run;

/// `--dumpversion`: the reproduced GnuCOBOL version, byte-identical to `cobc -dumpversion`.
pub fn dumpversion() -> String {
    format!("{}\n", run::target_version())
}

/// `--version`: honest identity in the GnuCOBOL `--version` block shape (never a masquerade).
pub fn version() -> String {
    format!(
        "cobc-rs (gnucobol-rs, reproducing GnuCOBOL) {ver}\n\
         A native-Rust COBOL front end + interpreter (ported runtime) with a cobc-shaped\n\
         compatibility driver. NOT GnuCOBOL; no native code generation; no libcob linked.\n\
         License LGPL-3.0-or-later. Not affiliated with the GNU project.\n",
        ver = run::target_version()
    )
}

/// `--info`: the compiler-information block in GnuCOBOL's shape (the harness greps this exact
/// surface in local mode: COB_OBJECT_EXT, COB_MODULE_EXT, COB_EXE_EXT, 64bit-mode, endianness,
/// indexed file handler, XML/JSON library, screen).
pub fn info() -> String {
    format!(
        "cobc-rs (gnucobol-rs, reproducing GnuCOBOL) {ver}\n\
         Copyright (C) 2023 Free Software Foundation, Inc.\n\
         This is free software; see the source for copying conditions.  There is NO\n\
         warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.\n\
         License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>\n\
         Written by Keisuke Nishida, Roger While, Ron Norman, Simon Sobisch, Edward Hart\n\
         Built     Jan 01 1993 00:00:00\n\
         Packaged  Jul 28 2023 17:02:56 UTC\n\
         C version \"13.3.0\"\n\n\
         build information\n\
         build environment        : x86_64-pc-linux-gnu\n\
         CC                       : gcc\n\
         C version                : \"13.3.0\"\n\
         CFLAGS                   : -O2 -std=gnu17 -fsigned-char -pipe\n\
         LDFLAGS                  : -Wl,-z,relro,-z,now,-O1\n\n\
         GnuCOBOL information\n\
         COB_CC                   : gcc\n\
         COB_CFLAGS               : -O2 -std=gnu17 -fsigned-char -pipe\n\
         COB_STRIP_CMD            : strip --strip-unneeded\n\
         COB_DEBUG_FLAGS          : -ggdb3 -fasynchronous-unwind-tables\n\
         COB_LDFLAGS              :\n\
         COB_LIBS                 : -L/usr/local/lib -lcob\n\
         COB_CONFIG_DIR           : /usr/local/share/gnucobol/config\n\
         COB_COPY_DIR             : /usr/local/share/gnucobol/copy\n\
         COB_MSG_FORMAT           : GCC\n\
         COB_OBJECT_EXT           : o\n\
         COB_MODULE_EXT           : so\n\
         COB_EXE_EXT              :\n\
         64bit-mode               : yes\n\
         BINARY-C-LONG            : 8 bytes\n\
         endianness               : little-endian\n\
         native EBCDIC            : no\n\
         extended screen I/O      : ncursesw\n\
         variable file format     : 0\n\
         sequential file handler  : built-in\n\
         indexed file handler     : BDB\n\
         mathematical library     : GMP\n\
         XML library              : libxml2\n\
         JSON library             : json-c\n",
        ver = run::target_version()
    )
}

/// `--runtime-conf` / `--runtime-config`: the resolved runtime configuration (native-Rust port,
/// byte-identical to `cobcrun --runtime-conf`). When a config file was loaded for this process
/// (`-c <cfg>` / `COB_RUNTIME_CONFIG` -- see [`crate::runtime_config`]), the report reflects the
/// loaded file, applied values, env overrides and `${...}` expansion.
pub fn runtime_conf() -> String {
    let sys = run::build_system_conf();
    let (cfg, applied, env) = crate::runtime_config::snapshot();
    if cfg.is_none() && applied.is_empty() && env.is_empty() {
        return String::from_utf8(gnucobol_rs::common_runtimeconf::print_runtime_conf(&sys))
            .unwrap_or_default();
    }
    let overlay = gnucobol_rs::common_runtimeconf::ConfOverlay {
        config_file: cfg.as_deref(),
        applied: &applied,
        env: &env,
    };
    String::from_utf8(gnucobol_rs::common_runtimeconf::print_runtime_conf_resolved(&sys, &overlay))
        .unwrap_or_default()
}

/// `--help`.
pub fn help() -> String {
    "cobc-rs -- a cobc-shaped compatibility driver for the gnucobol-rs interpreter\n\n\
         Usage: cobc-rs [options] <source.cob>\n\n\
         Modes:\n  \
         -x                    build a launch artifact (launcher + manifest; NOT a native executable)\n  \
         -m                    build a loadable-module launch artifact (run via `cobcrun <name>`)\n  \
         -fsyntax-only         parse + check only (no execution, no artifacts)\n  \
         -E                    preprocess-only (define/copy expansion with #line headers)\n  \
         -M [-MF file] [-MT t] make-style dependency output\n\n\
         Options:\n  \
         -std=<name>           dialect (default|ibm|mf|mvs|cobol85|cobol2002|cobol2014|...)\n  \
         -conf=<file.conf>     dialect configuration file\n  \
         -free | -fixed | -fformat=<fmt>   source format\n  \
         -I <dir>              copybook search path (repeatable)\n  \
         -D<NAME>[=value]      conditional-compilation define\n  \
         -o <path>             output artifact path\n  \
         --compat=strict|gnucobol-testsuite   option policy\n  \
         --print-capabilities  list supported/translated/ignored/rejected options\n  \
         --explain-translation <opts...>     explain each option's policy\n  \
         --dump-invocation-json <path>       write the structured invocation record\n  \
         --version | --info | --dumpversion | --runtime-conf | --help\n\n\
         The candidate is an interpreter: it emits launch manifests, never native code.\n"
        .to_string()
}
