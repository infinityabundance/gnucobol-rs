# Stable 3.2 -> Git baseline mapping

1. **match an official release tag**: no annotated tags in the OCamlPro repo; the branch commit 645b417 'GnuCOBOL 3.2 - 20230728' is the 3.2 GA release point
2. **match a git tree after excluding release-generated files**: 273/274 common files byte-identical to 645b417; only configure.ac differs (CVS $Revision$ keyword)
3. **match file-by-file content hashes**: identical for all source files (cobc/, libcob/, bin/, tests/, config/)
4. **nearest historical commit + release packaging delta**: 645b417 + 55 release-generated files (Bison/Flex, autotools, man pages) - 120 repo-only files (build_windows/ etc.)
5. **synthetic source-tree baseline manifest**: not needed (steps 2-4 exact)

Baseline commit: `645b417` (GnuCOBOL 3.2 GA, r5150). The tarball is the release packaging of that commit; no synthetic baseline needed.
