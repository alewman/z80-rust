//! Public Z80 CPU API: `step`, state capture/restore, and the lifecycle requests.
//!
//! Transcribed from `z80_python/cpu.py`.

use crate::core::{Bus, Fault, Z80};
use crate::state::CpuState;

impl<B: Bus> Z80<B> {
    /// Advance one instruction boundary and return its documented T-state count.
    ///
    /// An asserted RESET takes priority over NMI and an accepted maskable
    /// interrupt; each is serviced before instruction fetch. A halted CPU
    /// consumes a four-T-state idle cycle until an accepted interrupt wakes
    /// it. T-states are instruction/lifecycle totals, not externally
    /// observable bus cycles.
    pub fn step(&mut self) -> Result<u32, Fault> {
        if self.reset_pending {
            return Ok(self.accept_reset());
        }
        if self.non_maskable_interrupt_pending {
            return Ok(self.accept_non_maskable_interrupt());
        }
        if self.can_accept_maskable_interrupt() {
            return self.accept_maskable_interrupt();
        }

        let delay_was_active = self.ei_delay > 0;
        let t_states = if self.halted {
            self.inc_r();
            self.update_q(false);
            4
        } else {
            self.decode_and_execute()?
        };
        if delay_was_active {
            self.ei_delay -= 1;
        }
        Ok(t_states)
    }

    /// Return a snapshot of all CPU-owned execution state.
    ///
    /// Host memory, ports, devices, scheduling, and counters are not included.
    pub fn capture_state(&self) -> CpuState {
        CpuState {
            a: self.a,
            f: self.f.byte(),
            b: self.b,
            c: self.c,
            d: self.d,
            e: self.e,
            h: self.h,
            l: self.l,
            ix: self.ix,
            iy: self.iy,
            sp: self.sp,
            pc: self.pc,
            wz: self.wz,
            i: self.i,
            r: self.r,
            iff1: self.iff1,
            iff2: self.iff2,
            im: self.im,
            af_alt: self.af_,
            bc_alt: self.bc_,
            de_alt: self.de_,
            hl_alt: self.hl_,
            q: self.q,
            halted: self.halted,
            ei_delay: self.ei_delay,
            reset_pending: self.reset_pending,
            maskable_interrupt_vector: self.pending_maskable_interrupt,
            non_maskable_interrupt_pending: self.non_maskable_interrupt_pending,
        }
    }

    /// Restore a previously captured CPU state without touching the host.
    pub fn restore_state(&mut self, state: &CpuState) {
        self.a = state.a;
        self.f.set_byte(state.f);
        self.b = state.b;
        self.c = state.c;
        self.d = state.d;
        self.e = state.e;
        self.h = state.h;
        self.l = state.l;
        self.ix = state.ix;
        self.iy = state.iy;
        self.sp = state.sp;
        self.pc = state.pc;
        self.wz = state.wz;
        self.i = state.i;
        self.r = state.r;
        self.iff1 = state.iff1;
        self.iff2 = state.iff2;
        self.im = state.im;
        self.af_ = state.af_alt;
        self.bc_ = state.bc_alt;
        self.de_ = state.de_alt;
        self.hl_ = state.hl_alt;
        self.q = state.q;
        self.halted = state.halted;
        self.ei_delay = state.ei_delay;
        self.reset_pending = state.reset_pending;
        self.pending_maskable_interrupt = state.maskable_interrupt_vector;
        self.non_maskable_interrupt_pending = state.non_maskable_interrupt_pending;
    }

    /// Whether the host has asserted RESET.
    pub fn reset_pending(&self) -> bool {
        self.reset_pending
    }

    /// Assert RESET for servicing at the next instruction boundary.
    pub fn request_reset(&mut self) {
        self.reset_pending = true;
    }

    /// Release the host-controlled RESET line.
    pub fn clear_reset(&mut self) {
        self.reset_pending = false;
    }

    /// Whether a device has requested a maskable interrupt not yet accepted.
    pub fn maskable_interrupt_pending(&self) -> bool {
        self.pending_maskable_interrupt.is_some()
    }

    /// Assert the maskable-interrupt request line between instruction boundaries.
    pub fn request_maskable_interrupt(&mut self, vector_byte: u8) {
        self.pending_maskable_interrupt = Some(vector_byte);
    }

    /// Deassert a previously requested but not-yet-accepted interrupt.
    pub fn clear_maskable_interrupt(&mut self) {
        self.pending_maskable_interrupt = None;
    }

    /// Whether a device has requested a non-maskable interrupt not yet accepted.
    pub fn non_maskable_interrupt_pending(&self) -> bool {
        self.non_maskable_interrupt_pending
    }

    /// Latch an NMI request for service at the next instruction boundary.
    pub fn request_non_maskable_interrupt(&mut self) {
        self.non_maskable_interrupt_pending = true;
    }

    /// Cancel a requested NMI that has not yet reached an instruction boundary.
    pub fn clear_non_maskable_interrupt(&mut self) {
        self.non_maskable_interrupt_pending = false;
    }
}
