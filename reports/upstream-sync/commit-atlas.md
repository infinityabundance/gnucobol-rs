# GnuCOBOL upstream commit atlas — stable 3.2 baseline → current head

- range: `645b417..5568b8fc770ff310e5017300d561d8f3deec257c`
- admit repo: `lab/admit/gnucobol-upstream-current/`
- rows: 367 (matches `git rev-list` count)
- merges: 122; non-merge: 245; first-parent chain: 160
- curated semantic entries: 182

## Status totals

| status | count |
|---|---|
| UPSTREAM_MERGE_ACCOUNTED | 122 |
| NOT_APPLICABLE_WITH_PROOF | 65 |
| CI_ONLY_ACCOUNTED | 54 |
| RUNTIME_PORTED | 36 |
| FRONTEND_REIMPLEMENTED | 22 |
| TEST_IMPORTED | 17 |
| CONFIGURATION_INTEGRATED | 16 |
| BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | 10 |
| WRAPPER_INTEGRATED | 9 |
| HARNESS_ADOPTED | 9 |
| DOCUMENTATION_TRACKED | 5 |
| PLATFORM_BEHAVIOR_INTEGRATED | 2 |

## Integrity checks

- `row_count_matches_rev_list`: PASS
- `no_duplicate_shas`: PASS
- `all_rows_have_enum_status`: PASS
- `all_merges_accounted_as_merges`: PASS
- `no_uncurated_semantic_commit`: PASS
- `all_override_shas_in_range`: PASS

## First-parent chain (chronological)

