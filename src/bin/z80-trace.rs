//! Run a conformance manifest on the Rust core and write its JSON Lines trace.
//!
//! Usage: `z80-trace <manifest.json> [--out <trace.jsonl>]`
//!
//! Without `--out` the trace goes to stdout so it can be piped straight into
//! `python -m z80_python.conformance diff <manifest.json> -`. The run summary
//! (records, T-states, stop reason, captured CP/M output) goes to stderr.
//! Exit status is 0 on a completed run, 2 on a bad manifest or I/O error.

use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use z80_rust::conformance::{load_manifest, trace_manifest, StopReason};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut manifest_path: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        if arg == "--out" {
            out_path = args.next().map(PathBuf::from);
            if out_path.is_none() {
                eprintln!("error: --out requires a path");
                return ExitCode::from(2);
            }
        } else if manifest_path.is_none() {
            manifest_path = Some(PathBuf::from(arg));
        } else {
            eprintln!("usage: z80-trace <manifest.json> [--out <trace.jsonl>]");
            return ExitCode::from(2);
        }
    }
    let Some(manifest_path) = manifest_path else {
        eprintln!("usage: z80-trace <manifest.json> [--out <trace.jsonl>]");
        return ExitCode::from(2);
    };
    let manifest = match load_manifest(&manifest_path) {
        Ok(manifest) => manifest,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };
    let sink: Box<dyn Write> = match &out_path {
        Some(path) => match std::fs::File::create(path) {
            Ok(file) => Box::new(file),
            Err(error) => {
                eprintln!("error: {}: {error}", path.display());
                return ExitCode::from(2);
            }
        },
        None => Box::new(io::stdout().lock()),
    };
    let mut sink = BufWriter::with_capacity(1 << 20, sink);
    let run = match trace_manifest(&manifest, |record| record.write_to(&mut sink)) {
        Ok(run) => run,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };
    if let Err(error) = sink.flush() {
        // A closed pipe means the reader stopped early (it found a divergence).
        if error.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    }
    eprintln!(
        "{}: {} records, {} T-states, stopped on {}",
        manifest.name,
        run.steps,
        run.t_states,
        run.reason.as_str()
    );
    if let StopReason::Fault(fault) = &run.reason {
        eprintln!("fault: {fault}");
    }
    if !run.output.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&run.output));
    }
    ExitCode::SUCCESS
}
