//! Rung 1, self-contained: the two example manifests from z80-python produce
//! records equal to the committed reference traces.
//!
//! The authoritative check is `python -m z80_python.conformance diff`, which
//! CI also runs; this test lets `cargo test` catch a regression without
//! Python present.

use std::path::Path;

use serde_json::Value;
use z80_rust::conformance::{load_manifest, trace_manifest, StopReason};

fn check(name: &str, expected_reason: StopReason) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let manifest = load_manifest(&dir.join(format!("{name}.json"))).expect("manifest loads");
    let mut produced: Vec<Value> = Vec::new();
    let run = trace_manifest(&manifest, |record| {
        produced.push(serde_json::from_str(&record.to_json()).expect("record is JSON"));
        Ok(())
    })
    .expect("run completes");
    assert_eq!(run.reason, expected_reason);

    let reference = std::fs::read_to_string(dir.join(format!("{name}.jsonl"))).unwrap();
    let mut expected: Vec<Value> = reference
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("reference record is JSON"))
        .collect();
    for record in &mut expected {
        if let Some(instruction) = record["instruction"].as_object_mut() {
            instruction.remove("mnemonic");
            instruction.remove("operands");
        }
    }
    assert_eq!(produced.len(), expected.len(), "{name}: record count");
    for (position, (mine, theirs)) in produced.iter().zip(&expected).enumerate() {
        assert_eq!(mine, theirs, "{name}: record {position} differs");
    }
}

#[test]
fn flags_and_branches_matches_reference() {
    check("flags-and-branches", StopReason::Halted);
}

#[test]
fn interrupts_matches_reference() {
    check("interrupts", StopReason::Halted);
}
