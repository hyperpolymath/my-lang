#!/bin/bash -eu
<<<<<<< HEAD

cd $SRC/project
cargo +nightly fuzz build --release
cp fuzz/target/*/release/fuzz_* $OUT/
=======
cd $SRC/*/fuzz
cargo +nightly fuzz build
for target in fuzz_targets/*; do
    target_name=$(basename ${target%.rs})
    cp target/x86_64-unknown-linux-gnu/release/$target_name $OUT/
done
>>>>>>> 7f63c53cc206ad0448f9e17e5b74dde7cf393117
