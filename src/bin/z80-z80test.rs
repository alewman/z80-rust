//! Headless runner for raxoft/z80test's `.tap` programs: rung 4 of the ladder.
//!
//! Usage: `z80-z80test <program.tap>...`
//!
//! A transcription of validation/z80test_runner.py. The test programs are
//! launched via `RANDOMIZE USR 32768` from a BASIC loader and depend on the
//! Spectrum ROM for two things only: printing a character (`RST 0x10`) and
//! selecting the output channel (`CHAN-OPEN` at 0x1601). Neither touches
//! tested CPU state, so this runner replaces both with the same stubs the
//! reference uses instead of a ROM image:
//!
//! - 0x0010: `OUT (0xFF),A ; RET` captures the printed character on a
//!   private port.
//! - 0x1601: a bare `RET`.
//!
//! The one other hardware dependency, an `IN A,(0xFE)` keyboard guard, is
//! satisfied by always returning 0xBF; every other port reads 0xFF.
//!
//! Exit status 0 only when every program prints "Result: all tests passed."

use std::path::PathBuf;
use std::process::ExitCode;

use z80_rust::{Bus, Z80};

const LOAD_STUB_RST10: [u8; 3] = [0xD3, 0xFF, 0xC9]; // OUT (0xFF),A ; RET
const LOAD_STUB_CHANOPEN: [u8; 1] = [0xC9]; // RET
const RST10_ADDRESS: usize = 0x0010;
const CHANOPEN_ADDRESS: usize = 0x1601;
const SENTINEL_RETURN: u16 = 0x0000;
const MAX_INSTRUCTIONS: u64 = 2_000_000_000;

struct SpectrumHost {
    memory: Box<[u8; 0x10000]>,
    out_chars: Vec<u8>,
}

impl Bus for SpectrumHost {
    fn read_byte(&mut self, addr: u16) -> u8 {
        self.memory[usize::from(addr)]
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        self.memory[usize::from(addr)] = value;
    }

    fn read_port(&mut self, addr: u16) -> u8 {
        if (addr & 0xFF) == 0xFE {
            0xBF
        } else {
            0xFF
        }
    }

    fn write_port(&mut self, addr: u16, value: u8) {
        if (addr & 0xFF) == 0xFF {
            self.out_chars.push(value);
        }
    }
}

/// Extract the CODE block payload and load address from a z80test `.tap`.
fn parse_tap_code_block(data: &[u8]) -> Result<(Vec<u8>, u16), String> {
    let mut pos = 0;
    let mut blocks: Vec<&[u8]> = Vec::new();
    while pos + 2 <= data.len() {
        let length = usize::from(u16::from_le_bytes([data[pos], data[pos + 1]]));
        pos += 2;
        let end = (pos + length).min(data.len());
        blocks.push(&data[pos..end]);
        pos += length;
    }
    if blocks.len() < 4 {
        return Err(format!(
            "expected a BASIC+CODE tap file, found {} blocks",
            blocks.len()
        ));
    }
    let code_header = blocks[2];
    if code_header.len() < 16 || code_header[0] != 0x00 || code_header[1] != 3 {
        return Err("expected a CODE header as the third tap block".to_string());
    }
    let length = usize::from(u16::from_le_bytes([code_header[12], code_header[13]]));
    let load_addr = u16::from_le_bytes([code_header[14], code_header[15]]);
    let code_data = blocks[3];
    if code_data.first() != Some(&0xFF) {
        return Err("expected a data block as the fourth tap block".to_string());
    }
    let payload = &code_data[1..];
    if payload.len() < length {
        return Err(format!(
            "CODE block length mismatch: header says {length}, got {}",
            payload.len()
        ));
    }
    Ok((payload[..length].to_vec(), load_addr))
}

struct Outcome {
    output: String,
    instructions: u64,
}

fn run_program(tap: &[u8]) -> Result<Outcome, String> {
    let (code, load_addr) = parse_tap_code_block(tap)?;
    let mut host = SpectrumHost {
        memory: Box::new([0; 0x10000]),
        out_chars: Vec::new(),
    };
    let start = usize::from(load_addr);
    host.memory[start..start + code.len()].copy_from_slice(&code);
    host.memory[RST10_ADDRESS..RST10_ADDRESS + LOAD_STUB_RST10.len()]
        .copy_from_slice(&LOAD_STUB_RST10);
    host.memory[CHANOPEN_ADDRESS..CHANOPEN_ADDRESS + LOAD_STUB_CHANOPEN.len()]
        .copy_from_slice(&LOAD_STUB_CHANOPEN);
    let mut cpu = Z80::new(host);
    cpu.sp = 0xFFFC;
    cpu.bus.memory[0xFFFC] = SENTINEL_RETURN as u8;
    cpu.bus.memory[0xFFFD] = (SENTINEL_RETURN >> 8) as u8;
    cpu.pc = load_addr;
    let mut instructions: u64 = 0;
    while cpu.pc != SENTINEL_RETURN {
        cpu.step().map_err(|fault| format!("{fault}"))?;
        instructions += 1;
        if instructions >= MAX_INSTRUCTIONS {
            return Err(format!(
                "z80test did not terminate within {MAX_INSTRUCTIONS} instructions"
            ));
        }
    }
    Ok(Outcome {
        // z80test prints Spectrum characters; everything it reports is ASCII.
        output: cpu.bus.out_chars.iter().map(|&byte| byte as char).collect(),
        instructions,
    })
}

fn main() -> ExitCode {
    let paths: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if paths.is_empty() {
        eprintln!("usage: z80-z80test <program.tap>...");
        return ExitCode::from(2);
    }
    let mut all_passed = true;
    for path in &paths {
        let tap = match std::fs::read(path) {
            Ok(tap) => tap,
            Err(error) => {
                eprintln!("error: {}: {error}", path.display());
                return ExitCode::from(2);
            }
        };
        let started = std::time::Instant::now();
        match run_program(&tap) {
            Ok(outcome) => {
                let passed = outcome.output.contains("Result: all tests passed.");
                all_passed &= passed;
                // z80test terminates lines with the Spectrum's 0x0D.
                let summary = outcome
                    .output
                    .split(['\r', '\n'])
                    .find(|line| line.starts_with("Result:"))
                    .unwrap_or("(no Result line)");
                println!(
                    "{}: {} ({} instructions, {:.1} s)",
                    path.display(),
                    summary,
                    outcome.instructions,
                    started.elapsed().as_secs_f64()
                );
                if !passed {
                    println!(
                        "--- output ---\n{}\n--- end ---",
                        outcome.output.replace('\r', "\n")
                    );
                }
            }
            Err(error) => {
                all_passed = false;
                println!("{}: error: {error}", path.display());
            }
        }
    }
    if all_passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