| # | commit | date | status | subject |
|---|---|---|---|---|
| 1 | `5568b8fc770f` | 2026-08-04 | CI_ONLY_ACCOUNTED | Fix and update CI |
| 2 | `568531bd417a` | 2026-06-09 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x |
| 3 | `a3accbe7616c` | 2026-05-19 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x |
| 4 | `871f965fac83` | 2026-05-05 | CI_ONLY_ACCOUNTED | workflow updates |
| 5 | `326ce553416e` | 2026-03-23 | CI_ONLY_ACCOUNTED | windows-msvc: fix env var reference for dependencies |
| 6 | `253e5cc2602e` | 2026-01-27 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x |
| 7 | `3457bd5def5e` | 2025-12-24 | CI_ONLY_ACCOUNTED | MSVC CI update |
| 8 | `fb8f358e91ce` | 2025-12-12 | CI_ONLY_ACCOUNTED | Update windows-msvc.yml |
| 9 | `da31b9286647` | 2025-12-08 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x |
| 10 | `bc234cd17f19` | 2025-12-03 | CI_ONLY_ACCOUNTED | Adjust MSYS2 CI timeout |
| 11 | `1fc514e9d166` | 2025-12-03 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #256 from ddeclerck/fix_ci |
| 12 | `deeadffbafb7` | 2025-11-20 | CI_ONLY_ACCOUNTED | Update windows-msvc.yml |
| 13 | `d28f9fab8e27` | 2025-11-20 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #255 from nberth/update-ci-branch |
| 14 | `eda8905e4404` | 2025-11-19 | CI_ONLY_ACCOUNTED | Improve MSYS1 workflow definition |
| 15 | `b1275d4ee475` | 2025-11-19 | CI_ONLY_ACCOUNTED | Fix MacOS workflow |
| 16 | `5e45c5e64f37` | 2025-11-18 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #254 from OCamlPro/gnucobol-3.x |
| 17 | `31ba95f7a4c3` | 2025-11-18 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #253 from OCamlPro/gnucobol-3.x |
| 18 | `db111f65bd01` | 2025-11-06 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #251 from OCamlPro/gnucobol-3.x |
| 19 | `1fb152e8b536` | 2025-10-21 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 20 | `94efcf215788` | 2025-10-20 | CI_ONLY_ACCOUNTED | Fix CI |
| 21 | `f50ab5754982` | 2025-07-31 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 22 | `f8a1a5c2766c` | 2025-07-30 | CI_ONLY_ACCOUNTED | Add IBM POWER and Z CI |
| 23 | `33057ad3e052` | 2025-07-30 | CI_ONLY_ACCOUNTED | Update MacOS CI (DB4 removal imminent) |
| 24 | `d8bd3f3a02c8` | 2025-07-29 | CI_ONLY_ACCOUNTED | Adjust MSYS2 workflow timeout (was slightly too short) |
| 25 | `adf35557a63a` | 2025-07-29 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 26 | `a006789fa627` | 2025-07-21 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 27 | `a4be0beded8f` | 2025-07-17 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #235 from OCamlPro/gnucobol-3.x |
| 28 | `d3a3a3e6102f` | 2025-05-22 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #232 from ddeclerck/add_32bit_ci |
| 29 | `1b2c19e0cae8` | 2025-05-20 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 30 | `0e2b41f2c63b` | 2025-05-16 | CI_ONLY_ACCOUNTED | CI adjustments |
| 31 | `8c96392229ca` | 2025-05-15 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #226 from ddeclerck/gc3_ci_update |
| 32 | `dca6c3e5ec0b` | 2025-05-13 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 33 | `84359ec81a15` | 2025-04-16 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 34 | `cdb87a8b3aa9` | 2025-04-07 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 35 | `c48824511397` | 2025-03-31 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 36 | `0cb3eab5945e` | 2025-03-28 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 37 | `080e75630cc4` | 2025-03-26 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 38 | `85c708085fad` | 2025-02-16 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 39 | `369eb24f947e` | 2025-02-12 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 40 | `6cc5a5803005` | 2025-02-11 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #217 from ddeclerck/fix_msys2_ci |
| 41 | `3fb682569d9e` | 2025-01-28 | CI_ONLY_ACCOUNTED | Fix MacOS CI |
| 42 | `6698c4bf0e94` | 2025-01-19 | CI_ONLY_ACCOUNTED | Fix Ubuntu CI (Coverage) |
| 43 | `beeace4cded3` | 2025-01-13 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 44 | `731b81a327fe` | 2025-01-07 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 45 | `c4fbcc3050ad` | 2025-01-03 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 46 | `3f34a2461a6a` | 2024-12-30 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 47 | `5086fdf05e34` | 2024-12-20 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 48 | `afb68b34db69` | 2024-12-19 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 49 | `47501705ee02` | 2024-12-16 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 50 | `d5abb19870d0` | 2024-12-10 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #202 from OCamlPro/GitMensch-patch-1 |
| 51 | `129abba07f9c` | 2024-12-09 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 52 | `1be8d3f3493c` | 2024-12-06 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 53 | `16e84a584d7e` | 2024-11-24 | CI_ONLY_ACCOUNTED | Fix macOS CI |
| 54 | `04916dd9371e` | 2024-11-22 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 55 | `3edbc6d1f70b` | 2024-10-31 | NOT_APPLICABLE_WITH_PROOF | Update .gitpod.yml |
| 56 | `0cc8207d14de` | 2024-10-11 | TEST_IMPORTED | follow-up to r5356 - fixed skip via atlocal_win |
| 57 | `68b82c88f548` | 2024-10-11 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 58 | `a23f0dc875f6` | 2024-10-07 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #189 from OCamlPro/ci-update |
| 59 | `799c61376739` | 2024-10-02 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 60 | `61ffca26726a` | 2024-09-30 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 61 | `482206f49af4` | 2024-09-27 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #184 from OCamlPro/ci-minimal-build |
| 62 | `f5989ba77c79` | 2024-09-26 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 63 | `2c000f51a11e` | 2024-09-22 | CI_ONLY_ACCOUNTED | Update Windows workflows (upload testsuite.log on failure) |
| 64 | `e807aed2c9c9` | 2024-09-22 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 65 | `015735daa9c5` | 2024-09-20 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 66 | `f36f1506a16d` | 2024-09-19 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 67 | `fe28973b030f` | 2024-08-28 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #170 from nberth/ci-adjustments-4-gcos4gnucobol-3.x |
| 68 | `d3c4e188dd30` | 2024-08-28 | CI_ONLY_ACCOUNTED | Update MSVC CI |
| 69 | `0226a4e160d3` | 2024-08-28 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 70 | `f13eede92da6` | 2024-08-28 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 71 | `d0ef5aa6e124` | 2024-08-23 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 72 | `2ed1057b1d86` | 2024-08-22 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 73 | `8b7350a27a7a` | 2024-08-17 | NOT_APPLICABLE_WITH_PROOF | Update .gitpod.yml |
| 74 | `56a19214a986` | 2024-08-12 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 75 | `fabaca953a23` | 2024-08-07 | NOT_APPLICABLE_WITH_PROOF | Update .gitignore |
| 76 | `04fe8aaf44d7` | 2024-08-04 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 77 | `a672fbba0388` | 2024-08-01 | CI_ONLY_ACCOUNTED | Update MSVC & MSYS1 CI |
| 78 | `00f6832684d8` | 2024-07-26 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 79 | `4d51096de843` | 2024-07-25 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #140 from ddeclerck/ci_msvc |
| 80 | `de6053aad234` | 2024-07-12 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 81 | `39344cf66085` | 2024-06-20 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 82 | `7225e55ddb07` | 2024-06-20 | NOT_APPLICABLE_WITH_PROOF | Add gitpod configuration |
| 83 | `21b5d516ffd3` | 2024-05-16 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 84 | `4907e0d0683e` | 2024-05-14 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #145 from ddeclerck/fix_macos_ci |
| 85 | `70b4076e9e8d` | 2024-05-14 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 86 | `5ba97ae7594f` | 2024-05-13 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into HEAD |
| 87 | `87c4fb2905ed` | 2024-04-27 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 88 | `2e620aa926b6` | 2024-04-22 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 89 | `89c45a3bc80c` | 2024-03-13 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 90 | `a2a51fd5ea7f` | 2024-03-13 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 91 | `57133577c7e5` | 2024-02-20 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 92 | `db7db96f8c08` | 2024-02-19 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 93 | `4eece0f7ddcc` | 2024-02-03 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 94 | `2ff35a3e3725` | 2024-01-31 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 95 | `824f2a6445e0` | 2024-01-26 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 96 | `f059c849512a` | 2024-01-22 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 97 | `98a5c787c1e5` | 2024-01-11 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 98 | `c0d64addfd83` | 2023-08-15 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 99 | `0ab36fd83692` | 2023-07-26 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 100 | `6b4405108a30` | 2023-07-11 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 101 | `2a101a4bffdb` | 2023-07-09 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 102 | `a4abf11bf1cc` | 2023-07-05 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 103 | `4b8452abf1ca` | 2023-07-03 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 104 | `57e7a2851308` | 2023-07-02 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 105 | `743651ffd971` | 2023-06-23 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 106 | `cb5f7fa19a6c` | 2023-06-21 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 107 | `8caf9b25a444` | 2023-06-20 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 108 | `2dc28255dbc8` | 2023-06-20 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 109 | `7ba3fb5bf898` | 2023-06-13 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 110 | `8a0796d9f09a` | 2023-06-13 | CI_ONLY_ACCOUNTED | CI: check for c89 declaration (#97) |
| 111 | `cca51ff27837` | 2023-06-02 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 112 | `1992539c4fde` | 2023-05-24 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 113 | `fcd562f0f1cc` | 2023-04-20 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 114 | `7981d8aff1b1` | 2023-04-12 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 115 | `3d48698fb76e` | 2023-02-21 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 116 | `bdd8837832d4` | 2023-02-20 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 117 | `2ca79d6ae9dd` | 2023-02-11 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 118 | `608eac1800e0` | 2023-02-10 | NOT_APPLICABLE_WITH_PROOF | Recommit ar-lib |
| 119 | `200150f627aa` | 2023-02-10 | NOT_APPLICABLE_WITH_PROOF | remove ar-lib from GIT |
| 120 | `b63bf71a12b4` | 2023-02-08 | NOT_APPLICABLE_WITH_PROOF | Add autofonce configuration |
| 121 | `d4f364e44ee1` | 2023-02-02 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 122 | `7fd5d4936b2b` | 2023-02-01 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 123 | `9df05779b4cd` | 2023-01-30 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 124 | `d15d131af589` | 2023-01-28 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 125 | `db33c36f7670` | 2023-01-26 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 126 | `f550c9b865f1` | 2023-01-23 | CI_ONLY_ACCOUNTED | Add working Github action files, except for Windows (#79) |
| 127 | `516b813b3ca0` | 2023-01-19 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 128 | `91b7781e801f` | 2022-12-15 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 129 | `fe93f0d3b36d` | 2022-12-13 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 130 | `5a1ed6ffe5bd` | 2022-12-08 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 131 | `757ff9b2f769` | 2022-12-06 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 132 | `62dd11b8b92d` | 2022-12-05 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 133 | `d3c0b2f18bfc` | 2022-11-18 | CI_ONLY_ACCOUNTED | improving MacOS CI |
| 134 | `173790f1ef60` | 2022-11-15 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 135 | `0a62bc915695` | 2022-11-15 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 136 | `0ac36ad45763` | 2022-11-08 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 137 | `1c7abb41b185` | 2022-10-05 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 138 | `6ff12e91dc10` | 2022-10-04 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 139 | `87051279e6a6` | 2022-09-30 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 140 | `24f2a1036db8` | 2022-09-30 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #62 from nberth/fix-ci |
| 141 | `a5ea0d5c93ec` | 2022-09-27 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 142 | `41e53b7db427` | 2022-09-25 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 143 | `d7db95f77395` | 2022-09-22 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 144 | `1c3569d3a8d3` | 2022-09-20 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 145 | `cc3f6677dd17` | 2022-09-01 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 146 | `6fec839eae01` | 2022-08-25 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 147 | `a913f3c96dec` | 2022-07-29 | UPSTREAM_MERGE_ACCOUNTED | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| 148 | `e2505a60f8be` | 2022-07-27 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 149 | `e07100c7ce2a` | 2022-07-25 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 150 | `4c04ed86e837` | 2022-07-22 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #54 from nberth/fix-ci |
| 151 | `6a24f9a47e65` | 2022-07-08 | CI_ONLY_ACCOUNTED | Disable automated windows CI workflow |
| 152 | `8c6cafb8b4a4` | 2022-07-07 | UPSTREAM_MERGE_ACCOUNTED | Merge pull request #41 from nberth/fix-ci |
| 153 | `a070a64abef8` | 2022-07-07 | NOT_APPLICABLE_WITH_PROOF | Update .gitignore |
| 154 | `c8c789911a52` | 2022-07-07 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 155 | `5b54e1c993e9` | 2022-06-16 | UPSTREAM_MERGE_ACCOUNTED | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| 156 | `197398dfd15a` | 2022-05-19 | CI_ONLY_ACCOUNTED | Improve setup for CI jobs, with temporary focus on branch `gcos4gnucobol-3.x` |
| 157 | `a86cb1055aeb` | 2022-03-29 | DOCUMENTATION_TRACKED | Thanks OCamlPro contributors |
| 158 | `8ea9ac449c98` | 2022-01-28 | NOT_APPLICABLE_WITH_PROOF | Redispatch ChangeLog entries |
| 159 | `289c9aef58a9` | 2022-02-04 | CONFIGURATION_INTEGRATED | [GCOS] Add GCOS configuration file |
| 160 | `27788c5941de` | 2022-02-04 | CI_ONLY_ACCOUNTED | GIT-specific settings, with CI setup and github workflow for Ubuntu, Windows and Macos |

## Merges (accounted)

| commit | date | subject |
|---|---|---|
| `5b54e1c993e9` | 2022-06-16 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `c8c789911a52` | 2022-07-07 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `8c6cafb8b4a4` | 2022-07-07 | Merge pull request #41 from nberth/fix-ci |
| `4c04ed86e837` | 2022-07-22 | Merge pull request #54 from nberth/fix-ci |
| `e07100c7ce2a` | 2022-07-25 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `e2505a60f8be` | 2022-07-27 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `a913f3c96dec` | 2022-07-29 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `6fec839eae01` | 2022-08-25 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `cc3f6677dd17` | 2022-09-01 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `1c3569d3a8d3` | 2022-09-20 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `d7db95f77395` | 2022-09-22 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `41e53b7db427` | 2022-09-25 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `a5ea0d5c93ec` | 2022-09-27 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `24f2a1036db8` | 2022-09-30 | Merge pull request #62 from nberth/fix-ci |
| `87051279e6a6` | 2022-09-30 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `6ff12e91dc10` | 2022-10-04 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `1c7abb41b185` | 2022-10-05 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `0ac36ad45763` | 2022-11-08 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `0a62bc915695` | 2022-11-15 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `173790f1ef60` | 2022-11-15 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `62dd11b8b92d` | 2022-12-05 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `757ff9b2f769` | 2022-12-06 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `5a1ed6ffe5bd` | 2022-12-08 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `fe93f0d3b36d` | 2022-12-13 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `91b7781e801f` | 2022-12-15 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `516b813b3ca0` | 2023-01-19 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `db33c36f7670` | 2023-01-26 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `d15d131af589` | 2023-01-28 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `9df05779b4cd` | 2023-01-30 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `7fd5d4936b2b` | 2023-02-01 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `d4f364e44ee1` | 2023-02-02 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `2ca79d6ae9dd` | 2023-02-11 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `bdd8837832d4` | 2023-02-20 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `3d48698fb76e` | 2023-02-21 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `7981d8aff1b1` | 2023-04-12 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `fcd562f0f1cc` | 2023-04-20 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `1992539c4fde` | 2023-05-24 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `cca51ff27837` | 2023-06-02 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `7ba3fb5bf898` | 2023-06-13 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `2dc28255dbc8` | 2023-06-20 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `8caf9b25a444` | 2023-06-20 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `cb5f7fa19a6c` | 2023-06-21 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `743651ffd971` | 2023-06-23 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `57e7a2851308` | 2023-07-02 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `4b8452abf1ca` | 2023-07-03 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `a4abf11bf1cc` | 2023-07-05 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `2a101a4bffdb` | 2023-07-09 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `6b4405108a30` | 2023-07-11 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `0ab36fd83692` | 2023-07-26 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `c0d64addfd83` | 2023-08-15 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `98a5c787c1e5` | 2024-01-11 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `f059c849512a` | 2024-01-22 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `824f2a6445e0` | 2024-01-26 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `2ff35a3e3725` | 2024-01-31 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `4eece0f7ddcc` | 2024-02-03 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `db7db96f8c08` | 2024-02-19 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `57133577c7e5` | 2024-02-20 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `a2a51fd5ea7f` | 2024-03-13 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `89c45a3bc80c` | 2024-03-13 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `2e620aa926b6` | 2024-04-22 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `87c4fb2905ed` | 2024-04-27 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `5ba97ae7594f` | 2024-05-13 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into HEAD |
| `70b4076e9e8d` | 2024-05-14 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `4907e0d0683e` | 2024-05-14 | Merge pull request #145 from ddeclerck/fix_macos_ci |
| `21b5d516ffd3` | 2024-05-16 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `39344cf66085` | 2024-06-20 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `de6053aad234` | 2024-07-12 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `4d51096de843` | 2024-07-25 | Merge pull request #140 from ddeclerck/ci_msvc |
| `00f6832684d8` | 2024-07-26 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `04fe8aaf44d7` | 2024-08-04 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `56a19214a986` | 2024-08-12 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `2ed1057b1d86` | 2024-08-22 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `d0ef5aa6e124` | 2024-08-23 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `f13eede92da6` | 2024-08-28 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `0226a4e160d3` | 2024-08-28 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `fe28973b030f` | 2024-08-28 | Merge pull request #170 from nberth/ci-adjustments-4-gcos4gnucobol-3.x |
| `f36f1506a16d` | 2024-09-19 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `015735daa9c5` | 2024-09-20 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `e807aed2c9c9` | 2024-09-22 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `f5989ba77c79` | 2024-09-26 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `482206f49af4` | 2024-09-27 | Merge pull request #184 from OCamlPro/ci-minimal-build |
| `61ffca26726a` | 2024-09-30 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `799c61376739` | 2024-10-02 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `a23f0dc875f6` | 2024-10-07 | Merge pull request #189 from OCamlPro/ci-update |
| `68b82c88f548` | 2024-10-11 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `04916dd9371e` | 2024-11-22 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `1be8d3f3493c` | 2024-12-06 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `129abba07f9c` | 2024-12-09 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `d5abb19870d0` | 2024-12-10 | Merge pull request #202 from OCamlPro/GitMensch-patch-1 |
| `47501705ee02` | 2024-12-16 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `afb68b34db69` | 2024-12-19 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `5086fdf05e34` | 2024-12-20 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `3f34a2461a6a` | 2024-12-30 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `c4fbcc3050ad` | 2025-01-03 | Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x |
| `731b81a327fe` | 2025-01-07 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `beeace4cded3` | 2025-01-13 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `6cc5a5803005` | 2025-02-11 | Merge pull request #217 from ddeclerck/fix_msys2_ci |
| `369eb24f947e` | 2025-02-12 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `85c708085fad` | 2025-02-16 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `080e75630cc4` | 2025-03-26 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `0cb3eab5945e` | 2025-03-28 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `c48824511397` | 2025-03-31 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `cdb87a8b3aa9` | 2025-04-07 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `84359ec81a15` | 2025-04-16 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `dca6c3e5ec0b` | 2025-05-13 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `8c96392229ca` | 2025-05-15 | Merge pull request #226 from ddeclerck/gc3_ci_update |
| `1b2c19e0cae8` | 2025-05-20 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `d3a3a3e6102f` | 2025-05-22 | Merge pull request #232 from ddeclerck/add_32bit_ci |
| `a4be0beded8f` | 2025-07-17 | Merge pull request #235 from OCamlPro/gnucobol-3.x |
| `a006789fa627` | 2025-07-21 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `adf35557a63a` | 2025-07-29 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `f50ab5754982` | 2025-07-31 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `1fb152e8b536` | 2025-10-21 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x |
| `db111f65bd01` | 2025-11-06 | Merge pull request #251 from OCamlPro/gnucobol-3.x |
| `31ba95f7a4c3` | 2025-11-18 | Merge pull request #253 from OCamlPro/gnucobol-3.x |
| `5e45c5e64f37` | 2025-11-18 | Merge pull request #254 from OCamlPro/gnucobol-3.x |
| `d28f9fab8e27` | 2025-11-20 | Merge pull request #255 from nberth/update-ci-branch |
| `1fc514e9d166` | 2025-12-03 | Merge pull request #256 from ddeclerck/fix_ci |
| `da31b9286647` | 2025-12-08 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x |
| `253e5cc2602e` | 2026-01-27 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x |
| `a3accbe7616c` | 2026-05-19 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x |
| `568531bd417a` | 2026-06-09 | Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x |

## Curated semantic commits

### cobc area

| commit | date | status | subject | action |
|---|---|---|---|---|
| `8ea9ac449c98` | 2022-01-28 | NOT_APPLICABLE_WITH_PROOF | Redispatch ChangeLog entries | None: ChangeLog entry redispatch only |
| `289c9aef58a9` | 2022-02-04 | CONFIGURATION_INTEGRATED | [GCOS] Add GCOS configuration file | Adopt the GCOS configuration file |
| `777852c35adf` | 2023-10-17 | TEST_IMPORTED | testcase for [r5195] / [bugs:#923] | Adopted by the current-upstream suite lane; the behavior it tests is ported with 303917744 (module constants) |
| `12e31f960ebe` | 2023-12-14 | NOT_APPLICABLE_WITH_PROOF | minor doc adjustments and build_windows/config.h adjustment for 3.3-dev | None: 3.3-dev build_windows/config.h adjustments |
| `470f7db125a4` | 2024-01-16 | FRONTEND_REIMPLEMENTED | adjusted error handling | Adopt the adjusted error-handling behavior: error/warning selection, exit status, listings expectations (run_fundamental, run_misc, listings, syn_*); rm-strict.conf alignment |
| `85dccf1c72fb` | 2024-01-16 | NOT_APPLICABLE_WITH_PROOF | missed commit of tree.h in [r5185] | None: restores a missed tree.h hunk (codeoptim leading-zero skip) — native codegen optimization |
| `04614ac7afd2` | 2024-01-17 | RUNTIME_PORTED | Optimizations and syntax checks for INSPECT related functions | INSPECT optimizations and syntax checks: frontend syntax validation + runtime INSPECT behavior alignment |
| `f67da51cae38` | 2024-01-22 | RUNTIME_PORTED | Fix bug #917: segfault when accessing a decimal constant after calling a sub-program cobc: * codegen.c (codegen_internal, codegen_finalize): move declaration   of decimal constants from global storage to local storage to   fix bug #917 (segfault on decimal constant after CANCEL on   subprogram) | Decimal constants must live per-module (local storage) and be re-initialized after CANCEL — candidate module state model |
| `303917744a6c` | 2024-01-22 | FRONTEND_REIMPLEMENTED | Fix [bugs:#923] generated modules init/clear unused decimal constants | Generated modules init/clear unused decimal constants: candidate prepared-program must not emit unused constant state that alters module init/clear |
| `2f9892458c54` | 2024-01-22 | NOT_APPLICABLE_WITH_PROOF | Fix bug #920: Codegen: output of integer literals in generated C broken with MinGW * configure.ac: add checks to allow using stdint.h and inttypes.h * libcob/common.h: use stdint.h and inttypes.h when available to define cob_s64_t, cob_u64_t and the various CB_FMT_ macros | None: MinGW integer-literal codegen + stdint usage |
| `140a030d52ee` | 2024-01-22 | WRAPPER_INTEGRATED | New flag -fdiagnostics-absolute-path to display full paths within error locations * error.c (print_error_prefix), flag.def: new flag -fdiagnostics-absolute-paths to print the full path of a file for diagnostics; this flag can be activated if your editor and build system do not correctly work together to locate files from diagnostic output | Implement -fdiagnostics-absolute-path flag (full paths within diagnostics) |
| `44848f58b437` | 2024-01-22 | WRAPPER_INTEGRATED | Minor fixes * cobc/cobc.c (cobc_clean_up): when save-temps specifies a directory, do not move object files and preprocess files when they were specified as an explicit target on the command line (-E, -c) * libcob/common.c (cob_get_strerror), libcob/coblocal.h: export as utility function * libcob/common.c (cob_expand_env_string): fix potention buffer overflow | save-temps directory behavior: do not move object/preprocessed files when an explicit target (-E, -c) was given; adopt env-string expansion overflow fix |
| `47ffbd8363bf` | 2024-01-28 | NOT_APPLICABLE_WITH_PROOF | improved performance for comparisons between numeric DISPLAY, numeric DISPLAY to literal, as well as BCD + ZERO and to other (and BCD zero) co-authored by @chaat | None: performance optimization in the C numeric comparison layer; candidate numeric layer is independent Rust |
| `e36a124b2b72` | 2024-01-31 | WRAPPER_INTEGRATED | Add options --copy COPYBOOK and --include HEADER to cobc | Implement --copy COPYBOOK and --include HEADER options (adopt source-location mapping) |
| `106e7ce6c98c` | 2024-02-20 | NOT_APPLICABLE_WITH_PROOF | FR #459: support COLLATING SEQUENCE clause on SELECT / INDEXED files (currently only for the BDB backend) cobc: * codegen.c (output_file_initialization): output the indexed file/keys collating sequence (were already present in the AST) * tree.c (validate_indexed_key_field): process postponed key collating sequences * parser.y (collating_sequence_clause, collating_sequence_clause_key): replace CB_PENDING by CB_UNFINISHED on file and key collating sequence * flag.def, tree.c, tree.h, cobc.c, parser.y: add and handle a new -fdefault-file-colseq flag to specify the default collating sequence to use for files without a collating sequence clause libcob: * fileio.c (bdb_setkeycol, bdb_bt_compare, indexed_open, ...): take the file collating sequence into account when comparing keys * common.c, coblocal.h: rename common_cmps to cob_cmps and make it available locally | None: COLLATING SEQUENCE clause on SELECT/INDEXED files (BDB only) plus -fdefault-file-colseq flag affecting only indexed files |
| `61479ba0c781` | 2024-03-13 | FRONTEND_REIMPLEMENTED | Fix [bugs:#947]: VALUE ALL "-" not working in SCREEN SECTION * cobc/parser.y (screen_value_clause): replaced basic literals by literals | Fix VALUE ALL "-" in SCREEN SECTION (literal handling) |
| `14f0d0908d98` | 2024-03-13 | FRONTEND_REIMPLEMENTED | Fix SEGFAULT in checking prototype arguments * cobc/parser.y: fix SEGFAULT when checking the BY VALUE arguments of a prototype with ANY LENGTH | Fix SEGFAULT when checking BY VALUE arguments of a prototype with ANY LENGTH (checker robustness) |
| `7b6995042c4d` | 2024-04-09 | RUNTIME_PORTED | Add a profiling feature cobc: * parser.y: generate calls to "cob_prof_function_call" in the parsetree when profiling is unabled, when entering/leaving profiled blocks * flag.def: add `-fprof` to enable profiling * tree.h: add a flags field to cb_goto, add profiling fields to cb_program, add cb_prof_call enum and export cb_build_prof_call and cb_prof_procedure_fivision functions * tree.c (cb_build_program): initialize the new profiling fields of the cb_program structure * tree.c (cb_build_goto): add a "flags" argument (stored in the cb_program structure) * typeck.c (cb_emit_goto): add a "flags" argument (passed to cb_build_goto) * codegen.c: handle profiling code generation under the cb_flag_prof guard libcob: * Makefile.am: add `profiling.c` to sources * profiling.c: implement profiling functions (time spent in each procedure of the program) * common.c: add 4 environments variables COB_PROF_FILE, COB_PROF_MAX_DEPTH,COB_PROF_ENABLE and COB_PROF_FORMAT * common.c (cob_expand_env_string): add $b (executable basename), $f (executable filename), $d (date in yyyymmdd) and $t (time in hhmmss) * common.c (cob_set_main_argv0): extracted from cob_init * fileio.c (cob_path_to_absolute): extracted from insert and cob_set_main_argv0 config: * runtime.cfg: add COB_PROF_FILE | Implement the profiling feature for the interpreted candidate: -fprof flag; per-procedure time accounting in the interpreter; COB_PROF_FILE/COB_PROF_MAX_DEPTH/COB_PROF_ENABLE/COB_PROF_FORMAT env support; $b/$f/$d/$t expansion in env strings |
| `82100d64de35` | 2024-04-27 | NOT_APPLICABLE_WITH_PROOF | Optimization of memory usage in replace.c | None: memory optimization in the C replace.c; candidate REPLACE layer is independent Rust |
| `8366e1be1cf8` | 2024-05-02 | NOT_APPLICABLE_WITH_PROOF | housekeeping | None: housekeeping (build_aux file removal) |
| `442e6db6d430` | 2024-05-03 | NOT_APPLICABLE_WITH_PROOF | build and test fixes for Win32 | None: Win32 build and test fixes |
| `a0937bf4920e` | 2024-05-04 | FRONTEND_REIMPLEMENTED | fixing [bugs:#933] [bugs:#938] [bugs:#966] handling of broken expressions | Improve handling of broken expressions (recovery, no hangs, correct reject) |
| `7c60012c019b` | 2024-05-04 | TEST_IMPORTED | fixing typo | Adopted by the current-upstream suite lane (typo fix in run_misc.at) |
| `ed789c8a9bc2` | 2024-05-05 | PLATFORM_BEHAVIOR_INTEGRATED | Win32 fixes, mostly testcases | Adopt Win32-relevant test expectations via the current-upstream lane; the common.c/fileio.c parts are Windows-path fixes |
| `d2df58ad9685` | 2024-05-14 | NOT_APPLICABLE_WITH_PROOF | assorted minor cleanups | None: assorted minor cleanups (C-wide, no single observable behavior; verify with tests) |
| `67f93f93c5b5` | 2024-05-14 | NOT_APPLICABLE_WITH_PROOF | fix building with MSVC cobc: * flag.def: fix macro usage for MSVC build_windows: * general for libcob: add missing profiling.c | None: MSVC build fixes (flag.def macro usage, build_windows) |
| `63cb06ce7b87` | 2024-05-15 | NOT_APPLICABLE_WITH_PROOF | more compiler warnings fixed | None: compiler warnings fixed (C internal) |
| `d671493076c3` | 2024-05-15 | NOT_APPLICABLE_WITH_PROOF | improving cobc handling with MSVC assembler | None: cobc handling with the MSVC assembler (native assembly path) |
| `a4971637900e` | 2024-05-15 | NOT_APPLICABLE_WITH_PROOF | more warning fixes, additional improvement to build_windopws | None: warning fixes + build_windows improvements |
| `940f057e6522` | 2024-06-20 | NOT_APPLICABLE_WITH_PROOF | Windows build fixes + ChangeLog cleanups cobc/cobc.c (process_compile): fix MSVC build command tests/atlocal_win: fix path-related issues in Windows builds | None: Windows/MSVC build command fixes + atlocal_win path fixes |
| `7c7b55b9311b` | 2024-07-05 | RUNTIME_PORTED | Adjustment for move to edited numeric | Adjustment for move to edited numeric (with tests) |
| `435454f8df38` | 2024-07-10 | RUNTIME_PORTED | Adjustment for move to edited numeric | Adjustment for move to edited numeric (frontend picture + runtime edit alignment) |
| `9f1a64c32e11` | 2024-07-23 | RUNTIME_PORTED | [feature-requests:#448] using state structures instead of state vars for strings | Use state structures instead of state vars for STRING/UNSTRING/INSPECT: port the reworked string-operation state handling |
| `0fa2bf5f5238` | 2024-07-26 | FRONTEND_REIMPLEMENTED | increase portability for Micro Focus and ACUCOBOL-GT | Increase dialect portability for Micro Focus and ACUCOBOL-GT (reserved words/config.def/parser) |
| `ec5562cfb9f6` | 2024-07-30 | RUNTIME_PORTED | Adjustment to support the 2023 standard for edited numeric picture strings and to fix [bugs:#935] | Support the 2023 standard for edited numeric picture strings and fix bugs:#935 (picture-string validation + runtime edited move) |
| `71ea358aa910` | 2024-08-10 | FRONTEND_REIMPLEMENTED | work on ALPHABET definitions, especially ALPHABET FOR NATIONAL | Implement ALPHABET definitions, especially ALPHABET FOR NATIONAL (parse + collating behavior) |
| `41e2e4488de1` | 2024-08-11 | FRONTEND_REIMPLEMENTED | work on ALPHABET definitions, especially ALPHABET FOR NATIONAL - C90 fix for r5310 | Complete ALPHABET FOR NATIONAL support (C90 follow-up) |
| `5a8666888fad` | 2024-08-28 | FRONTEND_REIMPLEMENTED | Fix bugs reported by the MSVC runtime checker cobc: * tree.c (char_to_precedence_idx, get_char_type_description, valid_char_order): adjusted size of precedence table and gave proper precedence to U libcob: * intrinsics.c (cob_intr_random), move.c (cob_move_display_to_packed): make casts with loss of data explicit using masking to silence the MSVC runtime error checker | Give 'U' proper precedence in the expression precedence table (parser); port masking fixes in random/packed move where the candidate has equivalent numeric paths |
| `111d21f03445` | 2024-09-20 | TEST_IMPORTED | Minor adjustments (testsuite, ChangeLog entries, C89) | Adopted by the current-upstream suite lane (syn_definition.at updates); the pplex/scanner changes are C89-internal |
| `1104bda61e19` | 2024-09-25 | FRONTEND_REIMPLEMENTED | Check for incompatible data only when a receiver is of category numeric in MOVE or SET | Check for incompatible data in MOVE or SET only when the receiver is of category numeric |
| `10daa94c8936` | 2024-09-27 | NOT_APPLICABLE_WITH_PROOF | build system update | None: libtool/autotools build system update |
| `903ba84ff9db` | 2024-09-29 | NOT_APPLICABLE_WITH_PROOF | assorted updates | Verify in Phase 2: mixed C cleanup/updates (cobc, libcob, build, tests) without a single identified candidate-visible behavior; no known semantic delta |
| `49da19a3dfc0` | 2024-09-30 | WRAPPER_INTEGRATED | Add dependencies options and -fcopybook-deps cobc: * pplex.l (cb_text_list): prevent duplicates * cobc.c, help.c, pplex.l: add new flags to output dependencies following gcc: -M to output deps only, -MD to output deps while compiling (in .d files), -MP to output phony targets, -MG to keep missing copybooks, -MQ <target> to Makefile-quote target ; add -fcopybook-deps to output only copybook names instead of file paths. -fcopybook-deps also forces -E, -foneline-deps, -MT=copybooks, disables errors on missing copybooks and removes output on stdout doc: * gnucobol.texi: document new dependencies options | Implement -M/-MD/-MP/-MG/-MQ dependency options + -fcopybook-deps (copybook-only deps, forces -E -foneline-deps -MT=copybooks, disables missing-copybook errors) |
| `9b0259d78f87` | 2024-10-01 | NOT_APPLICABLE_WITH_PROOF | Support collating sequence for indexed file keys of alphanumeric class | None: collating sequence for indexed file keys of alphanumeric class (indexed backend) |
| `3f7c44b6f516` | 2024-10-02 | WRAPPER_INTEGRATED | improve stdin compilation | Improve stdin compilation: cobc-rs must compile from stdin with the documented naming/artifact behavior |
| `88937849b860` | 2024-10-02 | CONFIGURATION_INTEGRATED | new options for configure for customized version string / bug report URL | Adopt configurable version string / bug-report URL surfaces where the candidate exposes equivalents |
| `c53ae5f80351` | 2024-10-02 | RUNTIME_PORTED | signal handler updates | Signal handler updates: port the handler registration/behavior semantics |
| `2a53351eae5a` | 2024-11-18 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | Add PANEL functions from CURSES | Track only: PANEL functions from CURSES (native curses/panel dependency) |
| `b162a03c3d94` | 2024-12-08 | NOT_APPLICABLE_WITH_PROOF | minor update for sanitizers | None: sanitizer-related minor C adjustments |
| `87500ead47bd` | 2025-01-10 | FRONTEND_REIMPLEMENTED | fixed [bugs:#961]: Nested Elements Mishandled Despite 'with attributes' Specification | Fix nested-element handling with the 'with attributes' specification (SCREEN SECTION data-name qualification) |
| `23b5446c13ed` | 2025-01-13 | NOT_APPLICABLE_WITH_PROOF | Fix bad typeck.c indentation introduced by [r5112] | None: indentation-only fix in C typeck.c |
| `8a7c349d13ad` | 2025-02-12 | FRONTEND_REIMPLEMENTED | FR #176: "Implementation of GC directive to include .h (c/c++) files" cobc: * pplex.l, ppparse.y, cobc.h, codegen.c (output_gnucobol_defines): new >>IMP INCLUDE directive to include one or multiple header files in the generated C code (same behavior as the --include but with one directive per file) * scanner.l: the leading space for all internal directives is removed in the lexer. Source previously preprocessed may need to be adjusted | Implement the >>IMP INCLUDE directive (include .h/.c++ headers) at the preprocessing level; adopt the scanner change (leading space removed for internal directives) |
| `bba2a4ee7a73` | 2025-02-16 | WRAPPER_INTEGRATED | Display the help text of -fwinmain on both Win32 and Cygwin | Show -fwinmain help text on both Win32 and Cygwin in cobc-rs help output |
| `140aed5814bc` | 2025-03-03 | NOT_APPLICABLE_WITH_PROOF | Remove erroneous ifdef/define in replace.c | None: removes an erroneous ifdef/define in the C replace.c |
| `3f99dba47432` | 2025-03-26 | NOT_APPLICABLE_WITH_PROOF | minor, mostly build updates | None: minor mostly-autotools build updates |
| `54d4963026a1` | 2025-03-31 | NOT_APPLICABLE_WITH_PROOF | Add an EBCDIC/ASCII table generation feature build_windows: * general for cobc: include new gentable.c cobc: * gentable.c: generate EBCDIC/ASCII translation tables * cobc.c, help.c: new --gentable option doc: * gnucobol.texi: document the new --gentable option | None: --gentable generates native C translation tables (EBCDIC/ASCII) |
| `dc0cddebe0f0` | 2025-04-15 | WRAPPER_INTEGRATED | Fixes to the dependency generation feature introduced by [r5345] cobc: * cobc.c (process_filename): ensure we don't keep the preprocessed file when using -M or -fcopybook-deps * cobc.c, cobc.h, help.c, pplex.l: make -fcopybook-deps an experimental feature, activable with the EXPERIMENTAL_COPYBOOK_DEPS_OPTION flag | Fix -M/-fcopybook-deps behavior: do not keep the preprocessed file; gate -fcopybook-deps behind the experimental option; adopt tests |
| `79c65d0ecf1a` | 2025-04-16 | FRONTEND_REIMPLEMENTED | Fix [bugs:#948]: comparison with HIGH-VALUE in presence of collating sequences cobc: * tree.h (cb_program): add low_value and high_value fields to hold the low and high values used by the program collating sequence * tree.c (cb_build_program): initialize the low_value and high_value fields to reasonable default values * typeck.c: replace hard-coded cob_refer_ascii and cob_refer_ebcdic by ebcdic_to_ascii and ascii_to_ebcdic * typeck.c (cb_validate_collating): set the program's low_value and high_value fields * typeck.c (validate_alphabet): use the new tables, set the alphabet's low and high values * cobc.h: export the new symbols defined in typeck.c * cobc.c (process_command_line): always load the collating table * scanner.l (scan_ebcdic_char): remove code that loads and use a local collating table, use the table defined in typeck.c instead * codegen.c: replace hard-coded 0 and 255 / 0xff contants with the low_value and high_value fields where appropriate * codegen.c (output_low_value, output_high_value): move the cob_all_low and cob_all_high fields from global to local * codegen.c (output_collating_tables): remove local tables and code that loads the tables, since they are now loaded from cobc.c libcob: * strings.c: use the collating_sequence field of cob_module to determine the low value instead of the hard-coded constant "\0" | Program-level low/high collating values: compute per-program collating low/high in the frontend; runtime comparison (HIGH-VALUE / LOW-VALUE in presence of collating sequences) uses them |
| `486565722c48` | 2025-05-20 | RUNTIME_PORTED | Limit sed usage in testsuite, remove listing-sed cobc: * cobc.c (set_compile_date): fix SOURCE_DATE_EPOCH being ignored on subsequent invocations libcob: * common.c (cob_set_date_from_epoch): fix incorrect conversion of epoch (was off by one day) tests: * testsuite.src/run_misc.at, testsuite.src/syn_misc.at: reduce the use of sed by using SOURCE_DATE_EPOCH when possible and using @&t@ quadrigraphs in expected output with trailing spaces * listing-sed.sh: removed as no longer needed * atlocal.in, atlocal_win: remove the no longer needed UNIFY_LISTING variable | Fix SOURCE_DATE_EPOCH being ignored on subsequent invocations; fix epoch conversion; remove listing-sed dependency from harness |
| `eb8536cfcd33` | 2025-05-26 | RUNTIME_PORTED | follow up to [r5531]: further adjustemnts to date computation from epoch | Apply the further date-from-epoch adjustments (final state of the family) |
| `a5253353db12` | 2025-06-02 | NOT_APPLICABLE_WITH_PROOF | portability fixes (and more) | None: C portability fixes across the native build |
| `1fc700cc0cd9` | 2025-07-17 | NOT_APPLICABLE_WITH_PROOF | c89 compat adjustments | None: C89 compatibility adjustments for the C compiler |
| `3dd1d88da6ff` | 2025-07-17 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | initial (unfinished) support for XML PARSE | Track only: initial XML PARSE support is unfinished upstream and depends on the native XML backend |
| `7fef5fde70af` | 2025-07-28 | NOT_APPLICABLE_WITH_PROOF | c89/c23 compat and hardening adjustments, along with updated gettext infrastructure | None: C89/C23 source-compat and hardening plus gettext autotools infrastructure are native C build concerns |
| `8954b5fc10e6` | 2025-07-30 | RUNTIME_PORTED | Code and testsuite cleanup | Port the observable effects of the code cleanup across move/screenio/termio/mlio/fileio; adopt the updated tests (data_display, run_accept, run_extensions, run_file, run_manual_screen, run_misc, run_returncode, syn_*) |
| `f4ffd50ecd24` | 2025-07-30 | FRONTEND_REIMPLEMENTED | reserved word handling and trace update | Reserved-word handling update + trace update: adopt the changed reserved-word set and trace output |
| `7b324f50ebbb` | 2025-10-05 | FRONTEND_REIMPLEMENTED | parser cleanup and better handling of incomplete code | Parser cleanup + better handling of incomplete code: bounded recovery, no hangs, deterministic diagnostics |
| `277a07c2ee9c` | 2025-10-17 | FRONTEND_REIMPLEMENTED | improve SD syntax checks and error recovery | Port the SD syntax-check behavior plus its tests; ensure no hang on malformed SD |
| `23f8503529f0` | 2025-10-17 | FRONTEND_REIMPLEMENTED | improve SD syntax checks and error recovery | Improve SD (sort description) syntax checks and error recovery |
| `bf0b5878a898` | 2025-11-12 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | XML and JSON updates | Track only: XML and JSON updates (native backend) |
| `bc5c13b27467` | 2025-11-14 | NOT_APPLICABLE_WITH_PROOF | portability updates | None: C portability updates |
| `13963e15a2da` | 2025-11-18 | WRAPPER_INTEGRATED | ensure full output for -ftcmd using multiple continuation lines as necessary | -ftcmd listing output must continue across multiple lines instead of truncating (candidate listing generation) |
| `2c092ca140b4` | 2025-11-27 | FRONTEND_REIMPLEMENTED | check for terminating periods at the end of SET directives | Check for terminating periods at the end of SET directives; accept/reject and diagnose per upstream |
| `39ab4808c7e5` | 2025-12-02 | WRAPPER_INTEGRATED | listing header change: basename only | Listing header must show the basename only (candidate listing generation) |
| `4b72d0a9faac` | 2025-12-04 | RUNTIME_PORTED | improve memory handling in edge-cases | Improve memory handling in edge cases: port any observable bounds/state fixes; adopt tests |
| `c4eea8102820` | 2025-12-05 | FRONTEND_REIMPLEMENTED | fix areacheck - ENTRY statement should begin in area B not area A | Fix area-check: ENTRY statement must begin in area B, not area A (candidate checker area validation) |
| `9e0d66418efc` | 2025-12-29 | FRONTEND_REIMPLEMENTED | cobc/tree.c (finalize_file): if file is EXTFH enabled then don't warn for ORGANIZATION INDEXED, even when compiler is configured --without-db | Suppress the ORGANIZATION INDEXED warning when the file is EXTFH-enabled (candidate checker must not warn where upstream does not) |
| `47dda86c0013` | 2026-05-26 | FRONTEND_REIMPLEMENTED | Config option tab-width can receive a list of comma-separated widths | Implement -ftab-width=w1,w2,... list semantics: each 1..12, last repeats indefinitely, malformed/empty/overflow lists fail with stable config errors; apply to fixed/free/auto formats, preprocessing, listing, diagnostics; repeated options follow upstream precedence |
| `a672a26b52b5` | 2026-06-08 | FRONTEND_REIMPLEMENTED | Fix handling of some special contexts, and provide room for more | Implement typed nested parser-context mechanism (ContextSet + stack with enter/leave guards, recovery cleanup, leak assertions); extend beyond 32 flags; match upstream accept/reject for CALL convention, CALL USING, REPOSITORY, EXIT, USAGE, TYPEDEF, SPECIAL-NAMES, VALIDATE STATUS, READY/RESET contexts |

### libcob area

| commit | date | status | subject | action |
|---|---|---|---|---|
| `8ea9ac449c98` | 2022-01-28 | NOT_APPLICABLE_WITH_PROOF | Redispatch ChangeLog entries | None: ChangeLog entry redispatch only |
| `0166302909e9` | 2023-08-17 | RUNTIME_PORTED | fix [bugs:#904] MOVE PACKED-DECIMAL unsigned to signed leads to bad sign | Fix MOVE PACKED-DECIMAL unsigned to signed bad sign |
| `8208acac177e` | 2023-08-22 | NOT_APPLICABLE_WITH_PROOF | missing commit for [r5167] - version increase | None: native version.h increase |
| `12e31f960ebe` | 2023-12-14 | NOT_APPLICABLE_WITH_PROOF | minor doc adjustments and build_windows/config.h adjustment for 3.3-dev | None: 3.3-dev build_windows/config.h adjustments |
| `c3d5860bf219` | 2024-01-16 | RUNTIME_PORTED | minor cleanup and optimizations in libcob | Adopt cob_add_int scale handling and packed_is_negative semantics (numeric behavior) |
| `28b02be15485` | 2024-01-16 | RUNTIME_PORTED | ... fixed last commit that disabled code that is still used outside of experimental local checkouts... | Restore the cob_decimal_get_display sign-in-diff fix (numeric display sign behavior) |
| `04614ac7afd2` | 2024-01-17 | RUNTIME_PORTED | Optimizations and syntax checks for INSPECT related functions | INSPECT optimizations and syntax checks: frontend syntax validation + runtime INSPECT behavior alignment |
| `62b39805ca22` | 2024-01-18 | RUNTIME_PORTED | Fixing [bugs:#914] CLOSE LOCK abends program on OPEN | Fix CLOSE LOCK abend on OPEN (file state handling) |
| `0b22d441757e` | 2024-01-19 | RUNTIME_PORTED | Fixing [bugs:#913] DISPLAY and ACCEPT with simple attributes SIGSEGV | Fix DISPLAY and ACCEPT with simple attributes SIGSEGV (candidate screen statements with attribute handling) |
| `8e2ec25c26bc` | 2024-01-20 | RUNTIME_PORTED | fix [bugs:#918] partial broken COB_LS_VALIDATE | Fix partial broken COB_LS_VALIDATE (line-sequential validation) |
| `2f9892458c54` | 2024-01-22 | NOT_APPLICABLE_WITH_PROOF | Fix bug #920: Codegen: output of integer literals in generated C broken with MinGW * configure.ac: add checks to allow using stdint.h and inttypes.h * libcob/common.h: use stdint.h and inttypes.h when available to define cob_s64_t, cob_u64_t and the various CB_FMT_ macros | None: MinGW integer-literal codegen + stdint usage |
| `5d0eecfbdd6d` | 2024-01-22 | NOT_APPLICABLE_WITH_PROOF | Fix random segfaults in cob_call_with_exception_check on Windows * common.c (cob_terminate_routines, cob_call_with_exception_check): add a mechanism to postpone unloading of modules when using longjmp, as this is not safe on Windows (its implementation of longjmp performs stack-unwinding) | None: Windows longjmp module-unload postponement |
| `44848f58b437` | 2024-01-22 | WRAPPER_INTEGRATED | Minor fixes * cobc/cobc.c (cobc_clean_up): when save-temps specifies a directory, do not move object files and preprocess files when they were specified as an explicit target on the command line (-E, -c) * libcob/common.c (cob_get_strerror), libcob/coblocal.h: export as utility function * libcob/common.c (cob_expand_env_string): fix potention buffer overflow | save-temps directory behavior: do not move object/preprocessed files when an explicit target (-E, -c) was given; adopt env-string expansion overflow fix |
| `f9596f55fe49` | 2024-01-25 | NOT_APPLICABLE_WITH_PROOF | fixing compilation of [r5215] - [feature-requests:#459] COLLATING SEQUENCE for [!WITH_DB] | None: compilation fix for !WITH_DB builds |
| `47ffbd8363bf` | 2024-01-28 | NOT_APPLICABLE_WITH_PROOF | improved performance for comparisons between numeric DISPLAY, numeric DISPLAY to literal, as well as BCD + ZERO and to other (and BCD zero) co-authored by @chaat | None: performance optimization in the C numeric comparison layer; candidate numeric layer is independent Rust |
| `300b542f3caa` | 2024-02-16 | NOT_APPLICABLE_WITH_PROOF | fileio refactoring | Verify in Phase 2: fileio refactoring (native C internals); no candidate-visible behavior expected |
| `106e7ce6c98c` | 2024-02-20 | NOT_APPLICABLE_WITH_PROOF | FR #459: support COLLATING SEQUENCE clause on SELECT / INDEXED files (currently only for the BDB backend) cobc: * codegen.c (output_file_initialization): output the indexed file/keys collating sequence (were already present in the AST) * tree.c (validate_indexed_key_field): process postponed key collating sequences * parser.y (collating_sequence_clause, collating_sequence_clause_key): replace CB_PENDING by CB_UNFINISHED on file and key collating sequence * flag.def, tree.c, tree.h, cobc.c, parser.y: add and handle a new -fdefault-file-colseq flag to specify the default collating sequence to use for files without a collating sequence clause libcob: * fileio.c (bdb_setkeycol, bdb_bt_compare, indexed_open, ...): take the file collating sequence into account when comparing keys * common.c, coblocal.h: rename common_cmps to cob_cmps and make it available locally | None: COLLATING SEQUENCE clause on SELECT/INDEXED files (BDB only) plus -fdefault-file-colseq flag affecting only indexed files |
| `7b6995042c4d` | 2024-04-09 | RUNTIME_PORTED | Add a profiling feature cobc: * parser.y: generate calls to "cob_prof_function_call" in the parsetree when profiling is unabled, when entering/leaving profiled blocks * flag.def: add `-fprof` to enable profiling * tree.h: add a flags field to cb_goto, add profiling fields to cb_program, add cb_prof_call enum and export cb_build_prof_call and cb_prof_procedure_fivision functions * tree.c (cb_build_program): initialize the new profiling fields of the cb_program structure * tree.c (cb_build_goto): add a "flags" argument (stored in the cb_program structure) * typeck.c (cb_emit_goto): add a "flags" argument (passed to cb_build_goto) * codegen.c: handle profiling code generation under the cb_flag_prof guard libcob: * Makefile.am: add `profiling.c` to sources * profiling.c: implement profiling functions (time spent in each procedure of the program) * common.c: add 4 environments variables COB_PROF_FILE, COB_PROF_MAX_DEPTH,COB_PROF_ENABLE and COB_PROF_FORMAT * common.c (cob_expand_env_string): add $b (executable basename), $f (executable filename), $d (date in yyyymmdd) and $t (time in hhmmss) * common.c (cob_set_main_argv0): extracted from cob_init * fileio.c (cob_path_to_absolute): extracted from insert and cob_set_main_argv0 config: * runtime.cfg: add COB_PROF_FILE | Implement the profiling feature for the interpreted candidate: -fprof flag; per-procedure time accounting in the interpreter; COB_PROF_FILE/COB_PROF_MAX_DEPTH/COB_PROF_ENABLE/COB_PROF_FORMAT env support; $b/$f/$d/$t expansion in env strings |
| `8366e1be1cf8` | 2024-05-02 | NOT_APPLICABLE_WITH_PROOF | housekeeping | None: housekeeping (build_aux file removal) |
| `6fd7c72cd16e` | 2024-05-02 | NOT_APPLICABLE_WITH_PROOF | build fix [patches:#64] | None: build fix (patches:#64) |
| `6a23e2ce5a8c` | 2024-05-02 | NOT_APPLICABLE_WITH_PROOF | housekeeping | None: MinGW strcasecmp redefinition removal + whitespace |
| `67f8f532e194` | 2024-05-02 | NOT_APPLICABLE_WITH_PROOF | follow up to [r5215] [feature-requests:#459]: support COLLATING SEQUENCE clause on SELECT / INDEXED files | None: COLLATING SEQUENCE clause on SELECT/INDEXED files follow-up |
| `1b8af634e882` | 2024-05-03 | NOT_APPLICABLE_WITH_PROOF | fixing [r5244] fix... | None: BDB ABI comma fix (DB_VERSION_MAJOR >= 12) |
| `ed789c8a9bc2` | 2024-05-05 | PLATFORM_BEHAVIOR_INTEGRATED | Win32 fixes, mostly testcases | Adopt Win32-relevant test expectations via the current-upstream lane; the common.c/fileio.c parts are Windows-path fixes |
| `d2df58ad9685` | 2024-05-14 | NOT_APPLICABLE_WITH_PROOF | assorted minor cleanups | None: assorted minor cleanups (C-wide, no single observable behavior; verify with tests) |
| `9261f4096868` | 2024-05-14 | NOT_APPLICABLE_WITH_PROOF | minor cleanups and warning fixes | None: minor cleanups and warning fixes in libcob (no observable behavior delta; verify with tests) |
| `63cb06ce7b87` | 2024-05-15 | NOT_APPLICABLE_WITH_PROOF | more compiler warnings fixed | None: compiler warnings fixed (C internal) |
| `7c7b55b9311b` | 2024-07-05 | RUNTIME_PORTED | Adjustment for move to edited numeric | Adjustment for move to edited numeric (with tests) |
| `b33a87961f0d` | 2024-07-05 | NOT_APPLICABLE_WITH_PROOF | Adjustment to READ PREVIOUS for VBISAM / VISAM | None: READ PREVIOUS for VBISAM/VISAM (relative-indexed native backend) |
| `026a651ee406` | 2024-07-06 | NOT_APPLICABLE_WITH_PROOF | Fix errors caught by the Sanitizer functionality of GCC. | None: errors caught by the GCC Sanitizer (C internal) |
| `d33f2ec97d72` | 2024-07-06 | RUNTIME_PORTED | Added two new functions CBL_GC_SCR_DUMP and CBL_GC_SCR_RESTORE | Implement CBL_GC_SCR_DUMP and CBL_GC_SCR_RESTORE as candidate runtime callable functions |
| `e51b091b9921` | 2024-07-06 | RUNTIME_PORTED | Fix for bug 934 - default ROUNDED option | Fix default ROUNDED option behavior |
| `435454f8df38` | 2024-07-10 | RUNTIME_PORTED | Adjustment for move to edited numeric | Adjustment for move to edited numeric (frontend picture + runtime edit alignment) |
| `314adc1ca830` | 2024-07-11 | NOT_APPLICABLE_WITH_PROOF | Adjustment for build without curses and fix C90 warnings | None: curses-less build + C90 warnings |
| `73ad00d94545` | 2024-07-11 | NOT_APPLICABLE_WITH_PROOF | djustment for build without curses and fix C90 warnings | None: curses-less build + C90 warnings |
| `24ff1a9c9335` | 2024-07-12 | NOT_APPLICABLE_WITH_PROOF | Allow keys of different length in the BDB backend (optional, flag-controlled) libcob: fileio.c (bdb_bt_compare, indexed_open): handle BDB keys of different length with a flag USE_BDB_KEYDIFF (passed with preparser flag CPPFLAGS) common.c (cob_cmp_strings), coblocal.h (cob_cmp_strings): extracted from (cob_cmp_alnum) | None: BDB keys of different length (USE_BDB_KEYDIFF flag) |
| `7a173e6da655` | 2024-07-13 | NOT_APPLICABLE_WITH_PROOF | adjustment for Sanitizer warning | None: Sanitizer-warning adjustment (C internal) |
| `9f1a64c32e11` | 2024-07-23 | RUNTIME_PORTED | [feature-requests:#448] using state structures instead of state vars for strings | Use state structures instead of state vars for STRING/UNSTRING/INSPECT: port the reworked string-operation state handling |
| `ec5562cfb9f6` | 2024-07-30 | RUNTIME_PORTED | Adjustment to support the 2023 standard for edited numeric picture strings and to fix [bugs:#935] | Support the 2023 standard for edited numeric picture strings and fix bugs:#935 (picture-string validation + runtime edited move) |
| `6bf47af0209e` | 2024-08-19 | RUNTIME_PORTED | [feature-request:#474]: add runtime configuration to hide cursor for extended screenio | Implement runtime configuration to hide the cursor for extended screenio |
| `816bd2be16d8` | 2024-08-22 | NOT_APPLICABLE_WITH_PROOF | Disable Windows error popups in programs compiled with MSVC * libcob/common.c (DllMain) [_MSC_VER]: added calls to _CrtSetReportMode to disable Windows error popups and redirect them to stderr | None: Windows/MSVC CRT report-mode setting |
| `7529ba38d84b` | 2024-08-22 | NOT_APPLICABLE_WITH_PROOF | Remove debugapi.h include from common.c - follow-up to r5315 | None: Windows-only include removal |
| `5a8666888fad` | 2024-08-28 | FRONTEND_REIMPLEMENTED | Fix bugs reported by the MSVC runtime checker cobc: * tree.c (char_to_precedence_idx, get_char_type_description, valid_char_order): adjusted size of precedence table and gave proper precedence to U libcob: * intrinsics.c (cob_intr_random), move.c (cob_move_display_to_packed): make casts with loss of data explicit using masking to silence the MSVC runtime error checker | Give 'U' proper precedence in the expression precedence table (parser); port masking fixes in random/packed move where the candidate has equivalent numeric paths |
| `7b09c750ff7d` | 2024-09-20 | RUNTIME_PORTED | Fix [bugs:#990] COBOL screen, problem positioning cursor on line 1 | Fix cursor positioning on line 1 (COBOL screen) |
| `7ba5f9fcb116` | 2024-09-23 | RUNTIME_PORTED | preparation for Multiple Window support by WINDOW pointer | WINDOW pointer preparation: adopt the screenio WINDOW handling model where the candidate screen layer can represent it |
| `903ba84ff9db` | 2024-09-29 | NOT_APPLICABLE_WITH_PROOF | assorted updates | Verify in Phase 2: mixed C cleanup/updates (cobc, libcob, build, tests) without a single identified candidate-visible behavior; no known semantic delta |
| `a3e00bed1f21` | 2024-09-29 | NOT_APPLICABLE_WITH_PROOF | fix bad line in [r5343] | None: C preprocessor conditional fix (#elif) |
| `88937849b860` | 2024-10-02 | CONFIGURATION_INTEGRATED | new options for configure for customized version string / bug report URL | Adopt configurable version string / bug-report URL surfaces where the candidate exposes equivalents |
| `c53ae5f80351` | 2024-10-02 | RUNTIME_PORTED | signal handler updates | Signal handler updates: port the handler registration/behavior semantics |
| `3f897122aacd` | 2024-10-04 | RUNTIME_PORTED | signal and stack handling update | Signal and stack handling update: align candidate signal-registration and stack-guard behavior with upstream semantics |
| `ef11be499f4c` | 2024-10-04 | NOT_APPLICABLE_WITH_PROOF | fix previous commit | None: duplicate include removal |
| `83ec07716d30` | 2024-10-04 | NOT_APPLICABLE_WITH_PROOF | portability fix for last commits | None: const-correctness C fix |
| `ac862070c3e8` | 2024-10-24 | RUNTIME_PORTED | fixed [bugs:#999] ACCEPT with TIMEOUT issue when looping thru the verb | Fix ACCEPT with TIMEOUT looping through the verb (candidate ACCEPT TIMEOUT semantics) |
| `a6c4f2440452` | 2024-11-18 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | Add XML/JSON GENERATE tests for PIC P | Track tests: XML/JSON GENERATE tests for PIC P depend on the native XML/JSON backend |
| `2a53351eae5a` | 2024-11-18 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | Add PANEL functions from CURSES | Track only: PANEL functions from CURSES (native curses/panel dependency) |
| `0bf2ceb38ea4` | 2024-11-18 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | follow-up to r5369 - panel update | Track only: PANEL update + tests (curses) |
| `45ce8f622930` | 2024-11-19 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | follow-up to r5369 - panel update | Track only: PANEL update + tests (curses) |
| `d5eb0eb02335` | 2024-11-19 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | follow-up to r5369 - panel update | Track only: PANEL update (curses) |
| `87c1dd5799ff` | 2024-12-02 | RUNTIME_PORTED | Fixed [bugs:#1008] regression in move to numeric edited items * move.c (optimized_move_display_to_edited): fixed Bug #1008: regression in move to numeric edited items with insertion symbols B, 0 and / * move.c (optimized_move_display_to_edited): minor refactoring * exception.def, move.c: added definition for COBOL2025 COB_EC_DATA_NULL + COB_EC_DATA_TRUNCATION (currently not used) | Fix move-to-edited regression with insertion symbols B, 0 and /; register COBOL2025 COB_EC_DATA_NULL and COB_EC_DATA_TRUNCATION exception definitions |
| `7bddf706da7a` | 2024-12-08 | NOT_APPLICABLE_WITH_PROOF | fix copy+paste error in r5389 | None: whitespace-only correction |
| `dca86ab692a4` | 2024-12-08 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | follow-up to r5369 - panel update (portability fix) | Track only: PANEL portability fix (curses) |
| `1c357b4a3894` | 2024-12-12 | NOT_APPLICABLE_WITH_PROOF | Fixed [bugs:1032]: app_data field of DBT structure is not always copied when call bdb_bt_compare * libcob/fileio.c: fixed Bug #1032 by always using global thread-static variable bdb_app_data pointer to access the collating sequence function | None: BDB DBT app_data fix |
| `44c96d20a12e` | 2024-12-17 | RUNTIME_PORTED | Fix BLANK WHEN ZERO not working on signed NUMERIC-EDITED fields libcob: * move.c (optimized_move_display_to_edited): normalize numeric data * move.c (cob_move): extend use of optimized_move_display_to_edited to more cases (i.e. different source and destination sign, leading sign, non-separate sign) * move.c (optimized_move_display_to_edited): fixed additional bug reported with bug #1008: BLANK WHEN ZERO not working on signed NUMERIC-EDITED fields | BLANK WHEN ZERO on signed NUMERIC-EDITED fields: normalize numeric data in edited move; extend edited-move to sign variants |
| `cb5fe73262cf` | 2024-12-19 | RUNTIME_PORTED | Fix STRING/UNSTRING/INSPECT bug introduced in r5302 * libcob/string.c: fix a bug where the source of STRING/UNSTRING/INSPECT is overwritten, by restoring the *_copy fields that were removed with the change on 2024-02-26 | Fix STRING/UNSTRING/INSPECT source-overwrite bug (source fields must not be clobbered mid-operation) |
| `921108ea29fc` | 2024-12-20 | RUNTIME_PORTED | follow-up commit to r5403 - fix an out of bounds read access in optimized_move_display_to_edited | Fix out-of-bounds read in optimized move DISPLAY->edited (candidate edited move bounds) |
| `8cec9fdb89c0` | 2024-12-24 | RUNTIME_PORTED | Improve display of floats * libcob/termio.c (clean_double): skip more than a single leading zero in exponent display | Float display: skip more than a single leading zero in exponent digits (candidate float formatting) |
| `ff8f8953be84` | 2024-12-31 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | follow-up to r5369 - panel update | Track only: PANEL functions depend on the native curses backend |
| `47ec5f513416` | 2025-01-06 | NOT_APPLICABLE_WITH_PROOF | Improve handling of partial keys * libcob/fileio.c (indexed_start_internal): improve handling of partial keys, to ensure BDB always compares keys of identical length | None: BDB indexed partial-key comparison |
| `79c65d0ecf1a` | 2025-04-16 | FRONTEND_REIMPLEMENTED | Fix [bugs:#948]: comparison with HIGH-VALUE in presence of collating sequences cobc: * tree.h (cb_program): add low_value and high_value fields to hold the low and high values used by the program collating sequence * tree.c (cb_build_program): initialize the low_value and high_value fields to reasonable default values * typeck.c: replace hard-coded cob_refer_ascii and cob_refer_ebcdic by ebcdic_to_ascii and ascii_to_ebcdic * typeck.c (cb_validate_collating): set the program's low_value and high_value fields * typeck.c (validate_alphabet): use the new tables, set the alphabet's low and high values * cobc.h: export the new symbols defined in typeck.c * cobc.c (process_command_line): always load the collating table * scanner.l (scan_ebcdic_char): remove code that loads and use a local collating table, use the table defined in typeck.c instead * codegen.c: replace hard-coded 0 and 255 / 0xff contants with the low_value and high_value fields where appropriate * codegen.c (output_low_value, output_high_value): move the cob_all_low and cob_all_high fields from global to local * codegen.c (output_collating_tables): remove local tables and code that loads the tables, since they are now loaded from cobc.c libcob: * strings.c: use the collating_sequence field of cob_module to determine the low value instead of the hard-coded constant "\0" | Program-level low/high collating values: compute per-program collating low/high in the frontend; runtime comparison (HIGH-VALUE / LOW-VALUE in presence of collating sequences) uses them |
| `486565722c48` | 2025-05-20 | RUNTIME_PORTED | Limit sed usage in testsuite, remove listing-sed cobc: * cobc.c (set_compile_date): fix SOURCE_DATE_EPOCH being ignored on subsequent invocations libcob: * common.c (cob_set_date_from_epoch): fix incorrect conversion of epoch (was off by one day) tests: * testsuite.src/run_misc.at, testsuite.src/syn_misc.at: reduce the use of sed by using SOURCE_DATE_EPOCH when possible and using @&t@ quadrigraphs in expected output with trailing spaces * listing-sed.sh: removed as no longer needed * atlocal.in, atlocal_win: remove the no longer needed UNIFY_LISTING variable | Fix SOURCE_DATE_EPOCH being ignored on subsequent invocations; fix epoch conversion; remove listing-sed dependency from harness |
| `946f3e638c8f` | 2025-05-22 | RUNTIME_PORTED | Simplify and fix computation of dates from epoch cobc: * common.c (cob_set_date_from_epoch): simplification, which also fixes incorrect conversion of epoch (was off by one day) tests: * atlocal.in, atlocal_win: set TZ=UTC globally to help get a reproducible output | Fix epoch date conversion (was off by one day) in the candidate date routines; adopt TZ=UTC global test environment |
| `eb8536cfcd33` | 2025-05-26 | RUNTIME_PORTED | follow up to [r5531]: further adjustemnts to date computation from epoch | Apply the further date-from-epoch adjustments (final state of the family) |
| `a5253353db12` | 2025-06-02 | NOT_APPLICABLE_WITH_PROOF | portability fixes (and more) | None: C portability fixes across the native build |
| `410097c16722` | 2025-06-03 | NOT_APPLICABLE_WITH_PROOF | NEW CBL functions for VFILE functionality consistent with Microfocus / Fujitsu | None: CBL_* VFILE functions depend on the native VFILE backend |
| `ec76b500bb4f` | 2025-06-03 | NOT_APPLICABLE_WITH_PROOF | first VFILE update | None: VFILE is a native file backend (Microfocus/Fujitsu virtual file system) |
| `3dd1d88da6ff` | 2025-07-17 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | initial (unfinished) support for XML PARSE | Track only: initial XML PARSE support is unfinished upstream and depends on the native XML backend |
| `9d87cdbab882` | 2025-07-17 | NOT_APPLICABLE_WITH_PROOF | libcob/mlio.c: cater for libxml2 ABI break with LIBXML_VERSION >= 21200 by providing a check and definition of sax error handler with the old ABI | None: libxml2 ABI compatibility shim is native-library-specific |
| `7fef5fde70af` | 2025-07-28 | NOT_APPLICABLE_WITH_PROOF | c89/c23 compat and hardening adjustments, along with updated gettext infrastructure | None: C89/C23 source-compat and hardening plus gettext autotools infrastructure are native C build concerns |
| `8954b5fc10e6` | 2025-07-30 | RUNTIME_PORTED | Code and testsuite cleanup | Port the observable effects of the code cleanup across move/screenio/termio/mlio/fileio; adopt the updated tests (data_display, run_accept, run_extensions, run_file, run_manual_screen, run_misc, run_returncode, syn_*) |
| `277a07c2ee9c` | 2025-10-17 | FRONTEND_REIMPLEMENTED | improve SD syntax checks and error recovery | Port the SD syntax-check behavior plus its tests; ensure no hang on malformed SD |
| `5bb0fbe1bb59` | 2025-10-21 | RUNTIME_PORTED | Fix CHAR and ORD intrinsics in presence of collating sequence libcob: * intrinsic.c (cob_intr_char, cob_intr_ord): consider the program collating sequence in CHAR and ORD * intrinsic.c (cob_intr_char): raise COB_EC_ARGUMENT_FUNCTION when calling CHAR with an argument outside the collation range | CHAR and ORD intrinsics must consider the program collating sequence; CHAR outside collation range raises COB_EC_ARGUMENT_FUNCTION |
| `bf0b5878a898` | 2025-11-12 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | XML and JSON updates | Track only: XML and JSON updates (native backend) |
| `0359d0a78f10` | 2025-11-12 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | XML and JSON updates | Track only: XML and JSON updates (native libxml2/json-c backend) |
| `bc5c13b27467` | 2025-11-14 | NOT_APPLICABLE_WITH_PROOF | portability updates | None: C portability updates |
| `b836c467e7ed` | 2025-11-18 | RUNTIME_PORTED | cleanup memory handling in libcob for restart | Cleanup memory handling in libcob for restart: port module-restart state cleanup semantics |
| `a207a45955ec` | 2025-11-20 | RUNTIME_PORTED | new runtime configuration COB_SIGNAL_REGIME, allows skipping registration of the signal handler | Implement COB_SIGNAL_REGIME: valid values; registration policy (do-not-register / register-only-if-unclaimed / any other admitted modes); do not clobber external handlers; Unix coverage; classify unsupported platforms; async-signal-safe; runtime reporting |
| `4b72d0a9faac` | 2025-12-04 | RUNTIME_PORTED | improve memory handling in edge-cases | Improve memory handling in edge cases: port any observable bounds/state fixes; adopt tests |
| `50b58f682700` | 2025-12-29 | RUNTIME_PORTED | new COB_LOAD_GLOBAL boolean | Implement COB_LOAD_GLOBAL runtime configuration: determine upstream default and platform history; define interpreted-module equivalent distinguishing local vs global registry visibility; test preload, duplicates, CANCEL/reload, process isolation; keep native-DSO non-claim |
| `02964e42e1fa` | 2026-03-02 | RUNTIME_PORTED | Rename is_test to cob_is_test in libcob as it is an extern value | Rename the candidate runtime's exported is_test value to cob_is_test; update all references |

### config area

| commit | date | status | subject | action |
|---|---|---|---|---|
| `8ea9ac449c98` | 2022-01-28 | NOT_APPLICABLE_WITH_PROOF | Redispatch ChangeLog entries | None: ChangeLog entry redispatch only |
| `289c9aef58a9` | 2022-02-04 | CONFIGURATION_INTEGRATED | [GCOS] Add GCOS configuration file | Adopt the GCOS configuration file |
| `470f7db125a4` | 2024-01-16 | FRONTEND_REIMPLEMENTED | adjusted error handling | Adopt the adjusted error-handling behavior: error/warning selection, exit status, listings expectations (run_fundamental, run_misc, listings, syn_*); rm-strict.conf alignment |
| `7b6995042c4d` | 2024-04-09 | RUNTIME_PORTED | Add a profiling feature cobc: * parser.y: generate calls to "cob_prof_function_call" in the parsetree when profiling is unabled, when entering/leaving profiled blocks * flag.def: add `-fprof` to enable profiling * tree.h: add a flags field to cb_goto, add profiling fields to cb_program, add cb_prof_call enum and export cb_build_prof_call and cb_prof_procedure_fivision functions * tree.c (cb_build_program): initialize the new profiling fields of the cb_program structure * tree.c (cb_build_goto): add a "flags" argument (stored in the cb_program structure) * typeck.c (cb_emit_goto): add a "flags" argument (passed to cb_build_goto) * codegen.c: handle profiling code generation under the cb_flag_prof guard libcob: * Makefile.am: add `profiling.c` to sources * profiling.c: implement profiling functions (time spent in each procedure of the program) * common.c: add 4 environments variables COB_PROF_FILE, COB_PROF_MAX_DEPTH,COB_PROF_ENABLE and COB_PROF_FORMAT * common.c (cob_expand_env_string): add $b (executable basename), $f (executable filename), $d (date in yyyymmdd) and $t (time in hhmmss) * common.c (cob_set_main_argv0): extracted from cob_init * fileio.c (cob_path_to_absolute): extracted from insert and cob_set_main_argv0 config: * runtime.cfg: add COB_PROF_FILE | Implement the profiling feature for the interpreted candidate: -fprof flag; per-procedure time accounting in the interpreter; COB_PROF_FILE/COB_PROF_MAX_DEPTH/COB_PROF_ENABLE/COB_PROF_FORMAT env support; $b/$f/$d/$t expansion in env strings |
| `4695ee78629d` | 2024-05-02 | CONFIGURATION_INTEGRATED | mf dialect: adjusted missing-statement configuration [bugs:#965] | Adopt mf dialect missing-statement configuration |
| `1ea4059c6547` | 2024-05-02 | NOT_APPLICABLE_WITH_PROOF | configure now uses pkg-config/ncurses-config to search for ncurses and honors NCURSES_LIBS and NCURSES_CFLAGS | None: configure ncurses detection via pkg-config |
| `1fa8db0d0e6b` | 2024-07-11 | CONFIGURATION_INTEGRATED | Fix minor alignment / tab issues in config/*.conf | Adopt the alignment/tab normalization: all config/*.conf synced to the pinned upstream head bytes |
| `6bf47af0209e` | 2024-08-19 | RUNTIME_PORTED | [feature-request:#474]: add runtime configuration to hide cursor for extended screenio | Implement runtime configuration to hide the cursor for extended screenio |
| `111d21f03445` | 2024-09-20 | TEST_IMPORTED | Minor adjustments (testsuite, ChangeLog entries, C89) | Adopted by the current-upstream suite lane (syn_definition.at updates); the pplex/scanner changes are C89-internal |
| `a2e4627e6a48` | 2025-01-10 | CONFIGURATION_INTEGRATED | [GCOS dialect] Set init-justify to no * config/gcos-strict.conf: set init-justify to no after testing on GCOS | Adopt init-justify=no for the GCOS-strict dialect (applied by the gcos-strict.conf custody refresh) |
| `87500ead47bd` | 2025-01-10 | FRONTEND_REIMPLEMENTED | fixed [bugs:#961]: Nested Elements Mishandled Despite 'with attributes' Specification | Fix nested-element handling with the 'with attributes' specification (SCREEN SECTION data-name qualification) |
| `54d4963026a1` | 2025-03-31 | NOT_APPLICABLE_WITH_PROOF | Add an EBCDIC/ASCII table generation feature build_windows: * general for cobc: include new gentable.c cobc: * gentable.c: generate EBCDIC/ASCII translation tables * cobc.c, help.c: new --gentable option doc: * gnucobol.texi: document the new --gentable option | None: --gentable generates native C translation tables (EBCDIC/ASCII) |
| `410097c16722` | 2025-06-03 | NOT_APPLICABLE_WITH_PROOF | NEW CBL functions for VFILE functionality consistent with Microfocus / Fujitsu | None: CBL_* VFILE functions depend on the native VFILE backend |
| `7b324f50ebbb` | 2025-10-05 | FRONTEND_REIMPLEMENTED | parser cleanup and better handling of incomplete code | Parser cleanup + better handling of incomplete code: bounded recovery, no hangs, deterministic diagnostics |
| `a207a45955ec` | 2025-11-20 | RUNTIME_PORTED | new runtime configuration COB_SIGNAL_REGIME, allows skipping registration of the signal handler | Implement COB_SIGNAL_REGIME: valid values; registration policy (do-not-register / register-only-if-unclaimed / any other admitted modes); do not clobber external handlers; Unix coverage; classify unsupported platforms; async-signal-safe; runtime reporting |
| `50b58f682700` | 2025-12-29 | RUNTIME_PORTED | new COB_LOAD_GLOBAL boolean | Implement COB_LOAD_GLOBAL runtime configuration: determine upstream default and platform history; define interpreted-module equivalent distinguishing local vs global registry visibility; test preload, duplicates, CANCEL/reload, process isolation; keep native-DSO non-claim |

### tests area

| commit | date | status | subject | action |
|---|---|---|---|---|
| `289c9aef58a9` | 2022-02-04 | CONFIGURATION_INTEGRATED | [GCOS] Add GCOS configuration file | Adopt the GCOS configuration file |
| `0166302909e9` | 2023-08-17 | RUNTIME_PORTED | fix [bugs:#904] MOVE PACKED-DECIMAL unsigned to signed leads to bad sign | Fix MOVE PACKED-DECIMAL unsigned to signed bad sign |
| `60557e874dec` | 2023-08-22 | TEST_IMPORTED | missing commit for [r5167] - version increase | Adopted by the current-upstream suite lane (version-output expectation in run_misc.at) |
| `9d4be36a13ea` | 2023-09-16 | TEST_IMPORTED | correction of testsuite for [r5190] | Adopted by the current-upstream suite lane (testsuite correction for r5190) |
| `777852c35adf` | 2023-10-17 | TEST_IMPORTED | testcase for [r5195] / [bugs:#923] | Adopted by the current-upstream suite lane; the behavior it tests is ported with 303917744 (module constants) |
| `470f7db125a4` | 2024-01-16 | FRONTEND_REIMPLEMENTED | adjusted error handling | Adopt the adjusted error-handling behavior: error/warning selection, exit status, listings expectations (run_fundamental, run_misc, listings, syn_*); rm-strict.conf alignment |
| `04614ac7afd2` | 2024-01-17 | RUNTIME_PORTED | Optimizations and syntax checks for INSPECT related functions | INSPECT optimizations and syntax checks: frontend syntax validation + runtime INSPECT behavior alignment |
| `62b39805ca22` | 2024-01-18 | RUNTIME_PORTED | Fixing [bugs:#914] CLOSE LOCK abends program on OPEN | Fix CLOSE LOCK abend on OPEN (file state handling) |
| `0b22d441757e` | 2024-01-19 | RUNTIME_PORTED | Fixing [bugs:#913] DISPLAY and ACCEPT with simple attributes SIGSEGV | Fix DISPLAY and ACCEPT with simple attributes SIGSEGV (candidate screen statements with attribute handling) |
| `8e2ec25c26bc` | 2024-01-20 | RUNTIME_PORTED | fix [bugs:#918] partial broken COB_LS_VALIDATE | Fix partial broken COB_LS_VALIDATE (line-sequential validation) |
| `f67da51cae38` | 2024-01-22 | RUNTIME_PORTED | Fix bug #917: segfault when accessing a decimal constant after calling a sub-program cobc: * codegen.c (codegen_internal, codegen_finalize): move declaration   of decimal constants from global storage to local storage to   fix bug #917 (segfault on decimal constant after CANCEL on   subprogram) | Decimal constants must live per-module (local storage) and be re-initialized after CANCEL — candidate module state model |
| `140a030d52ee` | 2024-01-22 | WRAPPER_INTEGRATED | New flag -fdiagnostics-absolute-path to display full paths within error locations * error.c (print_error_prefix), flag.def: new flag -fdiagnostics-absolute-paths to print the full path of a file for diagnostics; this flag can be activated if your editor and build system do not correctly work together to locate files from diagnostic output | Implement -fdiagnostics-absolute-path flag (full paths within diagnostics) |
| `44848f58b437` | 2024-01-22 | WRAPPER_INTEGRATED | Minor fixes * cobc/cobc.c (cobc_clean_up): when save-temps specifies a directory, do not move object files and preprocess files when they were specified as an explicit target on the command line (-E, -c) * libcob/common.c (cob_get_strerror), libcob/coblocal.h: export as utility function * libcob/common.c (cob_expand_env_string): fix potention buffer overflow | save-temps directory behavior: do not move object/preprocessed files when an explicit target (-E, -c) was given; adopt env-string expansion overflow fix |
| `6e358998b272` | 2024-01-22 | TEST_IMPORTED | Fix falses positives due to path differences in testsuite (run_misc.at) on Windows | Adopted by the current-upstream suite lane (Windows path-difference fix in run_misc.at) |
| `e36a124b2b72` | 2024-01-31 | WRAPPER_INTEGRATED | Add options --copy COPYBOOK and --include HEADER to cobc | Implement --copy COPYBOOK and --include HEADER options (adopt source-location mapping) |
| `300b542f3caa` | 2024-02-16 | NOT_APPLICABLE_WITH_PROOF | fileio refactoring | Verify in Phase 2: fileio refactoring (native C internals); no candidate-visible behavior expected |
| `106e7ce6c98c` | 2024-02-20 | NOT_APPLICABLE_WITH_PROOF | FR #459: support COLLATING SEQUENCE clause on SELECT / INDEXED files (currently only for the BDB backend) cobc: * codegen.c (output_file_initialization): output the indexed file/keys collating sequence (were already present in the AST) * tree.c (validate_indexed_key_field): process postponed key collating sequences * parser.y (collating_sequence_clause, collating_sequence_clause_key): replace CB_PENDING by CB_UNFINISHED on file and key collating sequence * flag.def, tree.c, tree.h, cobc.c, parser.y: add and handle a new -fdefault-file-colseq flag to specify the default collating sequence to use for files without a collating sequence clause libcob: * fileio.c (bdb_setkeycol, bdb_bt_compare, indexed_open, ...): take the file collating sequence into account when comparing keys * common.c, coblocal.h: rename common_cmps to cob_cmps and make it available locally | None: COLLATING SEQUENCE clause on SELECT/INDEXED files (BDB only) plus -fdefault-file-colseq flag affecting only indexed files |
| `61479ba0c781` | 2024-03-13 | FRONTEND_REIMPLEMENTED | Fix [bugs:#947]: VALUE ALL "-" not working in SCREEN SECTION * cobc/parser.y (screen_value_clause): replaced basic literals by literals | Fix VALUE ALL "-" in SCREEN SECTION (literal handling) |
| `14f0d0908d98` | 2024-03-13 | FRONTEND_REIMPLEMENTED | Fix SEGFAULT in checking prototype arguments * cobc/parser.y: fix SEGFAULT when checking the BY VALUE arguments of a prototype with ANY LENGTH | Fix SEGFAULT when checking BY VALUE arguments of a prototype with ANY LENGTH (checker robustness) |
| `7b6995042c4d` | 2024-04-09 | RUNTIME_PORTED | Add a profiling feature cobc: * parser.y: generate calls to "cob_prof_function_call" in the parsetree when profiling is unabled, when entering/leaving profiled blocks * flag.def: add `-fprof` to enable profiling * tree.h: add a flags field to cb_goto, add profiling fields to cb_program, add cb_prof_call enum and export cb_build_prof_call and cb_prof_procedure_fivision functions * tree.c (cb_build_program): initialize the new profiling fields of the cb_program structure * tree.c (cb_build_goto): add a "flags" argument (stored in the cb_program structure) * typeck.c (cb_emit_goto): add a "flags" argument (passed to cb_build_goto) * codegen.c: handle profiling code generation under the cb_flag_prof guard libcob: * Makefile.am: add `profiling.c` to sources * profiling.c: implement profiling functions (time spent in each procedure of the program) * common.c: add 4 environments variables COB_PROF_FILE, COB_PROF_MAX_DEPTH,COB_PROF_ENABLE and COB_PROF_FORMAT * common.c (cob_expand_env_string): add $b (executable basename), $f (executable filename), $d (date in yyyymmdd) and $t (time in hhmmss) * common.c (cob_set_main_argv0): extracted from cob_init * fileio.c (cob_path_to_absolute): extracted from insert and cob_set_main_argv0 config: * runtime.cfg: add COB_PROF_FILE | Implement the profiling feature for the interpreted candidate: -fprof flag; per-procedure time accounting in the interpreter; COB_PROF_FILE/COB_PROF_MAX_DEPTH/COB_PROF_ENABLE/COB_PROF_FORMAT env support; $b/$f/$d/$t expansion in env strings |
| `82100d64de35` | 2024-04-27 | NOT_APPLICABLE_WITH_PROOF | Optimization of memory usage in replace.c | None: memory optimization in the C replace.c; candidate REPLACE layer is independent Rust |
| `442e6db6d430` | 2024-05-03 | NOT_APPLICABLE_WITH_PROOF | build and test fixes for Win32 | None: Win32 build and test fixes |
| `a0937bf4920e` | 2024-05-04 | FRONTEND_REIMPLEMENTED | fixing [bugs:#933] [bugs:#938] [bugs:#966] handling of broken expressions | Improve handling of broken expressions (recovery, no hangs, correct reject) |
| `7c60012c019b` | 2024-05-04 | TEST_IMPORTED | fixing typo | Adopted by the current-upstream suite lane (typo fix in run_misc.at) |
| `ed789c8a9bc2` | 2024-05-05 | PLATFORM_BEHAVIOR_INTEGRATED | Win32 fixes, mostly testcases | Adopt Win32-relevant test expectations via the current-upstream lane; the common.c/fileio.c parts are Windows-path fixes |
| `1daa3931493b` | 2024-05-06 | TEST_IMPORTED | portability fix for [r5249] | Adopted by the current-upstream suite lane (portability fix for r5249) |
| `d2df58ad9685` | 2024-05-14 | NOT_APPLICABLE_WITH_PROOF | assorted minor cleanups | None: assorted minor cleanups (C-wide, no single observable behavior; verify with tests) |
| `9261f4096868` | 2024-05-14 | NOT_APPLICABLE_WITH_PROOF | minor cleanups and warning fixes | None: minor cleanups and warning fixes in libcob (no observable behavior delta; verify with tests) |
| `63cb06ce7b87` | 2024-05-15 | NOT_APPLICABLE_WITH_PROOF | more compiler warnings fixed | None: compiler warnings fixed (C internal) |
| `d671493076c3` | 2024-05-15 | NOT_APPLICABLE_WITH_PROOF | improving cobc handling with MSVC assembler | None: cobc handling with the MSVC assembler (native assembly path) |
| `940f057e6522` | 2024-06-20 | NOT_APPLICABLE_WITH_PROOF | Windows build fixes + ChangeLog cleanups cobc/cobc.c (process_compile): fix MSVC build command tests/atlocal_win: fix path-related issues in Windows builds | None: Windows/MSVC build command fixes + atlocal_win path fixes |
| `7c7b55b9311b` | 2024-07-05 | RUNTIME_PORTED | Adjustment for move to edited numeric | Adjustment for move to edited numeric (with tests) |
| `b33a87961f0d` | 2024-07-05 | NOT_APPLICABLE_WITH_PROOF | Adjustment to READ PREVIOUS for VBISAM / VISAM | None: READ PREVIOUS for VBISAM/VISAM (relative-indexed native backend) |
| `e51b091b9921` | 2024-07-06 | RUNTIME_PORTED | Fix for bug 934 - default ROUNDED option | Fix default ROUNDED option behavior |
| `9f1a64c32e11` | 2024-07-23 | RUNTIME_PORTED | [feature-requests:#448] using state structures instead of state vars for strings | Use state structures instead of state vars for STRING/UNSTRING/INSPECT: port the reworked string-operation state handling |
| `0fa2bf5f5238` | 2024-07-26 | FRONTEND_REIMPLEMENTED | increase portability for Micro Focus and ACUCOBOL-GT | Increase dialect portability for Micro Focus and ACUCOBOL-GT (reserved words/config.def/parser) |
| `ec5562cfb9f6` | 2024-07-30 | RUNTIME_PORTED | Adjustment to support the 2023 standard for edited numeric picture strings and to fix [bugs:#935] | Support the 2023 standard for edited numeric picture strings and fix bugs:#935 (picture-string validation + runtime edited move) |
| `db0e8067d3e8` | 2024-07-31 | TEST_IMPORTED | fix small error in compile error expected results | Adopted by the current-upstream suite lane (compile-error expected results fix) |
| `1b01ffd2398e` | 2024-08-03 | TEST_IMPORTED | Testuite fixes for MSVC * testsuite.src/run_file.at, testsuite.src/run_misc.at: fix a few tests that break under MSVC Debug while working under MSVC Release, by forcing a flush of stdout with fflush and using cob_free instead of free in C codes | Adopted by the current-upstream suite lane (MSVC test fixes) |
| `71ea358aa910` | 2024-08-10 | FRONTEND_REIMPLEMENTED | work on ALPHABET definitions, especially ALPHABET FOR NATIONAL | Implement ALPHABET definitions, especially ALPHABET FOR NATIONAL (parse + collating behavior) |
| `6bf47af0209e` | 2024-08-19 | RUNTIME_PORTED | [feature-request:#474]: add runtime configuration to hide cursor for extended screenio | Implement runtime configuration to hide the cursor for extended screenio |
| `808c9be88a50` | 2024-08-27 | HARNESS_ADOPTED | Retrieve archive of NIST test suite from sourceforge instead of from an out-dated URL | Adopt the current NIST archive URL in the candidate NIST harness |
| `5a8666888fad` | 2024-08-28 | FRONTEND_REIMPLEMENTED | Fix bugs reported by the MSVC runtime checker cobc: * tree.c (char_to_precedence_idx, get_char_type_description, valid_char_order): adjusted size of precedence table and gave proper precedence to U libcob: * intrinsics.c (cob_intr_random), move.c (cob_move_display_to_packed): make casts with loss of data explicit using masking to silence the MSVC runtime error checker | Give 'U' proper precedence in the expression precedence table (parser); port masking fixes in random/packed move where the candidate has equivalent numeric paths |
| `97668518028e` | 2024-09-09 | HARNESS_ADOPTED | work on "make checkmanual" | Adopt checkmanual workflow improvements in the candidate doc harness |
| `a234462ff94b` | 2024-09-13 | TEST_IMPORTED | testsuite environment update | Adopted by the current-upstream suite lane (testsuite environment update) |
| `111d21f03445` | 2024-09-20 | TEST_IMPORTED | Minor adjustments (testsuite, ChangeLog entries, C89) | Adopted by the current-upstream suite lane (syn_definition.at updates); the pplex/scanner changes are C89-internal |
| `7b09c750ff7d` | 2024-09-20 | RUNTIME_PORTED | Fix [bugs:#990] COBOL screen, problem positioning cursor on line 1 | Fix cursor positioning on line 1 (COBOL screen) |
| `1104bda61e19` | 2024-09-25 | FRONTEND_REIMPLEMENTED | Check for incompatible data only when a receiver is of category numeric in MOVE or SET | Check for incompatible data in MOVE or SET only when the receiver is of category numeric |
| `7b3047cb2616` | 2024-09-27 | HARNESS_ADOPTED | updaste for NIST85 | Adopt the NIST85 run-definition updates in the candidate NIST85 harness |
| `903ba84ff9db` | 2024-09-29 | NOT_APPLICABLE_WITH_PROOF | assorted updates | Verify in Phase 2: mixed C cleanup/updates (cobc, libcob, build, tests) without a single identified candidate-visible behavior; no known semantic delta |
| `49da19a3dfc0` | 2024-09-30 | WRAPPER_INTEGRATED | Add dependencies options and -fcopybook-deps cobc: * pplex.l (cb_text_list): prevent duplicates * cobc.c, help.c, pplex.l: add new flags to output dependencies following gcc: -M to output deps only, -MD to output deps while compiling (in .d files), -MP to output phony targets, -MG to keep missing copybooks, -MQ <target> to Makefile-quote target ; add -fcopybook-deps to output only copybook names instead of file paths. -fcopybook-deps also forces -E, -foneline-deps, -MT=copybooks, disables errors on missing copybooks and removes output on stdout doc: * gnucobol.texi: document new dependencies options | Implement -M/-MD/-MP/-MG/-MQ dependency options + -fcopybook-deps (copybook-only deps, forces -E -foneline-deps -MT=copybooks, disables missing-copybook errors) |
| `9b0259d78f87` | 2024-10-01 | NOT_APPLICABLE_WITH_PROOF | Support collating sequence for indexed file keys of alphanumeric class | None: collating sequence for indexed file keys of alphanumeric class (indexed backend) |
| `710f053fbd7c` | 2024-10-02 | TEST_IMPORTED | testsuite update for special cases | Adopted by the current-upstream suite lane (special-cases test updates) |
| `3f7c44b6f516` | 2024-10-02 | WRAPPER_INTEGRATED | improve stdin compilation | Improve stdin compilation: cobc-rs must compile from stdin with the documented naming/artifact behavior |
| `c53ae5f80351` | 2024-10-02 | RUNTIME_PORTED | signal handler updates | Signal handler updates: port the handler registration/behavior semantics |
| `ca09f172185f` | 2024-10-04 | TEST_IMPORTED | minor testuite update | Adopted by the current-upstream suite lane (minor testsuite update) |
| `b583a357302a` | 2024-10-08 | HARNESS_ADOPTED | build and test updates | Adopt the build-and-test updates relevant to the candidate harness |
| `0cc8207d14de` | 2024-10-11 | TEST_IMPORTED | follow-up to r5356 - fixed skip via atlocal_win | Adopted by the current-upstream suite lane (skip via atlocal_win) |
| `190139b8baee` | 2024-10-11 | TEST_IMPORTED | follow-up to r5356 - fixed skip via atlocal_win | Adopted by the current-upstream suite lane (skip via atlocal_win) |
| `ac862070c3e8` | 2024-10-24 | RUNTIME_PORTED | fixed [bugs:#999] ACCEPT with TIMEOUT issue when looping thru the verb | Fix ACCEPT with TIMEOUT looping through the verb (candidate ACCEPT TIMEOUT semantics) |
| `a6c4f2440452` | 2024-11-18 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | Add XML/JSON GENERATE tests for PIC P | Track tests: XML/JSON GENERATE tests for PIC P depend on the native XML/JSON backend |
| `2a53351eae5a` | 2024-11-18 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | Add PANEL functions from CURSES | Track only: PANEL functions from CURSES (native curses/panel dependency) |
| `0bf2ceb38ea4` | 2024-11-18 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | follow-up to r5369 - panel update | Track only: PANEL update + tests (curses) |
| `45ce8f622930` | 2024-11-19 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | follow-up to r5369 - panel update | Track only: PANEL update + tests (curses) |
| `87c1dd5799ff` | 2024-12-02 | RUNTIME_PORTED | Fixed [bugs:#1008] regression in move to numeric edited items * move.c (optimized_move_display_to_edited): fixed Bug #1008: regression in move to numeric edited items with insertion symbols B, 0 and / * move.c (optimized_move_display_to_edited): minor refactoring * exception.def, move.c: added definition for COBOL2025 COB_EC_DATA_NULL + COB_EC_DATA_TRUNCATION (currently not used) | Fix move-to-edited regression with insertion symbols B, 0 and /; register COBOL2025 COB_EC_DATA_NULL and COB_EC_DATA_TRUNCATION exception definitions |
| `b162a03c3d94` | 2024-12-08 | NOT_APPLICABLE_WITH_PROOF | minor update for sanitizers | None: sanitizer-related minor C adjustments |
| `1c357b4a3894` | 2024-12-12 | NOT_APPLICABLE_WITH_PROOF | Fixed [bugs:1032]: app_data field of DBT structure is not always copied when call bdb_bt_compare * libcob/fileio.c: fixed Bug #1032 by always using global thread-static variable bdb_app_data pointer to access the collating sequence function | None: BDB DBT app_data fix |
| `44c96d20a12e` | 2024-12-17 | RUNTIME_PORTED | Fix BLANK WHEN ZERO not working on signed NUMERIC-EDITED fields libcob: * move.c (optimized_move_display_to_edited): normalize numeric data * move.c (cob_move): extend use of optimized_move_display_to_edited to more cases (i.e. different source and destination sign, leading sign, non-separate sign) * move.c (optimized_move_display_to_edited): fixed additional bug reported with bug #1008: BLANK WHEN ZERO not working on signed NUMERIC-EDITED fields | BLANK WHEN ZERO on signed NUMERIC-EDITED fields: normalize numeric data in edited move; extend edited-move to sign variants |
| `cb5fe73262cf` | 2024-12-19 | RUNTIME_PORTED | Fix STRING/UNSTRING/INSPECT bug introduced in r5302 * libcob/string.c: fix a bug where the source of STRING/UNSTRING/INSPECT is overwritten, by restoring the *_copy fields that were removed with the change on 2024-02-26 | Fix STRING/UNSTRING/INSPECT source-overwrite bug (source fields must not be clobbered mid-operation) |
| `921108ea29fc` | 2024-12-20 | RUNTIME_PORTED | follow-up commit to r5403 - fix an out of bounds read access in optimized_move_display_to_edited | Fix out-of-bounds read in optimized move DISPLAY->edited (candidate edited move bounds) |
| `8cec9fdb89c0` | 2024-12-24 | RUNTIME_PORTED | Improve display of floats * libcob/termio.c (clean_double): skip more than a single leading zero in exponent display | Float display: skip more than a single leading zero in exponent digits (candidate float formatting) |
| `47ec5f513416` | 2025-01-06 | NOT_APPLICABLE_WITH_PROOF | Improve handling of partial keys * libcob/fileio.c (indexed_start_internal): improve handling of partial keys, to ensure BDB always compares keys of identical length | None: BDB indexed partial-key comparison |
| `87500ead47bd` | 2025-01-10 | FRONTEND_REIMPLEMENTED | fixed [bugs:#961]: Nested Elements Mishandled Despite 'with attributes' Specification | Fix nested-element handling with the 'with attributes' specification (SCREEN SECTION data-name qualification) |
| `8a7c349d13ad` | 2025-02-12 | FRONTEND_REIMPLEMENTED | FR #176: "Implementation of GC directive to include .h (c/c++) files" cobc: * pplex.l, ppparse.y, cobc.h, codegen.c (output_gnucobol_defines): new >>IMP INCLUDE directive to include one or multiple header files in the generated C code (same behavior as the --include but with one directive per file) * scanner.l: the leading space for all internal directives is removed in the lexer. Source previously preprocessed may need to be adjusted | Implement the >>IMP INCLUDE directive (include .h/.c++ headers) at the preprocessing level; adopt the scanner change (leading space removed for internal directives) |
| `3f99dba47432` | 2025-03-26 | NOT_APPLICABLE_WITH_PROOF | minor, mostly build updates | None: minor mostly-autotools build updates |
| `54d4963026a1` | 2025-03-31 | NOT_APPLICABLE_WITH_PROOF | Add an EBCDIC/ASCII table generation feature build_windows: * general for cobc: include new gentable.c cobc: * gentable.c: generate EBCDIC/ASCII translation tables * cobc.c, help.c: new --gentable option doc: * gnucobol.texi: document the new --gentable option | None: --gentable generates native C translation tables (EBCDIC/ASCII) |
| `0a761c9fa42c` | 2025-04-04 | TEST_IMPORTED | Fix SIGTERM test randomly failing in tests/testsuite.src/used_binaries.at | Adopted by the current-upstream suite lane (SIGTERM test flake fix) |
| `f2106ff244e7` | 2025-04-07 | TEST_IMPORTED | Follow-up to r5473 - add missing comment | Adopted by the current-upstream suite lane (comment-only follow-up) |
| `dc0cddebe0f0` | 2025-04-15 | WRAPPER_INTEGRATED | Fixes to the dependency generation feature introduced by [r5345] cobc: * cobc.c (process_filename): ensure we don't keep the preprocessed file when using -M or -fcopybook-deps * cobc.c, cobc.h, help.c, pplex.l: make -fcopybook-deps an experimental feature, activable with the EXPERIMENTAL_COPYBOOK_DEPS_OPTION flag | Fix -M/-fcopybook-deps behavior: do not keep the preprocessed file; gate -fcopybook-deps behind the experimental option; adopt tests |
| `79c65d0ecf1a` | 2025-04-16 | FRONTEND_REIMPLEMENTED | Fix [bugs:#948]: comparison with HIGH-VALUE in presence of collating sequences cobc: * tree.h (cb_program): add low_value and high_value fields to hold the low and high values used by the program collating sequence * tree.c (cb_build_program): initialize the low_value and high_value fields to reasonable default values * typeck.c: replace hard-coded cob_refer_ascii and cob_refer_ebcdic by ebcdic_to_ascii and ascii_to_ebcdic * typeck.c (cb_validate_collating): set the program's low_value and high_value fields * typeck.c (validate_alphabet): use the new tables, set the alphabet's low and high values * cobc.h: export the new symbols defined in typeck.c * cobc.c (process_command_line): always load the collating table * scanner.l (scan_ebcdic_char): remove code that loads and use a local collating table, use the table defined in typeck.c instead * codegen.c: replace hard-coded 0 and 255 / 0xff contants with the low_value and high_value fields where appropriate * codegen.c (output_low_value, output_high_value): move the cob_all_low and cob_all_high fields from global to local * codegen.c (output_collating_tables): remove local tables and code that loads the tables, since they are now loaded from cobc.c libcob: * strings.c: use the collating_sequence field of cob_module to determine the low value instead of the hard-coded constant "\0" | Program-level low/high collating values: compute per-program collating low/high in the frontend; runtime comparison (HIGH-VALUE / LOW-VALUE in presence of collating sequences) uses them |
| `da5c185222c7` | 2025-05-13 | HARNESS_ADOPTED | Testing and overriding the diff command * configure.ac: testing working diff with the option to override by DIFF tests: * atlocal.in, atlocal_win, cobol85/Makefile.am, cobol85/Makefile.module.in, testsuite.src/*.at: use the new DIFF variable to invoke the diff command | Adopt DIFF-override support in the candidate testsuite harness |
| `486565722c48` | 2025-05-20 | RUNTIME_PORTED | Limit sed usage in testsuite, remove listing-sed cobc: * cobc.c (set_compile_date): fix SOURCE_DATE_EPOCH being ignored on subsequent invocations libcob: * common.c (cob_set_date_from_epoch): fix incorrect conversion of epoch (was off by one day) tests: * testsuite.src/run_misc.at, testsuite.src/syn_misc.at: reduce the use of sed by using SOURCE_DATE_EPOCH when possible and using @&t@ quadrigraphs in expected output with trailing spaces * listing-sed.sh: removed as no longer needed * atlocal.in, atlocal_win: remove the no longer needed UNIFY_LISTING variable | Fix SOURCE_DATE_EPOCH being ignored on subsequent invocations; fix epoch conversion; remove listing-sed dependency from harness |
| `946f3e638c8f` | 2025-05-22 | RUNTIME_PORTED | Simplify and fix computation of dates from epoch cobc: * common.c (cob_set_date_from_epoch): simplification, which also fixes incorrect conversion of epoch (was off by one day) tests: * atlocal.in, atlocal_win: set TZ=UTC globally to help get a reproducible output | Fix epoch date conversion (was off by one day) in the candidate date routines; adopt TZ=UTC global test environment |
| `a5253353db12` | 2025-06-02 | NOT_APPLICABLE_WITH_PROOF | portability fixes (and more) | None: C portability fixes across the native build |
| `410097c16722` | 2025-06-03 | NOT_APPLICABLE_WITH_PROOF | NEW CBL functions for VFILE functionality consistent with Microfocus / Fujitsu | None: CBL_* VFILE functions depend on the native VFILE backend |
| `ec76b500bb4f` | 2025-06-03 | NOT_APPLICABLE_WITH_PROOF | first VFILE update | None: VFILE is a native file backend (Microfocus/Fujitsu virtual file system) |
| `1fc700cc0cd9` | 2025-07-17 | NOT_APPLICABLE_WITH_PROOF | c89 compat adjustments | None: C89 compatibility adjustments for the C compiler |
| `7fef5fde70af` | 2025-07-28 | NOT_APPLICABLE_WITH_PROOF | c89/c23 compat and hardening adjustments, along with updated gettext infrastructure | None: C89/C23 source-compat and hardening plus gettext autotools infrastructure are native C build concerns |
| `8954b5fc10e6` | 2025-07-30 | RUNTIME_PORTED | Code and testsuite cleanup | Port the observable effects of the code cleanup across move/screenio/termio/mlio/fileio; adopt the updated tests (data_display, run_accept, run_extensions, run_file, run_manual_screen, run_misc, run_returncode, syn_*) |
| `f4ffd50ecd24` | 2025-07-30 | FRONTEND_REIMPLEMENTED | reserved word handling and trace update | Reserved-word handling update + trace update: adopt the changed reserved-word set and trace output |
| `94c8c561555a` | 2025-09-17 | HARNESS_ADOPTED | fix #1142 build system support for embeded paths | Adopt build-system support for embedded paths in the candidate harness |
| `7b324f50ebbb` | 2025-10-05 | FRONTEND_REIMPLEMENTED | parser cleanup and better handling of incomplete code | Parser cleanup + better handling of incomplete code: bounded recovery, no hangs, deterministic diagnostics |
| `277a07c2ee9c` | 2025-10-17 | FRONTEND_REIMPLEMENTED | improve SD syntax checks and error recovery | Port the SD syntax-check behavior plus its tests; ensure no hang on malformed SD |
| `5bb0fbe1bb59` | 2025-10-21 | RUNTIME_PORTED | Fix CHAR and ORD intrinsics in presence of collating sequence libcob: * intrinsic.c (cob_intr_char, cob_intr_ord): consider the program collating sequence in CHAR and ORD * intrinsic.c (cob_intr_char): raise COB_EC_ARGUMENT_FUNCTION when calling CHAR with an argument outside the collation range | CHAR and ORD intrinsics must consider the program collating sequence; CHAR outside collation range raises COB_EC_ARGUMENT_FUNCTION |
| `d877fb362d20` | 2025-10-31 | HARNESS_ADOPTED | test runner: perf record addition and quote-fix | Adopt perf-record support and quote fixes in the candidate test runner |
| `bf0b5878a898` | 2025-11-12 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | XML and JSON updates | Track only: XML and JSON updates (native backend) |
| `8dd5b382cf01` | 2025-11-17 | HARNESS_ADOPTED | quoting adjustments for use of builddir | Adopt the builddir quoting adjustments in the candidate testsuite harness |
| `a3d9d6435401` | 2025-11-18 | HARNESS_ADOPTED | drop bashisms in atlocal.in and pre-inst-env.in | Adopt the bashism removal in the candidate testsuite harness |
| `b836c467e7ed` | 2025-11-18 | RUNTIME_PORTED | cleanup memory handling in libcob for restart | Cleanup memory handling in libcob for restart: port module-restart state cleanup semantics |
| `13963e15a2da` | 2025-11-18 | WRAPPER_INTEGRATED | ensure full output for -ftcmd using multiple continuation lines as necessary | -ftcmd listing output must continue across multiple lines instead of truncating (candidate listing generation) |
| `a207a45955ec` | 2025-11-20 | RUNTIME_PORTED | new runtime configuration COB_SIGNAL_REGIME, allows skipping registration of the signal handler | Implement COB_SIGNAL_REGIME: valid values; registration policy (do-not-register / register-only-if-unclaimed / any other admitted modes); do not clobber external handlers; Unix coverage; classify unsupported platforms; async-signal-safe; runtime reporting |
| `2c092ca140b4` | 2025-11-27 | FRONTEND_REIMPLEMENTED | check for terminating periods at the end of SET directives | Check for terminating periods at the end of SET directives; accept/reject and diagnose per upstream |
| `39ab4808c7e5` | 2025-12-02 | WRAPPER_INTEGRATED | listing header change: basename only | Listing header must show the basename only (candidate listing generation) |
| `4b72d0a9faac` | 2025-12-04 | RUNTIME_PORTED | improve memory handling in edge-cases | Improve memory handling in edge cases: port any observable bounds/state fixes; adopt tests |
| `c4eea8102820` | 2025-12-05 | FRONTEND_REIMPLEMENTED | fix areacheck - ENTRY statement should begin in area B not area A | Fix area-check: ENTRY statement must begin in area B, not area A (candidate checker area validation) |
| `34efe755f6f4` | 2025-12-06 | TEST_IMPORTED | test update | Adopted by the current-upstream suite lane (test update) |
| `9e0d66418efc` | 2025-12-29 | FRONTEND_REIMPLEMENTED | cobc/tree.c (finalize_file): if file is EXTFH enabled then don't warn for ORGANIZATION INDEXED, even when compiler is configured --without-db | Suppress the ORGANIZATION INDEXED warning when the file is EXTFH-enabled (candidate checker must not warn where upstream does not) |
| `50b58f682700` | 2025-12-29 | RUNTIME_PORTED | new COB_LOAD_GLOBAL boolean | Implement COB_LOAD_GLOBAL runtime configuration: determine upstream default and platform history; define interpreted-module equivalent distinguishing local vs global registry visibility; test preload, duplicates, CANCEL/reload, process isolation; keep native-DSO non-claim |
| `47dda86c0013` | 2026-05-26 | FRONTEND_REIMPLEMENTED | Config option tab-width can receive a list of comma-separated widths | Implement -ftab-width=w1,w2,... list semantics: each 1..12, last repeats indefinitely, malformed/empty/overflow lists fail with stable config errors; apply to fixed/free/auto formats, preprocessing, listing, diagnostics; repeated options follow upstream precedence |
| `a672a26b52b5` | 2026-06-08 | FRONTEND_REIMPLEMENTED | Fix handling of some special contexts, and provide room for more | Implement typed nested parser-context mechanism (ContextSet + stack with enter/leave guards, recovery cleanup, leak assertions); extend beyond 32 flags; match upstream accept/reject for CALL convention, CALL USING, REPOSITORY, EXIT, USAGE, TYPEDEF, SPECIAL-NAMES, VALIDATE STATUS, READY/RESET contexts |

### bin area

| commit | date | status | subject | action |
|---|---|---|---|---|
| `42d9e7de0eb8` | 2024-09-06 | NOT_APPLICABLE_WITH_PROOF | missing changelog entry for [r4915] | None: ChangeLog-only commit |
| `88937849b860` | 2024-10-02 | CONFIGURATION_INTEGRATED | new options for configure for customized version string / bug report URL | Adopt configurable version string / bug-report URL surfaces where the candidate exposes equivalents |
| `3f99dba47432` | 2025-03-26 | NOT_APPLICABLE_WITH_PROOF | minor, mostly build updates | None: minor mostly-autotools build updates |
| `7fef5fde70af` | 2025-07-28 | NOT_APPLICABLE_WITH_PROOF | c89/c23 compat and hardening adjustments, along with updated gettext infrastructure | None: C89/C23 source-compat and hardening plus gettext autotools infrastructure are native C build concerns |

### build area

| commit | date | status | subject | action |
|---|---|---|---|---|
| `c140aafc1568` | 2023-08-22 | CONFIGURATION_INTEGRATED | configure.ac: add -fstack-clash-protection to --enable-hardening[=no] | None: configure.ac hardening flag is native C build infrastructure |
| `12e31f960ebe` | 2023-12-14 | NOT_APPLICABLE_WITH_PROOF | minor doc adjustments and build_windows/config.h adjustment for 3.3-dev | None: 3.3-dev build_windows/config.h adjustments |
| `2f9892458c54` | 2024-01-22 | NOT_APPLICABLE_WITH_PROOF | Fix bug #920: Codegen: output of integer literals in generated C broken with MinGW * configure.ac: add checks to allow using stdint.h and inttypes.h * libcob/common.h: use stdint.h and inttypes.h when available to define cob_s64_t, cob_u64_t and the various CB_FMT_ macros | None: MinGW integer-literal codegen + stdint usage |
| `1ea4059c6547` | 2024-05-02 | NOT_APPLICABLE_WITH_PROOF | configure now uses pkg-config/ncurses-config to search for ncurses and honors NCURSES_LIBS and NCURSES_CFLAGS | None: configure ncurses detection via pkg-config |
| `63bd0f81fa4d` | 2024-05-14 | CONFIGURATION_INTEGRATED | fix macOS testsuite issues * configure.ac: update flags for building dynamic libraries on macOS (helps fixing testsuite issues on recent macOS versions) | None: native macOS build/test flags |
| `71ea358aa910` | 2024-08-10 | FRONTEND_REIMPLEMENTED | work on ALPHABET definitions, especially ALPHABET FOR NATIONAL | Implement ALPHABET definitions, especially ALPHABET FOR NATIONAL (parse + collating behavior) |
| `9744112d5560` | 2024-09-07 | CONFIGURATION_INTEGRATED | build system update | None: autotools build system update |
| `10daa94c8936` | 2024-09-27 | NOT_APPLICABLE_WITH_PROOF | build system update | None: libtool/autotools build system update |
| `903ba84ff9db` | 2024-09-29 | NOT_APPLICABLE_WITH_PROOF | assorted updates | Verify in Phase 2: mixed C cleanup/updates (cobc, libcob, build, tests) without a single identified candidate-visible behavior; no known semantic delta |
| `88937849b860` | 2024-10-02 | CONFIGURATION_INTEGRATED | new options for configure for customized version string / bug report URL | Adopt configurable version string / bug-report URL surfaces where the candidate exposes equivalents |
| `b583a357302a` | 2024-10-08 | HARNESS_ADOPTED | build and test updates | Adopt the build-and-test updates relevant to the candidate harness |
| `2a53351eae5a` | 2024-11-18 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | Add PANEL functions from CURSES | Track only: PANEL functions from CURSES (native curses/panel dependency) |
| `d5eb0eb02335` | 2024-11-19 | BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY | follow-up to r5369 - panel update | Track only: PANEL update (curses) |
| `c2ee239a5209` | 2024-12-08 | CONFIGURATION_INTEGRATED | follow-up to r5369 - panel update | None: configure.ac follow-up is native build infra |
| `dda41815fe1f` | 2024-12-08 | CONFIGURATION_INTEGRATED | fix copy+paste error in r5389 | None: configure.ac copy+paste fix is native build infra |
| `c265f251f14f` | 2025-02-11 | CONFIGURATION_INTEGRATED | Fix configure.ac for Clang * configure.ac: add -Wno-unused-command-line-argument to CFLAGS under Clang, to prevent some features to be mistakenly detected as missing (in particular -Wno-pointer-sign and -fstack-clash-protection) | None: configure.ac Clang flag is native build infra |
| `7824bb9f16e4` | 2025-02-11 | CONFIGURATION_INTEGRATED | configure cleanup | None: configure cleanup is native build infra |
| `3f99dba47432` | 2025-03-26 | NOT_APPLICABLE_WITH_PROOF | minor, mostly build updates | None: minor mostly-autotools build updates |
| `aa297a7c6743` | 2025-03-27 | CONFIGURATION_INTEGRATED | build update | None: autotools build update |
| `54d4963026a1` | 2025-03-31 | NOT_APPLICABLE_WITH_PROOF | Add an EBCDIC/ASCII table generation feature build_windows: * general for cobc: include new gentable.c cobc: * gentable.c: generate EBCDIC/ASCII translation tables * cobc.c, help.c: new --gentable option doc: * gnucobol.texi: document the new --gentable option | None: --gentable generates native C translation tables (EBCDIC/ASCII) |
| `da5c185222c7` | 2025-05-13 | HARNESS_ADOPTED | Testing and overriding the diff command * configure.ac: testing working diff with the option to override by DIFF tests: * atlocal.in, atlocal_win, cobol85/Makefile.am, cobol85/Makefile.module.in, testsuite.src/*.at: use the new DIFF variable to invoke the diff command | Adopt DIFF-override support in the candidate testsuite harness |
| `486565722c48` | 2025-05-20 | RUNTIME_PORTED | Limit sed usage in testsuite, remove listing-sed cobc: * cobc.c (set_compile_date): fix SOURCE_DATE_EPOCH being ignored on subsequent invocations libcob: * common.c (cob_set_date_from_epoch): fix incorrect conversion of epoch (was off by one day) tests: * testsuite.src/run_misc.at, testsuite.src/syn_misc.at: reduce the use of sed by using SOURCE_DATE_EPOCH when possible and using @&t@ quadrigraphs in expected output with trailing spaces * listing-sed.sh: removed as no longer needed * atlocal.in, atlocal_win: remove the no longer needed UNIFY_LISTING variable | Fix SOURCE_DATE_EPOCH being ignored on subsequent invocations; fix epoch conversion; remove listing-sed dependency from harness |
| `a5253353db12` | 2025-06-02 | NOT_APPLICABLE_WITH_PROOF | portability fixes (and more) | None: C portability fixes across the native build |
| `7fef5fde70af` | 2025-07-28 | NOT_APPLICABLE_WITH_PROOF | c89/c23 compat and hardening adjustments, along with updated gettext infrastructure | None: C89/C23 source-compat and hardening plus gettext autotools infrastructure are native C build concerns |
| `94c8c561555a` | 2025-09-17 | HARNESS_ADOPTED | fix #1142 build system support for embeded paths | Adopt build-system support for embedded paths in the candidate harness |
| `bc5c13b27467` | 2025-11-14 | NOT_APPLICABLE_WITH_PROOF | portability updates | None: C portability updates |
| `8dd5b382cf01` | 2025-11-17 | HARNESS_ADOPTED | quoting adjustments for use of builddir | Adopt the builddir quoting adjustments in the candidate testsuite harness |
| `a3d9d6435401` | 2025-11-18 | HARNESS_ADOPTED | drop bashisms in atlocal.in and pre-inst-env.in | Adopt the bashism removal in the candidate testsuite harness |
| `f49bf5314302` | 2025-11-20 | CONFIGURATION_INTEGRATED | configure adjustments | None: configure adjustments are native build infra |
| `a207a45955ec` | 2025-11-20 | RUNTIME_PORTED | new runtime configuration COB_SIGNAL_REGIME, allows skipping registration of the signal handler | Implement COB_SIGNAL_REGIME: valid values; registration policy (do-not-register / register-only-if-unclaimed / any other admitted modes); do not clobber external handlers; Unix coverage; classify unsupported platforms; async-signal-safe; runtime reporting |

## Phase-2 integration evidence

| upstream commit | upstream date | status | Rust integration commit |
|---|---|---|---|
| `289c9aef58a9` | 2022-02-04 | CONFIGURATION_INTEGRATED | `7b97303952fe` |
| `0166302909e9` | 2023-08-17 | RUNTIME_PORTED | `37a3779b1d66` |
| `470f7db125a4` | 2024-01-16 | FRONTEND_REIMPLEMENTED | `5ca481883211` |
| `c3d5860bf219` | 2024-01-16 | RUNTIME_PORTED | `6921f51a3aba` |
| `28b02be15485` | 2024-01-16 | RUNTIME_PORTED | `6921f51a3aba` |
| `1fa8db0d0e6b` | 2024-07-11 | CONFIGURATION_INTEGRATED | `BATCH` |
| `a2e4627e6a48` | 2025-01-10 | CONFIGURATION_INTEGRATED | `7b97303952fe` |

## Lane-adopted test/harness commits (evidence at Phase 3)

These test-only / harness-only upstream changes are adopted by the current-upstream suite lane
(Phase 3): the lane runs the pinned source tree's own `.at` files, so the test updates are
exercised verbatim there. Evidence is recorded when that lane lands.

| commit | date | status | subject |
|---|---|---|---|
| `60557e874dec` | 2023-08-22 | TEST_IMPORTED | missing commit for [r5167] - version increase |
| `c140aafc1568` | 2023-08-22 | CONFIGURATION_INTEGRATED | configure.ac: add -fstack-clash-protection to --enable-hardening[=no] |
| `9d4be36a13ea` | 2023-09-16 | TEST_IMPORTED | correction of testsuite for [r5190] |
| `777852c35adf` | 2023-10-17 | TEST_IMPORTED | testcase for [r5195] / [bugs:#923] |
| `6e358998b272` | 2024-01-22 | TEST_IMPORTED | Fix falses positives due to path differences in testsuite (run_misc.at) on Windows |
| `7c60012c019b` | 2024-05-04 | TEST_IMPORTED | fixing typo |
| `ed789c8a9bc2` | 2024-05-05 | PLATFORM_BEHAVIOR_INTEGRATED | Win32 fixes, mostly testcases |
| `1daa3931493b` | 2024-05-06 | TEST_IMPORTED | portability fix for [r5249] |
| `63bd0f81fa4d` | 2024-05-14 | CONFIGURATION_INTEGRATED | fix macOS testsuite issues * configure.ac: update flags for building dynamic libraries on macOS (helps fixing testsuite issues on recent macOS versions) |
| `db0e8067d3e8` | 2024-07-31 | TEST_IMPORTED | fix small error in compile error expected results |
| `1b01ffd2398e` | 2024-08-03 | TEST_IMPORTED | Testuite fixes for MSVC * testsuite.src/run_file.at, testsuite.src/run_misc.at: fix a few tests that break under MSVC Debug while working under MSVC Release, by forcing a flush of stdout with fflush and using cob_free instead of free in C codes |
| `9744112d5560` | 2024-09-07 | CONFIGURATION_INTEGRATED | build system update |
| `a234462ff94b` | 2024-09-13 | TEST_IMPORTED | testsuite environment update |
| `111d21f03445` | 2024-09-20 | TEST_IMPORTED | Minor adjustments (testsuite, ChangeLog entries, C89) |
| `710f053fbd7c` | 2024-10-02 | TEST_IMPORTED | testsuite update for special cases |
| `88937849b860` | 2024-10-02 | CONFIGURATION_INTEGRATED | new options for configure for customized version string / bug report URL |
| `929b403b68ff` | 2024-10-03 | PLATFORM_BEHAVIOR_INTEGRATED | fix [r5349] missing PKGVERSION for build_windows |
| `ca09f172185f` | 2024-10-04 | TEST_IMPORTED | minor testuite update |
| `0cc8207d14de` | 2024-10-11 | TEST_IMPORTED | follow-up to r5356 - fixed skip via atlocal_win |
| `190139b8baee` | 2024-10-11 | TEST_IMPORTED | follow-up to r5356 - fixed skip via atlocal_win |
| `c2ee239a5209` | 2024-12-08 | CONFIGURATION_INTEGRATED | follow-up to r5369 - panel update |
| `dda41815fe1f` | 2024-12-08 | CONFIGURATION_INTEGRATED | fix copy+paste error in r5389 |
| `c265f251f14f` | 2025-02-11 | CONFIGURATION_INTEGRATED | Fix configure.ac for Clang * configure.ac: add -Wno-unused-command-line-argument to CFLAGS under Clang, to prevent some features to be mistakenly detected as missing (in particular -Wno-pointer-sign and -fstack-clash-protection) |
| `7824bb9f16e4` | 2025-02-11 | CONFIGURATION_INTEGRATED | configure cleanup |
| `aa297a7c6743` | 2025-03-27 | CONFIGURATION_INTEGRATED | build update |
| `0a761c9fa42c` | 2025-04-04 | TEST_IMPORTED | Fix SIGTERM test randomly failing in tests/testsuite.src/used_binaries.at |
| `f2106ff244e7` | 2025-04-07 | TEST_IMPORTED | Follow-up to r5473 - add missing comment |
| `26a5cba4eda9` | 2025-07-29 | CONFIGURATION_INTEGRATED | missing file in r5552 from gettext infrastructure update |
| `a3d9d6435401` | 2025-11-18 | HARNESS_ADOPTED | drop bashisms in atlocal.in and pre-inst-env.in |
| `f49bf5314302` | 2025-11-20 | CONFIGURATION_INTEGRATED | configure adjustments |
| `34efe755f6f4` | 2025-12-06 | TEST_IMPORTED | test update |

## Non-curated mechanical rows (CI / docs / build / test-only)

- `27788c5941de` 2022-02-04 [CI_ONLY_ACCOUNTED] GIT-specific settings, with CI setup and github workflow for Ubuntu, Windows and Macos
- `a86cb1055aeb` 2022-03-29 [DOCUMENTATION_TRACKED] Thanks OCamlPro contributors
- `197398dfd15a` 2022-05-19 [CI_ONLY_ACCOUNTED] Improve setup for CI jobs, with temporary focus on branch `gcos4gnucobol-3.x`
- `b203d95dc329` 2022-05-20 [CI_ONLY_ACCOUNTED] Use github actions to emit a distribution archive and test logs
- `5b54e1c993e9` 2022-06-16 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `c8c789911a52` 2022-07-07 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `a070a64abef8` 2022-07-07 [NOT_APPLICABLE_WITH_PROOF] Update .gitignore
- `6301de737d07` 2022-07-07 [CI_ONLY_ACCOUNTED] Use ubuntu CI to measure coverage
- `da9db1e4a01c` 2022-07-07 [CI_ONLY_ACCOUNTED] Adjust macos & windows CIs
- `42008b070960` 2022-07-07 [DOCUMENTATION_TRACKED] Add ChangeLog entry about coverage artifact
- `8c6cafb8b4a4` 2022-07-07 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #41 from nberth/fix-ci
- `6a24f9a47e65` 2022-07-08 [CI_ONLY_ACCOUNTED] Disable automated windows CI workflow
- `1dae6bf280a7` 2022-07-20 [CI_ONLY_ACCOUNTED] Fix handling of quotes in testsuite artifact name
- `4c04ed86e837` 2022-07-22 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #54 from nberth/fix-ci
- `e07100c7ce2a` 2022-07-25 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `e2505a60f8be` 2022-07-27 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `a913f3c96dec` 2022-07-29 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `6fec839eae01` 2022-08-25 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `cc3f6677dd17` 2022-09-01 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `1c3569d3a8d3` 2022-09-20 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `d7db95f77395` 2022-09-22 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `41e53b7db427` 2022-09-25 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `a5ea0d5c93ec` 2022-09-27 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `b3afd7a81dae` 2022-09-30 [CI_ONLY_ACCOUNTED] Fix ubuntu CI
- `24f2a1036db8` 2022-09-30 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #62 from nberth/fix-ci
- `87051279e6a6` 2022-09-30 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `6ff12e91dc10` 2022-10-04 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `1c7abb41b185` 2022-10-05 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `0ac36ad45763` 2022-11-08 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `0a62bc915695` 2022-11-15 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `173790f1ef60` 2022-11-15 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `d3c0b2f18bfc` 2022-11-18 [CI_ONLY_ACCOUNTED] improving MacOS CI
- `62dd11b8b92d` 2022-12-05 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `757ff9b2f769` 2022-12-06 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `5a1ed6ffe5bd` 2022-12-08 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `fe93f0d3b36d` 2022-12-13 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `91b7781e801f` 2022-12-15 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `516b813b3ca0` 2023-01-19 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `f550c9b865f1` 2023-01-23 [CI_ONLY_ACCOUNTED] Add working Github action files, except for Windows (#79)
- `db33c36f7670` 2023-01-26 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `d15d131af589` 2023-01-28 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `9df05779b4cd` 2023-01-30 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `7fd5d4936b2b` 2023-02-01 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `d4f364e44ee1` 2023-02-02 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `b63bf71a12b4` 2023-02-08 [NOT_APPLICABLE_WITH_PROOF] Add autofonce configuration
- `200150f627aa` 2023-02-10 [NOT_APPLICABLE_WITH_PROOF] remove ar-lib from GIT
- `608eac1800e0` 2023-02-10 [NOT_APPLICABLE_WITH_PROOF] Recommit ar-lib
- `2ca79d6ae9dd` 2023-02-11 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `bdd8837832d4` 2023-02-20 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `3d48698fb76e` 2023-02-21 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `7981d8aff1b1` 2023-04-12 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `fcd562f0f1cc` 2023-04-20 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `1992539c4fde` 2023-05-24 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `cca51ff27837` 2023-06-02 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `8a0796d9f09a` 2023-06-13 [CI_ONLY_ACCOUNTED] CI: check for c89 declaration (#97)
- `7ba3fb5bf898` 2023-06-13 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `2dc28255dbc8` 2023-06-20 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `8caf9b25a444` 2023-06-20 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `cb5f7fa19a6c` 2023-06-21 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `743651ffd971` 2023-06-23 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `57e7a2851308` 2023-07-02 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `4b8452abf1ca` 2023-07-03 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `a4abf11bf1cc` 2023-07-05 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `2a101a4bffdb` 2023-07-09 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `6b4405108a30` 2023-07-11 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `0ab36fd83692` 2023-07-26 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `c0d64addfd83` 2023-08-15 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `98a5c787c1e5` 2024-01-11 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `f059c849512a` 2024-01-22 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `93d5877e42b1` 2024-01-25 [NOT_APPLICABLE_WITH_PROOF] follow-up to r5208 "header include"
- `824f2a6445e0` 2024-01-26 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `2ff35a3e3725` 2024-01-31 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `4eece0f7ddcc` 2024-02-03 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `db7db96f8c08` 2024-02-19 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `57133577c7e5` 2024-02-20 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `a2a51fd5ea7f` 2024-03-13 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `89c45a3bc80c` 2024-03-13 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `eb74013d3e4f` 2024-03-18 [CI_ONLY_ACCOUNTED] Add/Update Windows workflows
- `2e620aa926b6` 2024-04-22 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `87c4fb2905ed` 2024-04-27 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `5ba97ae7594f` 2024-05-13 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into HEAD
- `4f76d0a2021e` 2024-05-13 [CI_ONLY_ACCOUNTED] Fix MacOS CI
- `70b4076e9e8d` 2024-05-14 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `4907e0d0683e` 2024-05-14 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #145 from ddeclerck/fix_macos_ci
- `21b5d516ffd3` 2024-05-16 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `39344cf66085` 2024-06-20 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `de6053aad234` 2024-07-12 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `4d51096de843` 2024-07-25 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #140 from ddeclerck/ci_msvc
- `00f6832684d8` 2024-07-26 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `a672fbba0388` 2024-08-01 [CI_ONLY_ACCOUNTED] Update MSVC & MSYS1 CI
- `04fe8aaf44d7` 2024-08-04 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `3e146193295e` 2024-08-11 [NOT_APPLICABLE_WITH_PROOF] added missing iconv.m4 update - follow-up to r5310
- `56a19214a986` 2024-08-12 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `d9ef15191468` 2024-08-20 [CI_ONLY_ACCOUNTED] Upgrade versions of github actions used in CI
- `dfb0c9c82f96` 2024-08-20 [CI_ONLY_ACCOUNTED] Swap install and tests in CI for Windows MSYS2
- `803826741663` 2024-08-21 [CI_ONLY_ACCOUNTED] Further CI adjustments
- `2ed1057b1d86` 2024-08-22 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `d0ef5aa6e124` 2024-08-23 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `c1665fc6e603` 2024-08-23 [CI_ONLY_ACCOUNTED] Adjust MSVC workflow now that error popups are disabled by default
- `d45913dd76da` 2024-08-26 [CI_ONLY_ACCOUNTED] Enforce a 45 minutes timeout on Windows CI workflows
- `fed0e25ca1f9` 2024-08-27 [CI_ONLY_ACCOUNTED] Further adjustments to the CI for MacOS
- `5be2d04b8492` 2024-08-27 [CI_ONLY_ACCOUNTED] Stop using `-pedantic` flag in Coverage and Warnings workflow
- `34d0bd4b354f` 2024-08-27 [CI_ONLY_ACCOUNTED] Various improvements in Warnings and Coverage workflow
- `e14cfcae03c2` 2024-08-27 [CI_ONLY_ACCOUNTED] Fix lookup of `newcob.val` in MacOS workflow
- `d4ea52e5afb0` 2024-08-28 [CI_ONLY_ACCOUNTED] Cache `newcob.val` instead of an archive
- `f13eede92da6` 2024-08-28 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `0226a4e160d3` 2024-08-28 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `d3c4e188dd30` 2024-08-28 [CI_ONLY_ACCOUNTED] Update MSVC CI
- `77522f50e0ab` 2024-08-28 [CI_ONLY_ACCOUNTED] Run testsuite even on Debug target in Windows MSVC CI
- `c566c4abc0f4` 2024-08-28 [CI_ONLY_ACCOUNTED] Cache `newcob.val` in Windows MSYS2 workflow as well
- `e31b1ad2059a` 2024-08-28 [CI_ONLY_ACCOUNTED] Tar MSYS2 distribution archive to avoid EMFILE error
- `fe28973b030f` 2024-08-28 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #170 from nberth/ci-adjustments-4-gcos4gnucobol-3.x
- `f36f1506a16d` 2024-09-19 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `015735daa9c5` 2024-09-20 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `e807aed2c9c9` 2024-09-22 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `2c000f51a11e` 2024-09-22 [CI_ONLY_ACCOUNTED] Update Windows workflows (upload testsuite.log on failure)
- `f5989ba77c79` 2024-09-26 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `e3283da76017` 2024-09-27 [CI_ONLY_ACCOUNTED] CI: adding minimal build
- `482206f49af4` 2024-09-27 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #184 from OCamlPro/ci-minimal-build
- `61ffca26726a` 2024-09-30 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `799c61376739` 2024-10-02 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `7d4a2fd772a6` 2024-10-07 [CI_ONLY_ACCOUNTED] ci adjustments
- `a23f0dc875f6` 2024-10-07 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #189 from OCamlPro/ci-update
- `68b82c88f548` 2024-10-11 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `04916dd9371e` 2024-11-22 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `16e84a584d7e` 2024-11-24 [CI_ONLY_ACCOUNTED] Fix macOS CI
- `1be8d3f3493c` 2024-12-06 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `129abba07f9c` 2024-12-09 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `06d814688dbc` 2024-12-10 [CI_ONLY_ACCOUNTED] Update macos.yml
- `d5abb19870d0` 2024-12-10 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #202 from OCamlPro/GitMensch-patch-1
- `47501705ee02` 2024-12-16 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `afb68b34db69` 2024-12-19 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `5086fdf05e34` 2024-12-20 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `3f34a2461a6a` 2024-12-30 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `c4fbcc3050ad` 2025-01-03 [UPSTREAM_MERGE_ACCOUNTED] Merge branch 'gnucobol-3.x' into gcos4gnucobol-3.x
- `731b81a327fe` 2025-01-07 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `2f58e5ff7c5a` 2025-01-09 [DOCUMENTATION_TRACKED] Fix documentation generation * doc/cbrunt.tex.gen: fix for missing "@end verbatim"
- `beeace4cded3` 2025-01-13 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `6698c4bf0e94` 2025-01-19 [CI_ONLY_ACCOUNTED] Fix Ubuntu CI (Coverage)
- `3fb682569d9e` 2025-01-28 [CI_ONLY_ACCOUNTED] Fix MacOS CI
- `f88c80ee1363` 2025-02-11 [CI_ONLY_ACCOUNTED] Fix MSYS2 CI
- `6cc5a5803005` 2025-02-11 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #217 from ddeclerck/fix_msys2_ci
- `369eb24f947e` 2025-02-12 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `85c708085fad` 2025-02-16 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `080e75630cc4` 2025-03-26 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `0cb3eab5945e` 2025-03-28 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `c48824511397` 2025-03-31 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `cdb87a8b3aa9` 2025-04-07 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `84359ec81a15` 2025-04-16 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `d90e2850e1ed` 2025-04-18 [CI_ONLY_ACCOUNTED] Update CIs
- `dca6c3e5ec0b` 2025-05-13 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `8c96392229ca` 2025-05-15 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #226 from ddeclerck/gc3_ci_update
- `0e2b41f2c63b` 2025-05-16 [CI_ONLY_ACCOUNTED] CI adjustments
- `1b2c19e0cae8` 2025-05-20 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `b57d2f3b36e3` 2025-05-21 [CI_ONLY_ACCOUNTED] Add 32-bit Ubuntu workflow
- `d3a3a3e6102f` 2025-05-22 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #232 from ddeclerck/add_32bit_ci
- `a4be0beded8f` 2025-07-17 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #235 from OCamlPro/gnucobol-3.x
- `a006789fa627` 2025-07-21 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `adf35557a63a` 2025-07-29 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `d8bd3f3a02c8` 2025-07-29 [CI_ONLY_ACCOUNTED] Adjust MSYS2 workflow timeout (was slightly too short)
- `33057ad3e052` 2025-07-30 [CI_ONLY_ACCOUNTED] Update MacOS CI (DB4 removal imminent)
- `f8a1a5c2766c` 2025-07-30 [CI_ONLY_ACCOUNTED] Add IBM POWER and Z CI
- `f50ab5754982` 2025-07-31 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `94efcf215788` 2025-10-20 [CI_ONLY_ACCOUNTED] Fix CI
- `1fb152e8b536` 2025-10-21 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gcos4gnucobol-3.x
- `db111f65bd01` 2025-11-06 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #251 from OCamlPro/gnucobol-3.x
- `31ba95f7a4c3` 2025-11-18 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #253 from OCamlPro/gnucobol-3.x
- `5e45c5e64f37` 2025-11-18 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #254 from OCamlPro/gnucobol-3.x
- `b1275d4ee475` 2025-11-19 [CI_ONLY_ACCOUNTED] Fix MacOS workflow
- `eda8905e4404` 2025-11-19 [CI_ONLY_ACCOUNTED] Improve MSYS1 workflow definition
- `8d7308d92849` 2025-11-20 [CI_ONLY_ACCOUNTED] Change name of main branch in CI workflows
- `d28f9fab8e27` 2025-11-20 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #255 from nberth/update-ci-branch
- `a98fcd30cef7` 2025-11-20 [CI_ONLY_ACCOUNTED] Fix CI
- `deeadffbafb7` 2025-11-20 [CI_ONLY_ACCOUNTED] Update windows-msvc.yml
- `1fc514e9d166` 2025-12-03 [UPSTREAM_MERGE_ACCOUNTED] Merge pull request #256 from ddeclerck/fix_ci
- `bc234cd17f19` 2025-12-03 [CI_ONLY_ACCOUNTED] Adjust MSYS2 CI timeout
- `da31b9286647` 2025-12-08 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x
- `fb8f358e91ce` 2025-12-12 [CI_ONLY_ACCOUNTED] Update windows-msvc.yml
- `3457bd5def5e` 2025-12-24 [CI_ONLY_ACCOUNTED] MSVC CI update
- `253e5cc2602e` 2026-01-27 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x
- `326ce553416e` 2026-03-23 [CI_ONLY_ACCOUNTED] windows-msvc: fix env var reference for dependencies
- `871f965fac83` 2026-05-05 [CI_ONLY_ACCOUNTED] workflow updates
- `a3accbe7616c` 2026-05-19 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x
- `568531bd417a` 2026-06-09 [UPSTREAM_MERGE_ACCOUNTED] Merge remote-tracking branch 'upstream/gnucobol-3.x' into gitside-gnucobol-3.x
- `5568b8fc770f` 2026-08-04 [CI_ONLY_ACCOUNTED] Fix and update CI

## Generator identity

- `lab/gnucobol-upstream-current/gen_atlas.py` + `atlas_overrides.json`
- status enum: RUNTIME_PORTED, FRONTEND_REIMPLEMENTED, WRAPPER_INTEGRATED, TEST_IMPORTED, HARNESS_ADOPTED, CONFIGURATION_INTEGRATED, PLATFORM_BEHAVIOR_INTEGRATED, DOCUMENTATION_TRACKED, CI_ONLY_ACCOUNTED, UPSTREAM_MERGE_ACCOUNTED, NOT_APPLICABLE_WITH_PROOF, SUPERSEDED_BY_LATER_COMMIT, BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY
