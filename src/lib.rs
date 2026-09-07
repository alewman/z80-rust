//! A Rust Z80 instruction core transcribed from `z80-python`.
//!
//! The module layout mirrors the reference so the two can be read side by
//! side: `flags`, `state`, `core`, `alu`, `blocks`, `control`, `dispatch`,
//! `index`, `index_dispatch`, `io`, `loads`, `rotate`, and `cpu` correspond
//! one-to-one with `z80_python/_flags.py`, `state.py`, `_core.py`, and so on.
//! `trace` and `conformance` implement the trace schema and the conformance
//! kit (`docs/trace-schema.md` and `docs/conformance.md` in z80-python).

pub mod alu;
pub mod blocks;
pub mod conformance;
pub mod control;
pub mod core;
pub mod cpu;
pub mod dispatch;
pub mod flags;
pub mod index;
pub mod index_dispatch;
pub mod io;
pub mod loads;
pub mod rotate;
pub mod state;
pub mod trace;

pub use crate::core::{Bus, Fault, Z80};
pub use crate::flags::Flags;
pub use crate::state::CpuState;
