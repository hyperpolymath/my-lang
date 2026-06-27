#!/bin/bash -eu
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# OSS-Fuzz / ClusterFuzzLite build script: compile the cargo-fuzz targets in
# `fuzz/` and stage each binary into $OUT. Run by `compile` inside the
# base-builder-rust image (see .clusterfuzzlite/Dockerfile).
cd "$SRC/my-lang"
cargo +nightly fuzz build -O

release="$SRC/my-lang/fuzz/target/x86_64-unknown-linux-gnu/release"
for target in fuzz/fuzz_targets/*.rs; do
    name="$(basename "${target%.rs}")"
    cp "$release/$name" "$OUT/"
done
