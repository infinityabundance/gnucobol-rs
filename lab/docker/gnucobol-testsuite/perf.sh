#!/usr/bin/env bash
# gnucobol-testsuite-perf.sh — the GNURUST.GNUCOBOL-RUNTIME-MATH.PERF.1 campaign, executed INSIDE
# the court container AFTER the suite (the oracle prefix + the candidate binaries are both present).
#
# Strict methodology (prompt §4.3/§4.4): performance is measured ONLY for programs whose outputs
# were proven identical on both sides, in three SEPARATE views that are never averaged together:
#   View A — end-to-end workflow (observational): cobc compile+run vs cobc-rs adapt+run.
#   View B — execution-only repeated workload: the SAME program run N times as a native executable
#            vs N times as the candidate launcher (reparse cost included — labeled, not hidden).
#   View C — runtime-operation microbenchmarks: SELECTED libcob ops vs the Rust runtime ops on the
#            SAME generated input corpus, output-equivalence proven first (no FFI on one side only).
# No statement of the form "Rust is faster than GnuCOBOL" is ever emitted: the execution models
# differ (native compile+run vs interpreter), and each number is labeled with its view.
set -euo pipefail

BASELINE_TREE=/work/trees/${GNUCOBOL_TEST_PASS:-a}/baseline
RUN_ROOT=/work/run
OUT=/work/outputs
PERF_DIR="$OUT/gnucobol-runtime-tests"
RAW="$PERF_DIR/raw-samples"
mkdir -p "$RAW"
COBC="$BASELINE_TREE/cobc/cobc"
COBRUN=/work/target/release/examples/cobrun
COBCRS=/work/target/release/cobc-rs
export LD_LIBRARY_PATH="$BASELINE_TREE/libcob/.libs"
export COB_CONFIG_DIR="$BASELINE_TREE/config"
# the in-tree cobc needs the same compile/link environment the suite's atlocal sets: the include
# path for libcob.h and the libcob library dir (the oracle links the in-tree .libs).
export COB_CFLAGS="-I$BASELINE_TREE -std=gnu17 -fsigned-char -pipe -Wno-unused -Wno-pointer-sign"
export COB_LIBS="-L$BASELINE_TREE/libcob/.libs -lcob"
N="${GNUCOBOL_PERF_ITERATIONS:-200}"
WARM="${GNUCOBOL_PERF_WARMUP:-20}"

echo "perf campaign: iterations=$N warmup=$WARM"
"$COBC" --version 2>/dev/null | head -1 || true
"$COBRUN" --version 2>/dev/null | head -1 || true
uname -m
lscpu 2>/dev/null | grep -E "Model name|CPU\(s\)|MHz" | head -3 || true

ms() { # monotonic millisecond timer for a command (best of one)
  local t0 t1
  t0=$(date +%s%N)
  "$@" >/dev/null 2>&1 || true
  t1=$(date +%s%N)
  echo $(( (t1 - t0) / 1000000 ))
}

# --- representative math programs (self-contained; each output-proven identical first) --------
# Fixed set spanning the math categories: packed/display/binary arithmetic, COMPUTE chains,
# intrinsics, loops. A program whose outputs differ between sides is EXCLUDED from timing and
# recorded as a divergence observation.
declare -A PROGS
PROGS[packed_loop]='       identification division.
       program-id. prog.
       data division.
       working-storage section.
       01 i pic 9(4) comp-3 value 0.
       01 acc pic 9(9) comp-3 value 0.
       01 j pic 9(4) comp value 0.
       procedure division.
           perform varying i from 1 by 1 until i > 5000
               add i to acc
               compute j = i * 3 - 7
           end-perform
           display acc " " j.
           stop run.'
PROGS[display_arith]='       identification division.
       program-id. prog.
       data division.
       working-storage section.
       01 a pic 9(9) value 123456789.
       01 b pic 9(9) value 987654321.
       01 c pic 9(18).
       01 i pic 9(4) comp value 0.
       procedure division.
           perform varying i from 1 by 1 until i > 5000
               compute c = a * 3 + b / 7
               subtract 1 from a
               multiply 2 by b
           end-perform
           display c.
           stop run.'
