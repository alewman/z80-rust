//! Runner for FUSE's Z80 core test set: rung 6 of the ladder.
//!
//! Usage: `z80-fuse <dir holding tests.in and tests.expected>`
//!
//! A transcription of z80-python's `validation/fuse_runner.py`, itself a
//! reproduction of FUSE's `coretest.c`: RAM filled with `DE AD BE EF`, port
//! reads returning the high byte of the port address, whole instructions
//! until the requested T-states have elapsed, and a comparison of the
//! registers, MEMPTR, I, R, the flip-flops, IM, the halted flag, the T-state
//! total, and every memory byte the expected file lists. Bus events are
//! outside the core's claim and are not compared.
//!
//! FUSE's expectations are emulator-derived. Six cases disagree with the
//! hardware-derived oracles the reference follows (see
//! `tests/test_fuse_suite.py` in z80-python for the reasons); they are
//! expected to diverge here too, strictly: the run fails if any other case
//! diverges, or if one of the six stops diverging.

use std::path::PathBuf;
use std::process::ExitCode;

use z80_rust::{Bus, Z80};

const KNOWN_DIVERGENCES: [(&str, &str); 6] = [
    (
        "76",
        "FUSE keeps PC on the HALT opcode; SingleStepTests 76.json advances it",
    ),
    (
        "edb2_1",
        "interrupted INIR: FUSE 1.6.0 predates the MEMPTR = PC+1 and flag findings",
    ),
    (
        "edb3_1",
        "interrupted OTIR: FUSE 1.6.0 predates the MEMPTR = PC+1 and flag findings",
    ),
    (
        "edb9_2",
        "interrupted CPDR: FUSE 1.6.0 does not take X/Y from the rewound PC",
    ),
    (
        "edba_1",
        "interrupted INDR: FUSE 1.6.0 predates the MEMPTR = PC+1 and flag findings",
    ),
    (
        "edbb_1",
        "interrupted OTDR: FUSE 1.6.0 predates the MEMPTR = PC+1 and flag findings",
    ),
];
const EXPECTED_CASES: usize = 1356;
const REGISTER_NAMES: [&str; 13] = [
    "AF", "BC", "DE", "HL", "AF'", "BC'", "DE'", "HL'", "IX", "IY", "SP", "PC", "MEMPTR",
];

struct Machine {
    registers: [u16; 13],
    i: u8,
    r: u8,
    iff1: bool,
    iff2: bool,
    im: u8,
    halted: bool,
    t_states: u64,
    memory: Vec<(u16, u8)>,
}

struct Case {
    name: String,
    before: Machine,
}

fn parse_hex16(item: &str) -> Result<u16, String> {
    u16::from_str_radix(item, 16).map_err(|_| format!("bad hex word {item:?}"))
}

fn parse_registers(line: &str) -> Result<[u16; 13], String> {
    let values: Vec<u16> = line
        .split_whitespace()
        .map(parse_hex16)
        .collect::<Result<_, _>>()?;
    values
        .try_into()
        .map_err(|_| format!("expected 13 registers in {line:?}"))
}

/// `I R IFF1 IFF2 IM halted tstates`
fn parse_misc(line: &str, registers: [u16; 13], memory: Vec<(u16, u8)>) -> Result<Machine, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 7 {
        return Err(format!("expected 7 fields in {line:?}"));
    }
    let number = |item: &str| {
        item.parse::<u64>()
            .map_err(|_| format!("bad number {item:?}"))
    };
    Ok(Machine {
        registers,
        i: parse_hex16(parts[0])? as u8,
        r: parse_hex16(parts[1])? as u8,
        iff1: number(parts[2])? != 0,
        iff2: number(parts[3])? != 0,
        im: number(parts[4])? as u8,
        halted: number(parts[5])? != 0,
        t_states: number(parts[6])?,
        memory,
    })
}

