//! DD/FD index-prefix dispatcher.
//!
//! Transcribed from `z80_python/_index_dispatch.py`. A DD or FD prefix is its
//! own M1 fetch on real hardware: it costs 4 T-states and bumps R before the
//! opcode after it is fetched. Base-set opcodes that the prefix does not
//! modify therefore run their ordinary handler and add that 4 here; the
//! handlers that take a `prefix` argument already include the prefix cost in
//! their documented totals.

use crate::core::{Bus, Fault, Z80};

impl<B: Bus> Z80<B> {
    pub(crate) fn execute_index(&mut self, prefix: u8, sub_opcode: u8) -> Result<u32, Fault> {
        if sub_opcode == 0xCB {
            let displacement = self.read_operand_byte();
            // The final DDCB opcode byte is read as an operand, not an M1 fetch, so R
            // advances only for the DD and CB prefixes.
            let cb_opcode = self.read_operand_byte();
            if cb_opcode <= 0x3F {
                return Ok(self.op_index_rot(prefix, displacement, cb_opcode));
            }
            if (0x40..=0x7F).contains(&cb_opcode) {
                return Ok(self.op_index_bit(prefix, displacement, cb_opcode));
            }
            if (0x80..=0xBF).contains(&cb_opcode) {
                return Ok(self.op_index_res_set(prefix, displacement, cb_opcode, false));
            }
            return Ok(self.op_index_res_set(prefix, displacement, cb_opcode, true));
        }
        if sub_opcode == 0x76 {
            return Ok(self.op_halt() + 4);
        }
        if sub_opcode == 0x00 {
            return Ok(self.op_nop() + 4);
        }
        if matches!(sub_opcode, 0x01 | 0x11 | 0x31) {
            return Ok(self.op_ld_rr_nn(sub_opcode) + 4);
        }
        if sub_opcode == 0x08 {
            return Ok(self.op_ex_af_af() + 4);
        }
        if sub_opcode == 0x10 {
            return Ok(self.op_djnz() + 4);
        }
        if matches!(
            sub_opcode,
            0x40 | 0x41
                | 0x42
                | 0x43
                | 0x47
                | 0x48
                | 0x49
                | 0x4A
                | 0x4B
                | 0x4F
                | 0x50
                | 0x51
                | 0x52
                | 0x53
                | 0x57
                | 0x58
                | 0x59
                | 0x5A
                | 0x5B
                | 0x5F
                | 0x78
                | 0x79
                | 0x7A
                | 0x7B
                | 0x7F
        ) {
            return Ok(self.op_ld_r_r(sub_opcode) + 4);
        }
        if sub_opcode == 0x7C {
            return Ok(self.op_ld_a_index_h(prefix) + 4);
        }
        if sub_opcode == 0x7D {
            return Ok(self.op_ld_a_index_l(prefix) + 4);
        }
        if matches!(
            sub_opcode,
            0x44 | 0x45 | 0x4C | 0x4D | 0x54 | 0x55 | 0x5C | 0x5D
        ) {
            return Ok(self.op_ld_r_index_byte(prefix, sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x64 | 0x65 | 0x6C | 0x6D) {
            return Ok(self.op_ld_index_byte_index_byte(prefix, sub_opcode) + 4);
        }
        if matches!(
            sub_opcode,
            0x60 | 0x61 | 0x62 | 0x63 | 0x67 | 0x68 | 0x69 | 0x6A | 0x6B | 0x6F
        ) {
            return Ok(self.op_ld_index_byte_r(prefix, sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x26 | 0x2E) {
            return Ok(self.op_ld_index_byte_n(prefix, sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x24 | 0x25 | 0x2C | 0x2D) {
            return Ok(self.op_inc_dec_index_byte(prefix, sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x04 | 0x0C | 0x14 | 0x1C | 0x3C) {
            return Ok(self.op_inc_r(sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x05 | 0x0D | 0x15 | 0x1D | 0x3D) {
            return Ok(self.op_dec_r(sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x03 | 0x13 | 0x33) {
            return Ok(self.op_inc_rr(sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x0B | 0x1B | 0x3B) {
            return Ok(self.op_dec_rr(sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x06 | 0x0E | 0x16 | 0x1E | 0x3E) {
            return Ok(self.op_ld_r_n(sub_opcode) + 4);
        }
        if matches!(sub_opcode, 0x0A | 0x1A) {
            let reg16 = if sub_opcode == 0x0A {
                self.bc()
            } else {
                self.de()
            };
            return Ok(self.op_ld_a_irr(reg16) + 4);
        }
        if matches!(sub_opcode, 0x02 | 0x12) {
            let reg16 = if sub_opcode == 0x02 {
                self.bc()
            } else {
                self.de()
            };
            return Ok(self.op_ld_irr_a(reg16) + 4);
        }
        if sub_opcode == 0x3A {
            return Ok(self.op_ld_a_inn() + 4);
        }
        if sub_opcode == 0x32 {
            return Ok(self.op_ld_inn_a() + 4);
        }
        if sub_opcode == 0xDB {
            return Ok(self.op_in_a_n() + 4);
        }
        if sub_opcode == 0xD3 {
            return Ok(self.op_out_n_a() + 4);
        }
        if matches!(sub_opcode, 0x18 | 0x20 | 0x28 | 0x30 | 0x38) {
            return Ok(self.op_jr(sub_opcode) + 4);
        }
        if sub_opcode == 0x27 {
            return Ok(self.op_daa() + 4);
        }
        if sub_opcode == 0x2F {
            return Ok(self.op_cpl() + 4);
        }
        if matches!(sub_opcode, 0x37 | 0x3F) {
            return Ok(self.op_scf_ccf(sub_opcode, true) + 4);
        }
        if sub_opcode == 0x07 {
            return Ok(self.op_rlca() + 4);
        }
        if sub_opcode == 0x0F {
            return Ok(self.op_rrca() + 4);
        }
        if sub_opcode == 0x17 {
            return Ok(self.op_rla() + 4);
        }
        if sub_opcode == 0x1F {
            return Ok(self.op_rra() + 4);
        }

        // plain_alu: every 8-bit ALU form whose operand is not H, L, or (HL).
        if (0x80..=0xBF).contains(&sub_opcode) && !matches!(sub_opcode & 0x07, 4 | 5 | 6) {
            return Ok(self.op_alu_r(sub_opcode) + 4);
        }
        // indexed_register_alu: the same forms with IXH/IXL (IYH/IYL) operands.
        match sub_opcode {
            0x84 => return Ok(self.op_add_a_index_h(prefix) + 4),
            0x85 => return Ok(self.op_add_a_index_l(prefix) + 4),
            0x8C => return Ok(self.op_adc_a_index_h(prefix) + 4),
            0x8D => return Ok(self.op_adc_a_index_l(prefix) + 4),
            0x94 => return Ok(self.op_sub_a_index_h(prefix) + 4),
            0x95 => return Ok(self.op_sub_a_index_l(prefix) + 4),
            0x9C => return Ok(self.op_sbc_a_index_h(prefix) + 4),
            0x9D => return Ok(self.op_sbc_a_index_l(prefix) + 4),
            0xA4 => return Ok(self.op_and_a_index_h(prefix) + 4),
            0xA5 => return Ok(self.op_and_a_index_l(prefix) + 4),
            0xAC => return Ok(self.op_xor_a_index_h(prefix) + 4),
            0xAD => return Ok(self.op_xor_a_index_l(prefix) + 4),
            0xB4 => return Ok(self.op_or_a_index_h(prefix) + 4),
            0xB5 => return Ok(self.op_or_a_index_l(prefix) + 4),
            0xBC => return Ok(self.op_cp_a_index_h(prefix) + 4),
            0xBD => return Ok(self.op_cp_a_index_l(prefix) + 4),
            _ => {}
        }

        if matches!(sub_opcode, 0x09 | 0x19 | 0x29 | 0x39) {
            return Ok(self.op_add_index_rr(prefix, sub_opcode));
        }
        if sub_opcode == 0x21 {
            return Ok(self.op_ld_index_nn(prefix));
        }
        if sub_opcode == 0x22 {
            return Ok(self.op_ld_nn_index(prefix));
        }
        if sub_opcode == 0x23 {
            return Ok(self.op_inc_index(prefix));
        }
        if sub_opcode == 0x2A {
            return Ok(self.op_ld_index_nn_from_mem(prefix));
        }
        if sub_opcode == 0x2B {
            return Ok(self.op_dec_index(prefix));
        }
        if sub_opcode == 0x34 {
            return Ok(self.op_inc_index_mem(prefix));
        }
        if sub_opcode == 0x35 {
            return Ok(self.op_dec_index_mem(prefix));
        }
        if sub_opcode == 0x36 {
            return Ok(self.op_ld_index_mem_n(prefix));
        }
        if matches!(sub_opcode, 0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x7E) {
            return Ok(self.op_ld_r_index_mem(prefix, sub_opcode));
        }
        if matches!(sub_opcode, 0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77) {
            return Ok(self.op_ld_index_mem_r(prefix, sub_opcode));
        }

        // indexed_memory_alu
        match sub_opcode {
            0x86 => return Ok(self.op_add_a_index_mem(prefix)),
            0x8E => return Ok(self.op_adc_a_index_mem(prefix)),
            0x96 => return Ok(self.op_sub_a_index_mem(prefix)),
            0x9E => return Ok(self.op_sbc_a_index_mem(prefix)),
            0xA6 => return Ok(self.op_and_a_index_mem(prefix)),
            0xAE => return Ok(self.op_xor_a_index_mem(prefix)),
            0xB6 => return Ok(self.op_or_a_index_mem(prefix)),
            0xBE => return Ok(self.op_cp_a_index_mem(prefix)),
            _ => {}
        }
        if matches!(
            sub_opcode,
            0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8
        ) {
            return Ok(self.op_ret_cc(sub_opcode) + 4);
        }
        if matches!(
            sub_opcode,
            0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA | 0xC3
        ) {
            return Ok(self.op_jp(sub_opcode) + 4);
        }
        if matches!(
            sub_opcode,
            0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC | 0xCD
        ) {
            return Ok(self.op_call(sub_opcode) + 4);
        }
        if sub_opcode == 0xC9 {
            return Ok(self.op_ret() + 4);
        }
        if matches!(
            sub_opcode,
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF
        ) {
            return Ok(self.op_rst(sub_opcode) + 4);
        }
        if matches!(
            sub_opcode,
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE
        ) {
            return Ok(self.op_alu_n(sub_opcode) + 4);
        }
        if sub_opcode == 0xEB {
            return Ok(self.op_ex_de_hl() + 4);
        }
        if sub_opcode == 0xD1 {
            return Ok(self.op_pop_rr(sub_opcode) + 4);
        }
        if sub_opcode == 0xD5 {
            return Ok(self.op_push_rr(sub_opcode) + 4);
        }
        if sub_opcode == 0xD9 {
            return Ok(self.op_exx() + 4);
        }
        if sub_opcode == 0xE1 {
            return Ok(self.op_pop_index(prefix));
        }
        if sub_opcode == 0xE3 {
            return Ok(self.op_ex_sp_index(prefix));
        }
        if sub_opcode == 0xE5 {
            return Ok(self.op_push_index(prefix));
        }
        if sub_opcode == 0xE9 {
            return Ok(self.op_jp_index(prefix));
        }
        if sub_opcode == 0xF1 {
            return Ok(self.op_pop_af() + 4);
        }
        if sub_opcode == 0xF5 {
            return Ok(self.op_push_af() + 4);
        }
        if sub_opcode == 0xF9 {
            return Ok(self.op_ld_sp_index(prefix));
        }
        if sub_opcode == 0xF3 {
            return Ok(self.op_interrupt_enable(false) + 4);
        }
        if sub_opcode == 0xFB {
            return Ok(self.op_interrupt_enable(true) + 4);
        }

        if sub_opcode == 0xC1 {
            return Ok(self.op_pop_rr(sub_opcode) + 4);
        }
        if sub_opcode == 0xC5 {
            return Ok(self.op_push_rr(sub_opcode) + 4);
        }

        // DD/FD followed by DD, FD, or ED: the reference rewinds PC to the byte
        // after the prefix and raises. Nothing else about the state is touched.
        let prefix_addr = self.pc.wrapping_sub(2);
        self.pc = prefix_addr.wrapping_add(1);
        Err(Fault::UnhandledIndexOpcode {
            prefix,
            opcode: sub_opcode,
            pc: prefix_addr,
        })
    }
}
