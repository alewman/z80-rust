//! Block transfer and search instruction implementation.
//!
//! Transcribed from `z80_python/_blocks.py`.

use crate::core::{Bus, Z80};

impl<B: Bus> Z80<B> {
    pub(crate) fn block_ld(&mut self, increment: bool) -> bool {
        let src = self.hl();
        let data = self.bus.read_byte(src);
        let dst = self.de();
        self.bus.write_byte(dst, data);
        let (hl, de) = if increment {
            (src.wrapping_add(1), dst.wrapping_add(1))
        } else {
            (src.wrapping_sub(1), dst.wrapping_sub(1))
        };
        self.h = (hl >> 8) as u8;
        self.l = hl as u8;
        self.d = (de >> 8) as u8;
        self.e = de as u8;
        self.f.set_n(0);
        self.f.set_h(0);
        let bc = self.bc().wrapping_sub(1);
        self.f.set_pv(if bc != 0 { 1 } else { 0 });
        self.b = (bc >> 8) as u8;
        self.c = bc as u8;
        // LD group: X/Y come from A + transferred byte, landing in bits 3 and 1
        // (not 3 and 5). Y here is bit 1 of that sum, the documented-undocumented rule.
        let sum = self.a.wrapping_add(data);
        self.f.set_x((sum & 0x08) >> 3);
        self.f.set_y((sum & 0x02) >> 1);
        bc != 0
    }

    pub(crate) fn block_cp(&mut self, increment: bool) -> bool {
        let src = self.hl();
        let data = self.bus.read_byte(src);
        let hl = if increment {
            self.wz = self.wz.wrapping_add(1);
            src.wrapping_add(1)
        } else {
            self.wz = self.wz.wrapping_sub(1);
            src.wrapping_sub(1)
        };
        self.h = (hl >> 8) as u8;
        self.l = hl as u8;
        let result = self.a.wrapping_sub(data);
        self.f.set_n(1);
        let bc = self.bc().wrapping_sub(1);
        self.f.set_pv(if bc != 0 { 1 } else { 0 });
        self.b = (bc >> 8) as u8;
        self.c = bc as u8;
        self.f.set_h(((self.a ^ data ^ result) & 0x10) >> 4);
        // X/Y come from A - (HL) - H, the ALU's intermediate before the final
        // correction, and land in bits 3 and 1 (not 3 and 5) for the CP group.
        let intermediate = result.wrapping_sub(self.f.h());
        self.f.set_x((intermediate & 0x08) >> 3);
        self.f.set_y((intermediate & 0x02) >> 1);
        self.set_sz(result);
        bc != 0 && self.f.z() == 0
    }

    pub(crate) fn block_repeat(&mut self) {
        self.pc = self.pc.wrapping_sub(2);
        self.wz = self.pc.wrapping_add(1);
        // On a repeat the CPU rewinds PC by 2 and re-fetches; X/Y then sample bits 3
        // and 5 of the rewound PC's high byte (PC bits 11 and 13), and WZ = PC + 1.
        self.f.set_x(((self.pc >> 11) & 1) as u8);
        self.f.set_y(((self.pc >> 13) & 1) as u8);
    }

    /// LDI -- (DE) <- (HL); HL++, DE++, BC--.
    pub(crate) fn op_ldi(&mut self) -> u32 {
        self.block_ld(true);
        self.update_q(true);
        16
    }

    /// LDD -- (DE) <- (HL); HL--, DE--, BC--.
    pub(crate) fn op_ldd(&mut self) -> u32 {
        self.block_ld(false);
        self.update_q(true);
        16
    }

    /// CPI -- compare A with (HL); HL++, BC--.
    pub(crate) fn op_cpi(&mut self) -> u32 {
        self.block_cp(true);
        self.update_q(true);
        16
    }

    /// CPD -- compare A with (HL); HL--, BC--.
    pub(crate) fn op_cpd(&mut self) -> u32 {
        self.block_cp(false);
        self.update_q(true);
        16
    }

    /// LDIR -- LDI repeated while BC != 0; 21 T-states per repeat, 16 on the last.
    pub(crate) fn op_ldir(&mut self) -> u32 {
        let repeat = self.block_ld(true);
        if repeat {
            self.block_repeat();
        }
        self.update_q(true);
        if repeat {
            21
        } else {
            16
        }
    }

    /// LDDR -- LDD repeated while BC != 0; 21 T-states per repeat, 16 on the last.
    pub(crate) fn op_lddr(&mut self) -> u32 {
        let repeat = self.block_ld(false);
        if repeat {
            self.block_repeat();
        }
        self.update_q(true);
        if repeat {
            21
        } else {
            16
        }
    }

    /// CPIR -- CPI repeated while BC != 0 and A != (HL); 21 T-states per repeat, 16 last.
    pub(crate) fn op_cpir(&mut self) -> u32 {
        let repeat = self.block_cp(true);
        if repeat {
            self.block_repeat();
        }
        self.update_q(true);
        if repeat {
            21
        } else {
            16
        }
    }

    /// CPDR -- CPD repeated while BC != 0 and A != (HL); 21 T-states per repeat, 16 last.
    pub(crate) fn op_cpdr(&mut self) -> u32 {
        let repeat = self.block_cp(false);
        if repeat {
            self.block_repeat();
        }
        self.update_q(true);
        if repeat {
            21
        } else {
            16
        }
    }
}