/// `address byte byte ... -1` lines up to a lone `-1` (or a blank line).
fn parse_memory(lines: &[&str], index: &mut usize) -> Result<Vec<(u16, u8)>, String> {
    let mut memory = Vec::new();
    while *index < lines.len() {
        let line = lines[*index].trim();
        if line.is_empty() || line == "-1" {
            break;
        }
        let mut parts = line.split_whitespace();
        let mut address = parse_hex16(parts.next().unwrap_or(""))?;
        for item in parts {
            if item == "-1" {
                break;
            }
            memory.push((address, parse_hex16(item)? as u8));
            address = address.wrapping_add(1);
        }
        *index += 1;
    }
    Ok(memory)
}

fn parse_tests_in(text: &str) -> Result<Vec<Case>, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut cases = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if lines[index].trim().is_empty() {
            index += 1;
            continue;
        }
        let name = lines[index].trim().to_string();
        let registers = parse_registers(lines[index + 1])?;
        let misc = lines[index + 2];
        index += 3;
        let memory = parse_memory(&lines, &mut index)?;
        index += 1; // the lone -1
        cases.push(Case {
            name,
            before: parse_misc(misc, registers, memory)?,
        });
    }
    Ok(cases)
}

fn parse_tests_expected(text: &str) -> Result<Vec<(String, Machine)>, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut expected = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if lines[index].trim().is_empty() {
            index += 1;
            continue;
        }
        let name = lines[index].trim().to_string();
        index += 1;
        while index < lines.len() && lines[index].starts_with(' ') {
            index += 1; // bus events: not compared
        }
        let registers = parse_registers(lines[index])?;
        let misc = lines[index + 1];
        index += 2;
        let memory = parse_memory(&lines, &mut index)?;
        expected.push((name, parse_misc(misc, registers, memory)?));
    }
    Ok(expected)
}

/// coretest.c's machine: patterned RAM, ports that read back their high byte.
struct FuseHost {
    memory: Box<[u8; 0x10000]>,
}

impl Bus for FuseHost {
    fn read_byte(&mut self, addr: u16) -> u8 {
        self.memory[usize::from(addr)]
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        self.memory[usize::from(addr)] = value;
    }

    fn read_port(&mut self, addr: u16) -> u8 {
        (addr >> 8) as u8
    }

    fn write_port(&mut self, _addr: u16, _value: u8) {}
}

fn run_case(case: &Case, expected: &Machine) -> Result<Vec<String>, String> {
    let mut memory = Box::new([0u8; 0x10000]);
    for (index, byte) in memory.iter_mut().enumerate() {
        *byte = [0xDE, 0xAD, 0xBE, 0xEF][index % 4];
    }
    for &(address, value) in &case.before.memory {
        memory[usize::from(address)] = value;
    }
    let mut cpu = Z80::new(FuseHost { memory });
    let before = &case.before;
    let [af, bc, de, hl, af_, bc_, de_, hl_, ix, iy, sp, pc, wz] = before.registers;
    cpu.a = (af >> 8) as u8;
    cpu.f.set_byte(af as u8);
    cpu.b = (bc >> 8) as u8;
    cpu.c = bc as u8;
    cpu.d = (de >> 8) as u8;
    cpu.e = de as u8;
    cpu.h = (hl >> 8) as u8;
    cpu.l = hl as u8;
    cpu.af_ = af_;
    cpu.bc_ = bc_;
    cpu.de_ = de_;
    cpu.hl_ = hl_;
    cpu.ix = ix;
    cpu.iy = iy;
    cpu.sp = sp;
    cpu.pc = pc;
    cpu.wz = wz;
    cpu.i = before.i;
    cpu.r = before.r;
    cpu.im = before.im;
    cpu.iff1 = before.iff1;
    cpu.iff2 = before.iff2;
    cpu.halted = before.halted;

    let mut total: u64 = 0;
    while total < before.t_states {
        total += u64::from(
            cpu.step()
                .map_err(|fault| format!("{}: {fault}", case.name))?,
        );
    }

    let actual: [u16; 13] = [
        (u16::from(cpu.a) << 8) | u16::from(cpu.f.byte()),
        (u16::from(cpu.b) << 8) | u16::from(cpu.c),
        (u16::from(cpu.d) << 8) | u16::from(cpu.e),
        (u16::from(cpu.h) << 8) | u16::from(cpu.l),
        cpu.af_,
        cpu.bc_,
        cpu.de_,
        cpu.hl_,
        cpu.ix,
        cpu.iy,
        cpu.sp,
        cpu.pc,
        cpu.wz,
    ];
    let mut differences = Vec::new();
    for (name, (got, want)) in REGISTER_NAMES
        .iter()
        .zip(actual.iter().zip(&expected.registers))
    {
        if got != want {
            differences.push(format!("{name}: got {got:04X}, expected {want:04X}"));
        }
    }
    let scalars = [
        ("I", u64::from(cpu.i), u64::from(expected.i)),
        ("R", u64::from(cpu.r), u64::from(expected.r)),
        ("IFF1", u64::from(cpu.iff1), u64::from(expected.iff1)),
        ("IFF2", u64::from(cpu.iff2), u64::from(expected.iff2)),
        ("IM", u64::from(cpu.im), u64::from(expected.im)),
        ("halted", u64::from(cpu.halted), u64::from(expected.halted)),
        ("T-states", total, expected.t_states),
    ];
    for (name, got, want) in scalars {
        if got != want {
            differences.push(format!("{name}: got {got}, expected {want}"));
        }
    }
    for &(address, value) in &expected.memory {
        let got = cpu.bus.memory[usize::from(address)];
        if got != value {
            differences.push(format!(
                "memory[{address:04X}]: got {got:02X}, expected {value:02X}"
            ));
        }
    }
    Ok(differences)
}

