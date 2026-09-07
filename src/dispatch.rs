//! Top-level, base, CB, and ED opcode dispatch.
//!
//! Transcribed from `z80_python/_dispatch.py`. The explicit if-chain shape is
//! deliberate and mirrors the reference: every opcode is one grep away.

use crate::core::{Bus, Fault, Z80};

impl<B: Bus> Z80<B> {
    /// Fetch and execute one instruction, returning its documented T-states.
    ///
    /// This is the instruction-only entry point; [`Z80::step`] is the host
    /// entry point that services lifecycle requests first.
    pub fn decode_and_execute(&mut self) -> Result<u32, Fault> {
        self.fetched.clear();
        let opcode = self.fetch_byte();
        if opcode == 0xCB {
            let sub_opcode = self.fetch_byte();
            return Ok(self.execute_cb(sub_opcode));
        }
        if opcode == 0xED {
            let sub_opcode = self.fetch_byte();
            return Ok(self.execute_ed(sub_opcode));
        }
        if matches!(opcode, 0xDD | 0xFD) {
            let sub_opcode = self.fetch_byte();
            return self.execute_index(opcode, sub_opcode);
        }
        self.execute_main(opcode)
    }

    pub(crate) fn execute_cb(&mut self, sub_opcode: u8) -> u32 {
        if sub_opcode <= 0x3F {
            return self.op_rot(sub_opcode);
        }
        if sub_opcode <= 0x7F {
            return self.op_bit(sub_opcode);
        }
        if sub_opcode <= 0xBF {
            return self.op_res(sub_opcode);
        }
        self.op_set(sub_opcode)
    }

    pub(crate) fn execute_main(&mut self, opcode: u8) -> Result<u32, Fault> {
        if opcode == 0x00 {
            return Ok(self.op_nop());
        }
        if matches!(opcode, 0x01 | 0x11 | 0x21 | 0x31) {
            return Ok(self.op_ld_rr_nn(opcode));
        }
        if opcode == 0x08 {
            return Ok(self.op_ex_af_af());
        }
        if opcode == 0x10 {
            return Ok(self.op_djnz());
        }
        if opcode == 0x76 {
            return Ok(self.op_halt());
        }
        if (0x40..=0x7F).contains(&opcode) && opcode != 0x76 {
            return Ok(self.op_ld_r_r(opcode));
        }
        if matches!(
            opcode,
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E
        ) {
            return Ok(self.op_ld_r_n(opcode));
        }
        if matches!(opcode, 0x0A | 0x1A) {
            let reg16 = if opcode == 0x0A { self.bc() } else { self.de() };
            return Ok(self.op_ld_a_irr(reg16));
        }
        if matches!(opcode, 0x02 | 0x12) {
            let reg16 = if opcode == 0x02 { self.bc() } else { self.de() };
            return Ok(self.op_ld_irr_a(reg16));
        }
        if opcode == 0x3A {
            return Ok(self.op_ld_a_inn());
        }
        if opcode == 0x32 {
            return Ok(self.op_ld_inn_a());
        }
        if matches!(opcode, 0x37 | 0x3F) {
            return Ok(self.op_scf_ccf(opcode, false));
        }
        if opcode == 0x22 {
            return Ok(self.op_ld_nn_hl());
        }
        if opcode == 0x2A {
            return Ok(self.op_ld_hl_nn_from_mem());
        }
        if opcode == 0xDB {
            return Ok(self.op_in_a_n());
        }
        if opcode == 0xD3 {
            return Ok(self.op_out_n_a());
        }
        if (0x80..=0xBF).contains(&opcode) {
            return Ok(self.op_alu_r(opcode));
        }
        if matches!(
            opcode,
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE
        ) {
            return Ok(self.op_alu_n(opcode));
        }
        if matches!(opcode, 0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x3C) {
            return Ok(self.op_inc_r(opcode));
        }
        if matches!(opcode, 0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x3D) {
            return Ok(self.op_dec_r(opcode));
        }
        if opcode == 0x34 {
            return Ok(self.op_inc_hl());
        }
        if opcode == 0x35 {
            return Ok(self.op_dec_hl());
        }
        if opcode == 0x27 {
            return Ok(self.op_daa());
        }
        if opcode == 0x2F {
            return Ok(self.op_cpl());
        }
        if opcode == 0x07 {
            return Ok(self.op_rlca());
        }
        if opcode == 0x0F {
            return Ok(self.op_rrca());
        }
        if opcode == 0x17 {
            return Ok(self.op_rla());
        }
        if opcode == 0x1F {
            return Ok(self.op_rra());
        }
        if matches!(opcode, 0x09 | 0x19 | 0x29 | 0x39) {
            return Ok(self.op_add_hl_rr(opcode));
        }
        if matches!(opcode, 0x03 | 0x13 | 0x23 | 0x33) {
            return Ok(self.op_inc_rr(opcode));
        }
        if matches!(opcode, 0x0B | 0x1B | 0x2B | 0x3B) {
            return Ok(self.op_dec_rr(opcode));
        }
        if matches!(opcode, 0x18 | 0x20 | 0x28 | 0x30 | 0x38) {
            return Ok(self.op_jr(opcode));
        }
        if matches!(
            opcode,
            0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8
        ) {
            return Ok(self.op_ret_cc(opcode));
        }
        if opcode == 0xC9 {
            return Ok(self.op_ret());
        }
        if matches!(
            opcode,
            0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA | 0xC3
        ) {
            return Ok(self.op_jp(opcode));
        }
        if matches!(
            opcode,
            0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC | 0xCD
        ) {
            return Ok(self.op_call(opcode));
        }
        if matches!(
            opcode,
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF
        ) {
            return Ok(self.op_rst(opcode));
        }
        if matches!(opcode, 0xC1 | 0xD1 | 0xE1) {
            return Ok(self.op_pop_rr(opcode));
        }
        if matches!(opcode, 0xC5 | 0xD5 | 0xE5) {
            return Ok(self.op_push_rr(opcode));
        }
        if opcode == 0xF1 {
            return Ok(self.op_pop_af());
        }
        if opcode == 0xF5 {
            return Ok(self.op_push_af());
        }
        if opcode == 0xF9 {
            return Ok(self.op_ld_sp_hl());
        }
        if opcode == 0xD9 {
            return Ok(self.op_exx());
        }
        if opcode == 0xE3 {
            return Ok(self.op_ex_sp_hl());
        }
        if opcode == 0xE9 {
            return Ok(self.op_jp_hl());
        }
        if opcode == 0xEB {
            return Ok(self.op_ex_de_hl());
        }
        if opcode == 0xF3 {
            return Ok(self.op_interrupt_enable(false));
        }
        if opcode == 0xFB {
            return Ok(self.op_interrupt_enable(true));
        }
        Err(Fault::UnhandledOpcode {
            opcode,
            pc: self.pc.wrapping_sub(1),
        })
    }

