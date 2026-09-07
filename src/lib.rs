//! A Rust Z80 instruction core transcribed from `z80-python`.
//!
//! The module layout mirrors the reference so the two can be read side by
//! side: `flags`, `state`, `core`, `alu`, `blocks`, `control`, `dispatch`,
//! `index`, `index_dispatch`, `io`, `loads`, `rotate`, and `cpu` correspond
//! one-to-one with `z80_python/_flags.py`, `state.py`, `_core.py`, and so on.

pub mod alu;
pub mod blocks;
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

pub use crate::core::{Bus, Fault, Z80};
pub use crate::flags::Flags;
pub use crate::state::CpuState;