fn main() -> ExitCode {
    let Some(dir) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: z80-fuse <dir holding tests.in and tests.expected>");
        return ExitCode::from(2);
    };
    let read = |name: &str| {
        std::fs::read_to_string(dir.join(name))
            .map_err(|error| format!("{}: {error}", dir.join(name).display()))
    };
    let suite = read("tests.in").and_then(|text| parse_tests_in(&text));
    let expected = read("tests.expected").and_then(|text| parse_tests_expected(&text));
    let (cases, expected) = match (suite, expected) {
        (Ok(cases), Ok(expected)) => (cases, expected),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };
    if cases.len() != EXPECTED_CASES || expected.len() != EXPECTED_CASES {
        eprintln!(
            "error: expected {EXPECTED_CASES} cases (FUSE 1.6.0), found {} in tests.in and {} in tests.expected",
            cases.len(),
            expected.len()
        );
        return ExitCode::from(2);
    }
    let started = std::time::Instant::now();
    let (mut agreed, mut expected_divergences, mut unexpected) = (0usize, 0usize, 0usize);
    for (case, (expected_name, expected)) in cases.iter().zip(&expected) {
        if case.name != *expected_name {
            eprintln!(
                "error: tests.in and tests.expected disagree on order at {}",
                case.name
            );
            return ExitCode::from(2);
        }
        let differences = match run_case(case, expected) {
            Ok(differences) => differences,
            Err(error) => {
                eprintln!("error: {error}");
                return ExitCode::from(2);
            }
        };
        let known = KNOWN_DIVERGENCES
            .iter()
            .find(|(name, _)| *name == case.name);
        match (differences.is_empty(), known) {
            (true, None) => agreed += 1,
            (false, Some((_, reason))) => {
                expected_divergences += 1;
                println!("expected divergence {}: {reason}", case.name);
            }
            (true, Some(_)) => {
                unexpected += 1;
                println!(
                    "UNEXPECTED AGREEMENT {}: FUSE and this core now agree; review the known list",
                    case.name
                );
            }
            (false, None) => {
                unexpected += 1;
                println!("DIVERGENCE {}: {}", case.name, differences.join("; "));
            }
        }
    }
    println!(
        "FUSE 1.6.0: {agreed} agree, {expected_divergences} expected divergences, {unexpected} unexpected, of {} cases ({:.2} s)",
        cases.len(),
        started.elapsed().as_secs_f64()
    );
    if unexpected == 0 && expected_divergences == KNOWN_DIVERGENCES.len() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