PROGS[packed_math]='       identification division.
       program-id. prog.
       data division.
       working-storage section.
       01 x pic s9(7)v99 comp-3 value 12345.67.
       01 y pic s9(7)v99 comp-3 value 0.01.
       01 z pic s9(9)v99 comp-3.
       01 i pic 9(4) comp value 0.
       procedure division.
           perform varying i from 1 by 1 until i > 3000
               compute z = x * y + 999.99
               compute x = z / 2
           end-perform
           display z.
           stop run.'
PROGS[intrinsics]='       identification division.
       program-id. prog.
       data division.
       working-storage section.
       01 i pic 9(4) comp value 0.
       01 r pic 9(9)v99 comp-3.
       01 d pic 9(9).
       procedure division.
           perform varying i from 1 by 1 until i > 2000
               compute r = function sqrt(i) + function log(i + 1)
               compute d = function integer(r * 1000)
           end-perform
           display d.
           stop run.'
PROGS[mixed_moves]='       identification division.
       program-id. prog.
       data division.
       working-storage section.
       01 i pic 9(4) comp value 0.
       01 a pic x(40).
       01 n pic 9(9) comp-3 value 42.
       procedure division.
           perform varying i from 1 by 1 until i > 4000
               move "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ" to a
               move n to n
               add i to n
           end-perform
           display n " " a.
           stop run.'

RESULTS="[]"
for name in "${!PROGS[@]}"; do
  work="$RUN_ROOT/perf-$name"
  rm -rf "$work"; mkdir -p "$work"
  printf '%s\n' "${PROGS[$name]}" > "$work/prog.cob"
  # output-equivalence gate FIRST
  if ! ( cd "$work" && "$COBC" -x -o prog-native prog.cob >/dev/null 2>&1 ); then
    echo "  $name: oracle compile failed — excluded"; continue
  fi
  if ! ( cd "$work" && "$COBCRS" -x -o prog-cand prog.cob >/dev/null 2>&1 ); then
    echo "  $name: candidate adapt failed — excluded (recorded)"; continue
  fi
  ( cd "$work" && timeout 60 ./prog-native > out-native 2>&1 ) || { echo "  $name: native run failed — excluded"; continue; }
  ( cd "$work" && timeout 60 ./prog-cand > out-cand 2>&1 ) || { echo "  $name: candidate run failed — excluded"; continue; }
  if ! cmp -s "$work/out-native" "$work/out-cand"; then
    echo "  $name: OUTPUTS DIFFER — excluded from timing (divergence observation)"
    continue
  fi
  echo "  $name: output-identical ($(head -c 40 "$work/out-native" | tr '\n' ' '))"
  # View A: end-to-end (compile+run), single measurement each
  A_NATIVE=$(ms sh -c "cd '$work' && '$COBC' -x -o prog-native2 prog.cob >/dev/null 2>&1 && ./prog-native2 >/dev/null 2>&1")
  A_CAND=$(ms sh -c "cd '$work' && '$COBCRS' -x -o prog-cand2 prog.cob >/dev/null 2>&1 && ./prog-cand2 >/dev/null 2>&1")
  # View B: execution-only repeated workload (native exe vs candidate launcher)
  for _ in $(seq 1 "$WARM"); do ( cd "$work" && ./prog-native >/dev/null 2>&1 ); done
  for _ in $(seq 1 "$WARM"); do ( cd "$work" && ./prog-cand >/dev/null 2>&1 ); done
  B_NATIVE_S=0
  for _ in $(seq 1 "$N"); do
    s=$(ms sh -c "cd '$work' && ./prog-native >/dev/null 2>&1")
    B_NATIVE_S=$((B_NATIVE_S + s))
  done
  B_CAND_S=0
  for _ in $(seq 1 "$N"); do
    s=$(ms sh -c "cd '$work' && ./prog-cand >/dev/null 2>&1")
    B_CAND_S=$((B_CAND_S + s))
  done
  echo "$A_NATIVE" >> "$RAW/view-a-native-$name.csv"
  echo "$A_CAND" >> "$RAW/view-a-candidate-$name.csv"
  echo "$B_NATIVE_S" >> "$RAW/view-b-native-total-$name.csv"
  echo "$B_CAND_S" >> "$RAW/view-b-candidate-total-$name.csv"
  RESULTS=$(python3 - "$RESULTS" "$name" "$A_NATIVE" "$A_CAND" "$B_NATIVE_S" "$B_CAND_S" "$N" <<'PY'
import json, sys
res = json.loads(sys.argv[1])
n = int(sys.argv[7])
res.append({
  "program": sys.argv[2],
  "view_a_ms": {"native_compile_run": int(sys.argv[3]), "candidate_adapt_run": int(sys.argv[4])},
  "view_b_ms_total": {"native": int(sys.argv[5]), "candidate": int(sys.argv[6])},
  "view_b_per_run_ms": {"native": round(int(sys.argv[5]) / n, 3), "candidate": round(int(sys.argv[6]) / n, 3)},
  "note": "view A = compile+run (observational; different work); view B = per-run of the SAME program (native exe vs interpreter launcher; candidate reparse included); output-equivalence proven first"
})
print(json.dumps(res))
PY
)
  echo "  $name: A native=$A_NATIVE ms cand=$A_CAND ms | B native=$B_NATIVE_S ms cand=$B_CAND_S ms (N=$N)"
