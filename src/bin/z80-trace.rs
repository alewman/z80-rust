//! Run a conformance manifest on the Rust core and write its JSON Lines trace.
//!
//! Usage: `z80-trace <manifest.json> [--out <trace.jsonl> | --no-trace]
//!         [--checkpoint-every <records> --checkpoint-dir <dir>]`
//!
//! Without `--out` the trace goes to stdout so it can be piped straight into
//! `python -m z80_python.conformance diff <manifest.json> -`. The run summary
//! (records, T-states, stop reason, captured CP/M output) goes to stderr.
//! Exit status is 0 on a completed run, 2 on a bad manifest or I/O error.
//!
//! With `--checkpoint-every N`, a manifest resuming the run at record 0, N,
//! 2N, ... is written into `--checkpoint-dir`, each with `max_steps` N, so a
//! long run can be diffed against the reference as parallel segments (see
//! `scripts/rung3.sh`). `--no-trace` skips formatting the records, which is
//! what makes writing the checkpoints for a multi-billion-record run quick.

use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use z80_rust::conformance::{load_manifest, trace_manifest_with, write_checkpoint, StopReason};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut manifest_path: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    let mut no_trace = false;
    let mut checkpoint_every: Option<u64> = None;
    let mut checkpoint_dir: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        if arg == "--out" {
            out_path = args.next().map(PathBuf::from);
            if out_path.is_none() {
                eprintln!("error: --out requires a path");
                return ExitCode::from(2);
            }
        } else if arg == "--no-trace" {
            no_trace = true;
        } else if arg == "--checkpoint-every" {
            checkpoint_every = args.next().and_then(|value| value.parse().ok());
            if checkpoint_every.is_none_or(|every| every == 0) {
                eprintln!("error: --checkpoint-every requires a positive record count");
                return ExitCode::from(2);
            }
        } else if arg == "--checkpoint-dir" {
            checkpoint_dir = args.next().map(PathBuf::from);
            if checkpoint_dir.is_none() {
                eprintln!("error: --checkpoint-dir requires a path");
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
    if checkpoint_every.is_some() != checkpoint_dir.is_some() {
        eprintln!("error: --checkpoint-every and --checkpoint-dir go together");
        return ExitCode::from(2);
    }
    if let Some(dir) = &checkpoint_dir {
        if let Err(error) = std::fs::create_dir_all(dir) {
            eprintln!("error: {}: {error}", dir.display());
            return ExitCode::from(2);
        }
    }
    let mut sink = BufWriter::with_capacity(1 << 20, sink);
    let run = match trace_manifest_with(
        &manifest,
        |record| {
            if no_trace {
                Ok(())
            } else {
                record.write_to(&mut sink)
            }
        },
        |step, cpu| match (checkpoint_every, &checkpoint_dir) {
            (Some(every), Some(dir)) if step % every == 0 => {
                write_checkpoint(&manifest, cpu, step, every, dir).map(|_| ())
            }
            _ => Ok(()),
        },
    ) {
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
