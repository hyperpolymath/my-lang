// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Conformance + example integration tests.
//!
//! These tests turn the repository's existing `.my` fixtures into real,
//! `cargo test`-driven coverage. Previously they were only exercised by
//! `conformance/run_conformance.sh`, which (a) is easy to skip locally and
//! (b) drives the `my-cli` binary through a `--parse-only` flag that no longer
//! exists. Driving the library API directly removes the binary dependency and
//! lets the suite run under coverage instrumentation.
//!
//! Fixtures (relative to the workspace root):
//!   - `conformance/valid/*.my`   — MUST parse successfully
//!   - `conformance/invalid/*.my` — MUST fail to parse
//!   - `examples/*.my`            — MUST parse successfully
//!
//! New fixtures dropped into those directories are picked up automatically.
//!
//! ## Known parse gaps (fail-closed allowlist)
//!
//! Some `conformance/valid/*.my` fixtures describe *intended* grammar that the
//! current parser does not yet accept. Rather than delete those fixtures (which
//! would lose the spec) or weaken the suite, they are listed in
//! [`KNOWN_PARSE_GAPS`]. The test asserts that each allowlisted fixture STILL
//! fails to parse. The moment the parser learns to handle one, this test breaks
//! and tells you to delete the entry — so the allowlist can never silently rot.
//! See `TESTING.md` for the roadmap that tracks closing these gaps.

use std::fs;
use std::path::{Path, PathBuf};

/// Valid-grammar fixtures the parser does not yet accept.
///
/// Keyed by file name (within `conformance/valid/`) with the missing feature.
/// Remove an entry the moment its feature lands — the test will demand it.
const KNOWN_PARSE_GAPS: &[(&str, &str)] = &[
    ("v04_let.my", "assignment statements (`y = y + x;`) are not yet parsed"),
    ("v06_match.my", "`match` is not yet accepted as a function tail expression"),
    ("v07_agent.my", "unit type `()` in effect op signatures is not yet parsed"),
    ("v08_effect.my", "lambda syntax (`|n| => expr`) is not yet parsed"),
];

fn is_known_gap(file: &Path) -> Option<&'static str> {
    let name = file.file_name()?.to_str()?;
    KNOWN_PARSE_GAPS
        .iter()
        .find(|(f, _)| *f == name)
        .map(|(_, reason)| *reason)
}

/// Absolute path to the workspace root (two levels up from this crate).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(Path::parent) // repo root
        .expect("crate is nested two levels under the workspace root")
        .to_path_buf()
}

/// Collect every `*.my` file directly inside `dir`, sorted for determinism.
///
/// Non-recursive on purpose: subdirectories such as `examples/newsroom/` are
/// multi-file projects, not single-file parse fixtures.
fn my_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read fixture dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "my"))
        .collect();
    files.sort();
    files
}

/// Read a fixture, panicking with a useful message on failure.
fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()))
}

fn file_name(path: &Path) -> std::borrow::Cow<'_, str> {
    path.file_name().unwrap_or(path.as_os_str()).to_string_lossy()
}

#[test]
fn valid_conformance_fixtures_parse() {
    let dir = workspace_root().join("conformance/valid");
    let files = my_files(&dir);
    assert!(
        !files.is_empty(),
        "no valid conformance fixtures found in {}",
        dir.display()
    );

    for file in files {
        let source = read(&file);
        let result = my_lang::parse(&source);
        match is_known_gap(&file) {
            // Allowlisted fixtures must STILL fail. If one starts parsing,
            // the parser gained the feature — delete it from KNOWN_PARSE_GAPS.
            Some(reason) => assert!(
                result.is_err(),
                "valid/{} now PARSES — the parser gained support for: {reason}.\n\
                 Remove it from KNOWN_PARSE_GAPS in tests/conformance.rs.",
                file_name(&file),
            ),
            None => assert!(
                result.is_ok(),
                "expected valid/{} to parse, got error: {:?}",
                file_name(&file),
                result.err(),
            ),
        }
    }
}

#[test]
fn invalid_conformance_fixtures_are_rejected() {
    let dir = workspace_root().join("conformance/invalid");
    let files = my_files(&dir);
    assert!(
        !files.is_empty(),
        "no invalid conformance fixtures found in {}",
        dir.display()
    );

    for file in files {
        let source = read(&file);
        let result = my_lang::parse(&source);
        assert!(
            result.is_err(),
            "expected invalid/{} to be REJECTED by the parser, but it parsed successfully",
            file_name(&file),
        );
    }
}

#[test]
fn example_programs_parse() {
    let dir = workspace_root().join("examples");
    let files = my_files(&dir);
    assert!(
        !files.is_empty(),
        "no example programs found in {}",
        dir.display()
    );

    for file in files {
        let source = read(&file);
        let result = my_lang::parse(&source);
        assert!(
            result.is_ok(),
            "expected example {} to parse, got error: {:?}",
            file_name(&file),
            result.err(),
        );
    }
}