    pub(crate) fn execute_ed(&mut self, opcode: u8) -> u32 {
        // Explicit dispatch keeps each opcode directly traceable.
        match opcode {
            0x47 => return self.op_ld_i_a(),
            0x4F => return self.op_ld_r_a(),
            0x57 => return self.op_ld_a_i(),
            0x5F => return self.op_ld_a_r(),
            0x67 => return self.op_rrd(),
            0x6F => return self.op_rld(),
            0x77 | 0x7F => return self.op_ed_nop(),
            0xA0 => return self.op_ldi(),
            0xA1 => return self.op_cpi(),
            0xA2 => return self.op_ini(),
            0xA3 => return self.op_outi(),
            0xA8 => return self.op_ldd(),
            0xA9 => return self.op_cpd(),
            0xAA => return self.op_ind(),
            0xAB => return self.op_outd(),
            0xB0 => return self.op_ldir(),
            0xB1 => return self.op_cpir(),
            0xB2 => return self.op_inir(),
            0xB3 => return self.op_otir(),
            0xB8 => return self.op_lddr(),
            0xB9 => return self.op_cpdr(),
            0xBA => return self.op_indr(),
            0xBB => return self.op_otdr(),
            _ => {}
        }
        if matches!(opcode, 0x45 | 0x55 | 0x65 | 0x75) {
            return self.op_retn();
        }
        if matches!(opcode, 0x4D | 0x5D | 0x6D | 0x7D) {
            return self.op_reti();
        }
        if matches!(opcode, 0x4A | 0x5A | 0x6A | 0x7A) {
            return self.op_adc_hl_rr(opcode);
        }
        if matches!(opcode, 0x42 | 0x52 | 0x62 | 0x72) {
            return self.op_sbc_hl_rr(opcode);
        }
        if matches!(opcode, 0x43 | 0x53 | 0x63 | 0x73) {
            return self.op_ld_nn_rr((opcode >> 4) & 0x03);
        }
        if matches!(opcode, 0x4B | 0x5B | 0x6B | 0x7B) {
            return self.op_ld_rr_nn_from_mem((opcode >> 4) & 0x03);
        }
        if matches!(opcode, 0x46 | 0x4E | 0x66 | 0x6E) {
            return self.op_im(0);
        }
        if matches!(opcode, 0x56 | 0x76) {
            return self.op_im(1);
        }
        if matches!(opcode, 0x5E | 0x7E) {
            return self.op_im(2);
        }
        if matches!(
            opcode,
            0x44 | 0x4C | 0x54 | 0x5C | 0x64 | 0x6C | 0x74 | 0x7C
        ) {
            return self.op_neg();
        }
        if (0x40..=0x78).contains(&opcode) && (opcode & 0x07) == 0 {
            return self.op_in_r_c(opcode);
        }
        if (0x41..=0x79).contains(&opcode) && (opcode & 0x07) == 1 {
            return self.op_out_c_r(opcode);
        }
        // Every ED-prefixed byte not otherwise defined is a genuine Z80 instruction
        // on real silicon: a 2-byte, 8 T-state no-op. Only 0x77/0x7F fell inside the
        // documented 0x40-0x7F block; the rest of the ED space needs the same rule.
        self.op_ed_nop()
    }
}