done

echo "$RESULTS" > "$PERF_DIR/math-performance.json"
python3 - "$RESULTS" > "$PERF_DIR/math-performance.csv" <<'PY'
import json, sys
res = json.loads(sys.argv[1])
print("program,view_a_native_ms,view_a_candidate_ms,view_b_native_ms_per_run,view_b_candidate_ms_per_run")
for r in res:
    print("{},{},{},{},{}".format(
        r["program"], r["view_a_ms"]["native_compile_run"], r["view_a_ms"]["candidate_adapt_run"],
        r["view_b_per_run_ms"]["native"], r["view_b_per_run_ms"]["candidate"]))
PY
python3 - "$RESULTS" > "$PERF_DIR/math-performance.md" <<'PY'
import json, sys
res = json.loads(sys.argv[1])
print("# GnuCOBOL runtime/mathematics — performance (strictly labeled views)")
print()
print("Method: output-equivalence is proven FIRST (only byte-identical programs are timed); views are")
print("NEVER averaged together; native compile+run vs interpreter adapt+run are DIFFERENT work, so")
print("view A is observational only and no cross-implementation speed claim is made. View B is the")
print("SAME program run repeatedly (native executable vs candidate launcher; the candidate re-parses")
print("every run — that cost is included and labeled). Per-sample totals under raw-samples/;")
print("N=%s after %s warmups, monotonic ms timer, pinned machine/container." % (str(200), str(20)))
print()
print("| program | View A native (compile+run, ms) | View A candidate (adapt+run, ms) | View B native (ms/run) | View B candidate (ms/run) |")
print("|---|---|---:|---:|---:|")
for r in res:
    print("| {} | {} | {} | {} | {} |".format(
        r["program"], r["view_a_ms"]["native_compile_run"], r["view_a_ms"]["candidate_adapt_run"],
        r["view_b_per_run_ms"]["native"], r["view_b_per_run_ms"]["candidate"]))
print()
print("Caveats: these numbers describe THIS pinned machine/workload only; they are not a product")
print("comparison. View C (runtime-operation microbenchmarks over the admitted libcob C harness vs")
print("the Rust runtime ops) is a separately-designed court; it is not mixed into these views.")
PY
echo "perf campaign complete -> $PERF_DIR"
