#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# Differential-conformance harness for the QTT checker coupling (#1).
#
# Asserts:  ∀ (g,t) in a random corpus.
#               my_qtt::check g t  ==  (Coq-extracted check) g t
#
# Pipeline:
#   1. coqc Extract.v          -> OCaml extracted from the VERIFIED `check`
#   2. ocaml compile oracle.ml -> the verified-checker ORACLE binary
#   3. cargo run conformance_gen -> corpus.sexp  +  rust_results.txt
#   4. ./oracle < corpus.sexp  -> coq_results.txt
#   5. diff rust_results.txt coq_results.txt   (must be identical)
#
# Requires: coqc (8.18), ocamlfind + ocamlc/ocamlopt, cargo. All present in the
# proof CI image. Env: COUNT (default 3000), SEED (default 1).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SC="$(cd "$HERE/.." && pwd)"                       # solo-core dir
ROOT="$(cd "$SC/../../../.." && pwd)"              # repo root
BUILD="$HERE/_build"
COUNT="${COUNT:-3000}"
SEED="${SEED:-1}"

echo "== QTT checker conformance =="
echo "   solo-core: $SC"
echo "   repo root: $ROOT"
rm -rf "$BUILD"; mkdir -p "$BUILD"

echo "-- 1. extract verified check -> OCaml"
( cd "$BUILD" && coqc -R "$SC" SoloCore "$SC/Extract.v" >/dev/null )

echo "-- 2. compile the oracle"
cp "$HERE/oracle.ml" "$BUILD/oracle.ml"
# dependency-sort ALL extracted modules (robust to whatever Coq emits —
# PeanoNat, BinNat, ...); compile each module's .mli then .ml, oracle last.
( cd "$BUILD"
  FILES=""
  for ml in $(ocamldep -sort *.ml); do
    base="${ml%.ml}"
    [ -f "$base.mli" ] && FILES="$FILES $base.mli"
    FILES="$FILES $ml"
  done
  ocamlc -w -a -o oracle $FILES )

echo "-- 3. generate corpus + rust results (count=$COUNT seed=$SEED)"
( cd "$ROOT" && cargo build -q -p my-qtt --bin conformance_gen )
GEN="$ROOT/target/debug/conformance_gen"
"$GEN" "$COUNT" "$SEED" "$BUILD/corpus.sexp" "$BUILD/rust_results.txt"

echo "-- 4. run the verified oracle over the corpus"
"$BUILD/oracle" < "$BUILD/corpus.sexp" > "$BUILD/coq_results.txt"

echo "-- 5. compare"
RN=$(wc -l < "$BUILD/rust_results.txt")
CN=$(wc -l < "$BUILD/coq_results.txt")
echo "   rust lines=$RN  coq lines=$CN"
if diff -u "$BUILD/rust_results.txt" "$BUILD/coq_results.txt" > "$BUILD/diff.txt"; then
  TOT=$((COUNT * 3))
  echo "PASS: $TOT/$TOT results agree over $COUNT terms × {check, one-step, normal-form}:"
  echo "      Rust my_qtt::check ≡ extracted Coq check   (coupling #1)"
  echo "      Rust my_qtt::step1 ≡ extracted Coq step1   (coupling #2; step1 proved sound+complete vs the `step` relation)"
  exit 0
else
  echo "FAIL: Rust and verified-Coq results differ:"
  head -40 "$BUILD/diff.txt"
  exit 1
fi
