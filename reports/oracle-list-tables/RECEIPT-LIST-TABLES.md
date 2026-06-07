# Oracle self-description tables (GNURUST.LISTTABLES.0)

GnuCOBOL describes itself. Rather than retype reserved-word / intrinsic / mnemonic / exception
/ system-routine tables (which drift by version and dialect), `gnucobol-rs` **harvests** them
from the admitted oracle and hashes them. These hashes are part of the oracle identity; future
parser/runtime courts admit them as dialect tables (`GNURUST.RESERVED.0`, `GNURUST.INTRINSIC.0`,
`GNURUST.EXCSTATUS.0`, …), never hand-maintained.

Captured from the built `cobc (GnuCOBOL) 3.2.0` under `LC_ALL=C.UTF-8`, default dialect:

| Table | `cobc` flag | lines | sha256 |
|-------|-------------|-------|--------|
| reserved words | `--list-reserved` | 1004 | `a6f568bfe08662db125ded8bd297daa59b2d0565145995c9111d0da25f2e1428` |
| intrinsic functions | `--list-intrinsics` | 116 | `125401691bbc6c487c088b78c0c8b6d205b4c887251bf97ef3106c7cc945a228` |
| mnemonic names | `--list-mnemonics` | 76 | `21984c5e7a85dca89b2b6aac558542e18c605f8fed3e72db5c6cd1b261edaf7b` |
| exception names | `--list-exceptions` | 177 | `728fdf1c041e8715ea4075633b48c0718a2c28844bb77048e74319c3a57ea286` |
| system routines | `--list-system` | 65 | `7fe204973da7f7cf85d5412fe4fbe09c72fdb9f07d78a3ec46c43cab7e0b43f4` |

The `.txt` files alongside this receipt are the captured tables. They are reference artifacts,
not a parity claim: the current decimal slice does not parse source, so it uses none of them —
they are admitted now so future courts cannot silently hand-roll these lists.
