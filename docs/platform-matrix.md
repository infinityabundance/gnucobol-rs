# Platform matrix — no silent portability claim

GnuCOBOL runs on Linux, BSD, proprietary Unix, macOS, and Windows. `gnucobol-rs` parity is
claimed **only** on the platform whose oracle was built and recorded. Other platforms are
future work, never silently implied.

| Platform | Status |
|----------|--------|
| linux-glibc (`x86_64-pc-linux-gnu`) | **admitted** (this oracle) |
| linux-musl | future |
| macOS | future |
| MinGW / Windows | future — distinct traps (see below) |
| MSVC / Windows | future / likely non-claim |
| *BSD | future |

## Windows is not "just line endings" (future traps to receipt)

CRLF source handling; console encoding; path separator + drive colon; DLL lookup;
`COB_CONFIG_DIR` colon-in-path parsing; `.exe` suffix; file sharing/locking; text-vs-binary
mode; MinGW-vs-MSVC build differences.

## Endianness / charset

This oracle is **little-endian, ASCII host** (`COB_EBCDIC_MACHINE` off). Big-endian and
EBCDIC-host behaviours (notably EBCDIC zoned-sign processing) are classified out of current
claims; binary-field byte order is a future per-attr concern.
