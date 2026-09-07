//! Trace records, one per processor boundary, in the version 1 JSON Lines
//! schema (`docs/trace-schema.md` in z80-python).
//!
//! A record carries the boundary kind, the T-states it consumed, the
//! instruction's address and bytes (no mnemonic: the reference disassembles
//! the bytes itself), and the complete state before and after.

use std::fmt::Write as _;
use std::io;

use crate::core::{Bus, Fault, Z80};
use crate::state::CpuState;

pub const TRACE_SCHEMA_VERSION: u32 = 1;

/// Kind of work performed by one `step()` call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryKind {
    Instruction,
    HaltIdle,
    Reset,
    NonMaskableInterrupt,
    MaskableInterrupt,
}

impl BoundaryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            BoundaryKind::Instruction => "instruction",
            BoundaryKind::HaltIdle => "halt_idle",
            BoundaryKind::Reset => "reset",
            BoundaryKind::NonMaskableInterrupt => "non_maskable_interrupt",
            BoundaryKind::MaskableInterrupt => "maskable_interrupt",
        }
    }
}

/// Classify the next boundary from the state and lifecycle requests, in the
/// order the schema fixes: reset, NMI, acceptable maskable, halted, else
/// instruction.
pub fn boundary_kind(state: &CpuState) -> BoundaryKind {
    if state.reset_pending {
        return BoundaryKind::Reset;
    }
    if state.non_maskable_interrupt_pending {
        return BoundaryKind::NonMaskableInterrupt;
    }
    if state.maskable_interrupt_vector.is_some() && state.iff1 && state.ei_delay == 0 {
        return BoundaryKind::MaskableInterrupt;
    }
    if state.halted {
        return BoundaryKind::HaltIdle;
    }
    BoundaryKind::Instruction
}

/// The bytes one instruction occupied, prefixes and operands included.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionBytes {
    pub address: u16,
    pub data: Vec<u8>,
}

/// Before/after evidence for one processor boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepRecord {
    pub sequence: u64,
    pub kind: BoundaryKind,
    pub t_states: u32,
    pub instruction: Option<InstructionBytes>,
    pub before: CpuState,
    pub after: CpuState,
}

impl StepRecord {
    /// Encode as one JSON object with sorted keys and no whitespace, the
    /// form `z80_python.trace.write_trace` produces, without a newline.
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("{\"after\":");
        self.after.write_json(&mut out);
        out.push_str(",\"before\":");
        self.before.write_json(&mut out);
        out.push_str(",\"instruction\":");
        match &self.instruction {
            None => out.push_str("null"),
            Some(instruction) => {
                let _ = write!(out, "{{\"address\":{},\"data\":\"", instruction.address);
                for byte in &instruction.data {
                    let _ = write!(out, "{byte:02x}");
                }
                out.push_str("\"}");
            }
        }
        let _ = write!(
            out,
            ",\"kind\":\"{}\",\"sequence\":{},\"t_states\":{},\"version\":{}}}",
            self.kind.as_str(),
            self.sequence,
            self.t_states,
            TRACE_SCHEMA_VERSION
        );
        out
    }

    /// Write the record as one JSON Lines record.
    pub fn write_to(&self, out: &mut impl io::Write) -> io::Result<()> {
        out.write_all(self.to_json().as_bytes())?;
        out.write_all(b"\n")
    }
}

/// Advance `cpu` one boundary and return its record.
///
/// On a [`Fault`] the CPU is left as the reference leaves it and no record
/// is produced, because the reference produces none either.
pub fn step_record<B: Bus>(cpu: &mut Z80<B>, sequence: u64) -> Result<StepRecord, Fault> {
    let before = cpu.capture_state();
    let kind = boundary_kind(&before);
    let t_states = cpu.step()?;
    let after = cpu.capture_state();
    let instruction = if kind == BoundaryKind::Instruction {
        Some(InstructionBytes {
            address: before.pc,
            data: cpu.last_instruction_bytes().to_vec(),
        })
    } else {
        None
    };
    Ok(StepRecord {
        sequence,
        kind,
        t_states,
        instruction,
        before,
        after,
    })
}
