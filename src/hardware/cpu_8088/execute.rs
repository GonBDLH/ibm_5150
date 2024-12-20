use log::error;

use super::cpu_utils::*;
use super::instr_utils::*;
use super::regs::check_p;
use super::regs::check_s;
use super::Bus;
use super::CPU;

impl CPU {
    pub fn execute(&mut self, bus: &mut Bus) {
        match self.instr.opcode {
            Opcode::MOV => {
                let val = self.get_val(bus, self.instr.operand2);
                self.set_val(bus, self.instr.operand1, val);
            }
            Opcode::PUSH => {
                let mut val = self.get_val(bus, self.instr.operand1);
                if let OperandType::Register(op) = self.instr.operand1 {
                    if op == Operand::SP {
                        val = val.wrapping_sub(2);
                    }
                };
                self.push_stack_16(bus, val);
            }
            Opcode::POP => {
                let val = self.pop_stack_16(bus);
                self.set_val(bus, self.instr.operand1, val);
            }
            Opcode::XCHG => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);

                if let OperandType::Register(operand) = self.instr.operand1 {
                    self.set_reg(self.instr.data_length, operand, val2);
                } else {
                    unreachable!("No se pudo hacer esto")
                };
                self.set_val(bus, self.instr.operand2, val1);
            }
            Opcode::IN => {
                #[cfg(not(test))]
                let val = bus.port_in(self.instr.port);
                #[cfg(test)]
                let val = 0xFFFF;
                self.set_val(bus, self.instr.operand1, val);
            }
            Opcode::OUT => {
                #[cfg(not(test))]
                {
                    let val = self.get_val(bus, self.instr.operand2);
                    bus.port_out(self, val, self.instr.port);
                }
            }
            Opcode::XLAT => {
                let val = bus.read_8(
                    self.get_segment(self.instr.segment),
                    self.get_reg(Operand::BX)
                        .wrapping_add(self.get_reg(Operand::AL)),
                );
                self.set_reg8(Operand::AL, val);
            }
            Opcode::LEA => {
                // let val = bus.read_length(
                //     self,
                //     self.instr.segment,
                //     self.instr.offset,
                //     self.instr.data_length,
                // );
                // self.set_val(bus, self.instr.operand1, val);
                self.set_val(bus, self.instr.operand1, self.instr.offset);
            }
            Opcode::LDS => {
                let val = bus.read_length(
                    self,
                    self.instr.segment,
                    self.instr.offset,
                    self.instr.data_length,
                );
                // self.set_val(bus, self.instr.operand1, self.instr.offset);
                self.set_val(bus, self.instr.operand1, val);
                self.ds = bus.read_length(
                    self,
                    self.instr.segment,
                    self.instr.offset.wrapping_add(2),
                    self.instr.data_length,
                );
            }
            Opcode::LES => {
                let val = bus.read_length(
                    self,
                    self.instr.segment,
                    self.instr.offset,
                    self.instr.data_length,
                );
                // self.set_val(bus, self.instr.operand1, self.instr.offset);
                self.set_val(bus, self.instr.operand1, val);
                self.es = bus.read_length(
                    self,
                    self.instr.segment,
                    self.instr.offset.wrapping_add(2),
                    self.instr.data_length,
                );
            }
            Opcode::LAHF => {
                self.ax.high = self.flags.get_flags() as u8;
            }
            Opcode::SAHF => {
                // self.flags.set_flags(self.ax.high as u16);
                self.flags.s = self.ax.high & 0x80 > 0;
                self.flags.z = self.ax.high & 0x40 > 0;
                self.flags.a = self.ax.high & 0x10 > 0;
                self.flags.p = self.ax.high & 0x04 > 0;
                self.flags.c = self.ax.high & 0x01 > 0;
            }
            Opcode::PUSHF => {
                let val = self.flags.get_flags();
                self.push_stack_16(bus, val);
            }
            Opcode::POPF => {
                let val = self.pop_stack_16(bus);
                self.flags.set_flags(val);
            }

            Opcode::ADD => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = add(val1, val2, self.instr.data_length);
                self.set_val(bus, self.instr.operand1, res.0);
                self.flags
                    .set_add_flags(self.instr.data_length, val1, val2, res.0, res.1)
            }
            Opcode::ADC => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let cflag = self.flags.c as u16;
                let res = adc(val1, val2, cflag, self.instr.data_length);
                self.set_val(bus, self.instr.operand1, res.0);
                self.flags
                    .set_add_flags(self.instr.data_length, val1, val2, res.0, res.1)
            }
            Opcode::INC => {
                let val = self.get_val(bus, self.instr.operand1);
                let res = val.wrapping_add(1);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_inc_flags(self.instr.data_length, val, res);
            }

            // SACADO DE https://github.com/dbalsom/martypc/blob/version_0_2_3/core/src/cpu_808x/bcd.rs
            Opcode::AAA => {
                let old_al = self.ax.low;
                let new_al;

                if (self.ax.low & 0x0F) > 9 || self.flags.a {
                    self.ax.high = self.ax.high.wrapping_add(1);
                    new_al = self.ax.low.wrapping_add(0x06);
                    self.flags.a = true;
                    self.flags.c = true;
                } else {
                    new_al = self.ax.low;
                    self.flags.a = false;
                    self.flags.c = false;
                }
                self.ax.low = new_al & 0x0F;

                self.flags.z = new_al == 0;
                self.flags.p = check_p(new_al as u16);
                self.flags.o = (0x7A..=0x7F).contains(&old_al);
                self.flags.s = (0x7A..=0xF9).contains(&old_al);
            }

            // SACADO DE https://github.com/dbalsom/martypc/blob/version_0_2_3/core/src/cpu_808x/bcd.rs
            Opcode::DAA => {
                let old_al = self.ax.low;
                let old_af = self.flags.a;
                let old_cf = self.flags.c;

                self.flags.c = false;
                if old_cf {
                    if self.ax.low >= 0x1A && self.ax.low <= 0x7F {
                        self.flags.o = true;
                    }
                } else if self.ax.low >= 0x07A && self.ax.low <= 0x7F {
                    self.flags.o = false;
                }

                if (self.ax.low & 0x0F) > 9 || self.flags.a {
                    self.ax.low = self.ax.low.wrapping_add(6);
                    self.flags.a = true;
                } else {
                    self.flags.a = false;
                }

                let test_al = if old_af { 0x9F } else { 0x99 };

                if (old_al > test_al) || old_cf {
                    self.ax.low = self.ax.low.wrapping_add(0x60);
                    self.flags.c = true;
                } else {
                    self.flags.c = false;
                }

                self.flags.s = check_s(self.ax.low as u16, Length::Byte);
                self.flags.z = self.ax.low == 0;
                self.flags.p = check_p(self.ax.low as u16);
            }
            Opcode::SUB => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = sub(val1, val2, self.instr.data_length);
                // let res = val1.wrapping_sub(val2);
                self.set_val(bus, self.instr.operand1, res.0);
                self.flags
                    .set_sub_flags(self.instr.data_length, val1, val2, res.0, res.1);
            }
            Opcode::SBB => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let cflag = self.flags.c as u16;
                let res = sbb(val1, val2, cflag, self.instr.data_length);
                self.set_val(bus, self.instr.operand1, res.0);
                self.flags
                    .set_sub_flags(self.instr.data_length, val1, val2, res.0, res.1);
            }
            Opcode::DEC => {
                let val = self.get_val(bus, self.instr.operand1);
                let res = val.wrapping_sub(1);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_dec_flags(self.instr.data_length, val, res);
            }
            Opcode::NEG => {
                let val = self.get_val(bus, self.instr.operand1);
                let res = 0u16.wrapping_sub(val);
                self.set_val(bus, self.instr.operand1, res);
                self.flags
                    .set_neg_flags(self.instr.data_length, 0, val, res);
            }
            Opcode::CMP => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = sub(val1, val2, self.instr.data_length);
                self.flags
                    .set_sub_flags(self.instr.data_length, val1, val2, res.0, res.1);
            }
            Opcode::AAS => {
                let old_al = self.ax.low;
                let old_af = self.flags.a;
                let new_al;

                if (self.ax.low & 0x0F) > 9 || self.flags.a {
                    new_al = self.ax.low.wrapping_sub(6);
                    self.ax.high = self.ax.high.wrapping_sub(1);
                    self.flags.a = true;
                    self.flags.c = true;
                    self.ax.low = new_al & 0x0F;
                } else {
                    new_al = self.ax.low;
                    self.ax.low = new_al & 0x0F;
                    self.flags.a = false;
                    self.flags.c = false;
                }

                self.flags.o = false;
                self.flags.s = false;
                self.flags.z = new_al == 0;
                self.flags.p = check_p(new_al as u16);

                if old_af && (0x80..=0x85).contains(&old_al) {
                    self.flags.o = true;
                }
                if !old_af && old_al >= 0x80 {
                    self.flags.s = true;
                }
                if old_af && ((old_al <= 0x05) || (old_al >= 0x86)) {
                    self.flags.s = true;
                }
            }

            // SACADO DE https://github.com/dbalsom/martypc/blob/version_0_2_3/core/src/cpu_808x/bcd.rs
            Opcode::DAS => {
                let old_al = self.ax.low;
                let old_cf = self.flags.c;
                let old_af = self.flags.a;

                let test_al = if old_af { 0x9F } else { 0x99 };

                match (old_af, old_cf) {
                    (false, false) => {
                        if (0x9A..=0xDF).contains(&self.ax.low) {
                            self.flags.o = true;
                        }
                    }
                    (true, false) => {
                        if (0x80..=0x85).contains(&self.ax.low)
                            || (0xA0..=0xE5).contains(&self.ax.low)
                        {
                            self.flags.o = true;
                        }
                    }
                    (false, true) => {
                        if (0x80..=0xDF).contains(&self.ax.low) {
                            self.flags.o = true;
                        }
                    }
                    (true, true) => {
                        if (0x80..=0xE5).contains(&self.ax.low) {
                            self.flags.o = true;
                        }
                    }
                }

                self.flags.c = false;
                if (self.ax.low & 0x0F) > 9 || self.flags.a {
                    // let val = self.ax.low.overflowing_sub(6);
                    // self.ax.low = val.0;
                    // self.flags.c = old_cf || val.1;
                    // self.flags.a = true;
                    self.ax.low = self.ax.low.wrapping_sub(6);
                    self.flags.a = true;
                } else {
                    self.flags.a = false;
                }
                if old_al > test_al || old_cf {
                    self.ax.low = self.ax.low.wrapping_sub(0x60);
                    self.flags.c = true;
                } else {
                    self.flags.c = false;
                }
                self.flags.set_das_flags(Length::Byte, self.ax.low as u16);
            }
            Opcode::MUL => {
                let val1 = match self.instr.data_length {
                    Length::Byte => self.ax.low as u64,
                    Length::Word => self.ax.get_x() as u64,
                    _ => unreachable!(),
                };
                let val2 = self.get_val(bus, self.instr.operand1) as u64;
                let res = val1.wrapping_mul(val2);

                match self.instr.data_length {
                    Length::Byte => self.ax.set_x(res as u16),
                    Length::Word => {
                        self.dx.set_x((res >> 16) as u16);
                        self.ax.set_x(res as u16);
                    }
                    _ => unreachable!(),
                };

                self.flags.set_mul_flags(self.instr.data_length, res);
            }
            Opcode::IMUL => {
                let val1 = match self.instr.data_length {
                    Length::Byte => self.ax.low as i8 as i16 as i32,
                    Length::Word => self.ax.get_x() as i16 as i32,
                    _ => unreachable!(),
                };

                let res = match self.instr.data_length {
                    Length::Byte => {
                        let val2 = self.get_val(bus, self.instr.operand1) as i8 as i16 as i32;
                        let res = val1.wrapping_mul(val2);
                        self.ax.set_x(res as u16);
                        res
                    }
                    Length::Word => {
                        let val2 = self.get_val(bus, self.instr.operand1) as i16 as i32;
                        let res = val1.wrapping_mul(val2);
                        self.dx.set_x((res >> 16) as u16);
                        self.ax.set_x(res as u16);
                        res
                    }
                    _ => unreachable!(),
                };

                self.flags
                    .set_imul_flags(self.instr.data_length, res as u32);
            }
            Opcode::AAM => {
                let temp_al = self.ax.low;
                let val = self.get_val(bus, self.instr.operand1);
                // let val = 10;
                if val == 0 {
                    self.flags.set_aam_flags(self.ax.low);
                    self.sw_int = true;
                    self.instr.sw_int_type = 0;
                    return;
                }

                self.ax.high = temp_al / val as u8;
                self.ax.low = temp_al % val as u8;

                self.flags.set_aam_flags(self.ax.low);
            }
            Opcode::DIV => {
                let val2 = self.get_val(bus, self.instr.operand1);

                if val2 == 0 {
                    // self.interrupt(bus, 0);
                    self.sw_int = true;
                    self.instr.sw_int_type = 0;
                    return;
                }

                match self.instr.data_length {
                    Length::Byte => {
                        let val1 = self.ax.get_x();
                        let res_div = val1.wrapping_div(val2);

                        if res_div > 0xFF {
                            self.sw_int = true;
                            self.instr.sw_int_type = 0;
                            return;
                        }

                        self.set_reg(self.instr.data_length, Operand::AL, res_div);
                        self.set_reg(self.instr.data_length, Operand::AH, val1.wrapping_rem(val2));
                    }
                    Length::Word => {
                        let val1 = to_u32(self.ax.get_x(), self.dx.get_x());
                        let res_div = val1.wrapping_div(val2 as u32);
                        if res_div > 0xFFFF {
                            self.sw_int = true;
                            self.instr.sw_int_type = 0;
                            return;
                        }
                        self.set_reg(self.instr.data_length, Operand::AX, res_div as u16);
                        self.set_reg(
                            self.instr.data_length,
                            Operand::DX,
                            val1.wrapping_rem(val2 as u32) as u16,
                        );
                    }
                    _ => unreachable!(),
                };
            }
            Opcode::IDIV => {
                let val2 = self.get_val(bus, self.instr.operand1);

                if val2 == 0 {
                    // self.interrupt(bus, 0);
                    self.sw_int = true;
                    self.instr.sw_int_type = 0;
                    return;
                }

                let rep_neg = if self.instr.repetition_prefix != RepetitionPrefix::None {
                    -1
                } else {
                    1
                };

                match self.instr.data_length {
                    Length::Byte => {
                        let val1 = self.ax.get_x() as i16;
                        let val2 = val2 as u8 as i8 as i16;
                        let res = val1.wrapping_div(val2);
                        if !(-127..=127).contains(&res) {
                            self.sw_int = true;
                            self.instr.sw_int_type = 0;
                        } else {
                            self.set_reg(
                                self.instr.data_length,
                                Operand::AL,
                                (res * rep_neg) as u16,
                            );
                            self.set_reg(
                                self.instr.data_length,
                                Operand::AH,
                                val1.wrapping_rem(val2) as u16,
                            );
                        }
                    }
                    Length::Word => {
                        let val1 = to_u32(self.ax.get_x(), self.dx.get_x()) as i32;
                        let val2 = val2 as i16 as i32;
                        let res = val1.wrapping_div(val2);
                        if !(-32767..=32767).contains(&res) {
                            self.sw_int = true;
                            self.instr.sw_int_type = 0;
                        } else {
                            self.set_reg(
                                self.instr.data_length,
                                Operand::AX,
                                (res * (rep_neg as i32)) as u16,
                            );
                            self.set_reg(
                                self.instr.data_length,
                                Operand::DX,
                                val1.wrapping_rem(val2) as u16,
                            );
                        }
                    }
                    _ => unreachable!(),
                };
            }
            Opcode::AAD => {
                let temp_al = self.ax.low;
                let temp_ah = self.ax.high;
                self.ax.high = 0;
                let val = self.get_val(bus, self.instr.operand1);
                self.ax.low = (temp_al as u32 + (temp_ah as u32 * val as u32)) as u8;

                self.flags.set_aam_flags(self.ax.low);
            }
            Opcode::CBW => {
                self.ax.set_x(sign_extend(self.ax.low));
            }
            Opcode::CWD => {
                let val = to_2u16(sign_extend_32(self.ax.get_x()));
                self.ax.set_x(val.0);
                self.dx.set_x(val.1);
            }

            Opcode::NOT => {
                let val = self.get_val(bus, self.instr.operand1);
                self.set_val(bus, self.instr.operand1, !val)
            }
            Opcode::SALSHL => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let res = salshl(self, val, count, self.instr.data_length);

                self.set_val(bus, self.instr.operand1, res);

                self.flags.o = get_msb(res, self.instr.data_length) ^ self.flags.c;

                if count == 0 {
                    return;
                }

                self.flags.s = check_s(res, self.instr.data_length);
                self.flags.z = res == 0;
                self.flags.p = check_p(res);
            }
            Opcode::SALC => {
                if self.flags.c {
                    self.ax.low = 0xFF;
                } else {
                    self.ax.low = 0x00;
                }
            }
            Opcode::SHR => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let res = shr(self, val, count, self.instr.data_length);

                self.set_val(bus, self.instr.operand1, res);

                if count == 1 {
                    self.flags.o = get_msb(val, self.instr.data_length);
                } else {
                    self.flags.o = false;
                }

                self.flags.a = false;
                self.flags.s = check_s(res, self.instr.data_length);
                self.flags.z = res == 0;
                self.flags.p = check_p(res);
            }
            Opcode::SAR => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let res = sar(self, val, count, self.instr.data_length);

                self.set_val(bus, self.instr.operand1, res);

                self.flags.o = false;

                if count == 0 {
                    return;
                }

                self.flags.s = check_s(res, self.instr.data_length);
                self.flags.z = res == 0;
                self.flags.p = check_p(res);
            }
            Opcode::ROL => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                // let tmp_count = (count & 0x1F) % self.instr.data_length.get_num_bits() as u32;

                let res = rotate_left(val, count, self.instr.data_length);

                if count != 0 {
                    self.set_val(bus, self.instr.operand1, res);
                }

                self.flags
                    .set_rl_flags(count, self.instr.data_length, val, res, get_lsb(res as u8))
            }
            Opcode::ROR => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let tmp_count = (count & 0x1F) % self.instr.data_length.get_num_bits() as u32;

                let res = rotate_right(val, tmp_count, self.instr.data_length);

                if tmp_count != 0 {
                    self.set_val(bus, self.instr.operand1, res);
                }

                self.flags.set_rr_flags(
                    count,
                    self.instr.data_length,
                    val,
                    res,
                    get_msb(res, self.instr.data_length),
                )
            }
            Opcode::RCL => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let tmp_count = count % (self.instr.data_length.get_num_bits() as u32 + 1);

                let res = rotate_left_carry(self, val, tmp_count, self.instr.data_length);

                if tmp_count != 0 {
                    self.set_val(bus, self.instr.operand1, res);
                }

                self.flags
                    .set_rl_flags(count, self.instr.data_length, val, res, self.flags.c);
            }
            Opcode::RCR => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let tmp_count = count % (self.instr.data_length.get_num_bits() as u32 + 1);

                let res = rotate_right_carry(self, val, tmp_count, self.instr.data_length);

                if tmp_count != 0 {
                    self.set_val(bus, self.instr.operand1, res);
                }

                self.flags
                    .set_rr_flags(count, self.instr.data_length, val, res, self.flags.c);
            }
            Opcode::AND => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1 & val2;
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_logic_flags(self.instr.data_length, res)
            }
            Opcode::TEST => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1 & val2;
                self.flags.set_logic_flags(self.instr.data_length, res)
            }
            Opcode::OR => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1 | val2;
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_logic_flags(self.instr.data_length, res)
            }
            Opcode::XOR => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1 ^ val2;
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_logic_flags(self.instr.data_length, res)
            }

            Opcode::SETMO => {
                self.set_val(bus, self.instr.operand1, 0xFFFF);
                self.flags.p = check_p(0xFFFF);
                self.flags.a = false;
                self.flags.c = false;
                self.flags.o = false;
                self.flags.s = true;
                self.flags.z = false;
            }

            Opcode::SETMOC => {
                if self.cx.low != 0 {
                    self.set_val(bus, self.instr.operand1, 0xFFFF);
                    self.flags.p = check_p(0xFFFF);
                    self.flags.a = false;
                    self.flags.c = false;
                    self.flags.o = false;
                    self.flags.s = true;
                    self.flags.z = false;
                }
            }

            Opcode::MOVSB => self.string_op(bus, CPU::movs, 17),
            Opcode::MOVSW => self.string_op(bus, CPU::movs, 25),
            Opcode::CMPSB => self.string_op_z(bus, CPU::cmps, 22),
            Opcode::CMPSW => self.string_op_z(bus, CPU::cmps, 30),
            Opcode::SCASB => self.string_op_z(bus, CPU::scas, 15),
            Opcode::SCASW => self.string_op_z(bus, CPU::scas, 19),
            Opcode::LODSB => self.string_op(bus, CPU::lods, 13),
            Opcode::LODSW => self.string_op(bus, CPU::lods, 17),
            Opcode::STOSB => self.string_op(bus, CPU::stos, 13),
            Opcode::STOSW => self.string_op(bus, CPU::stos, 17),

            Opcode::CALL => match self.instr.jump_type {
                JumpType::DirWithinSegment(disp) => {
                    self.push_stack_16(bus, self.ip);
                    self.ip = self.ip.wrapping_add(disp);
                }
                JumpType::IndWithinSegment(new_ip) => {
                    self.push_stack_16(bus, self.ip);
                    self.ip = new_ip;
                }
                JumpType::DirIntersegment(offset, segment) => {
                    self.push_stack_16(bus, self.cs);
                    self.push_stack_16(bus, self.ip);
                    self.ip = offset;
                    self.cs = segment;
                }
                JumpType::IndIntersegment(new_cs, new_ip) => {
                    self.push_stack_16(bus, self.cs);
                    self.push_stack_16(bus, self.ip);
                    self.ip = new_ip;
                    self.cs = new_cs;
                }
                _ => unreachable!(),
            },
            Opcode::JMP => {
                match self.instr.jump_type {
                    JumpType::DirWithinSegment(disp) => self.ip = self.ip.wrapping_add(disp),
                    JumpType::DirWithinSegmentShort(disp) => {
                        self.ip = self.ip.wrapping_add(sign_extend(disp))
                    }
                    JumpType::IndWithinSegment(imm) => self.ip = imm,
                    JumpType::IndIntersegment(new_cs, new_ip) => {
                        self.ip = new_ip;
                        self.cs = new_cs;
                    }
                    JumpType::DirIntersegment(offset, segment) => {
                        self.cs = segment;
                        self.ip = offset;
                        let _a = 0;
                    }
                    _ => unreachable!(),
                };
            }
            Opcode::RET => match self.instr.ret_type {
                RetType::NearAdd(val) => {
                    self.ip = self.pop_stack_16(bus);
                    self.sp = self.sp.wrapping_add(val);
                }
                RetType::Near => {
                    self.ip = self.pop_stack_16(bus);
                }
                RetType::FarAdd(val) => {
                    self.ip = self.pop_stack_16(bus);
                    self.cs = self.pop_stack_16(bus);
                    self.sp = self.sp.wrapping_add(val);
                }
                RetType::Far => {
                    self.ip = self.pop_stack_16(bus);
                    self.cs = self.pop_stack_16(bus);
                }
                _ => unreachable!(),
            },
            Opcode::JEJZ => self.jump_short(self.flags.z),
            Opcode::JLJNGE => self.jump_short(self.flags.s ^ self.flags.o),
            Opcode::JLEJNG => self.jump_short((self.flags.s ^ self.flags.o) | self.flags.z),
            Opcode::JBJNAE => self.jump_short(self.flags.c),
            Opcode::JBEJNA => self.jump_short(self.flags.c | self.flags.z),
            Opcode::JPJPE => self.jump_short(self.flags.p),
            Opcode::JO => self.jump_short(self.flags.o),
            Opcode::JS => self.jump_short(self.flags.s),
            Opcode::JNEJNZ => self.jump_short(!self.flags.z),
            Opcode::JNLJGE => self.jump_short(!(self.flags.s ^ self.flags.o)),
            Opcode::JNLEJG => self.jump_short(!((self.flags.s ^ self.flags.o) | self.flags.z)),
            Opcode::JNBJAE => self.jump_short(!self.flags.c),
            Opcode::JNBEJA => self.jump_short(!self.flags.c & !self.flags.z),
            Opcode::JNPJPO => self.jump_short(!self.flags.p),
            Opcode::JNO => self.jump_short(!self.flags.o),
            Opcode::JNS => self.jump_short(!self.flags.s),
            Opcode::LOOP => {
                let cx = self.cx.get_x().wrapping_sub(1);
                self.cx.set_x(cx);
                self.jump_short(cx != 0);

                self.cycles += 1; // jump_short() ya suma lo demas
            }
            Opcode::LOOPZE => {
                let cx = self.cx.get_x().wrapping_sub(1);
                self.cx.set_x(cx);
                self.jump_short((cx != 0) & self.flags.z);

                self.cycles += 2; // jump_short() ya suma lo demas
            }
            Opcode::LOOPNZNE => {
                let cx = self.cx.get_x().wrapping_sub(1);
                self.cx.set_x(cx);
                self.jump_short((cx != 0) & !self.flags.z);

                self.cycles += 2; // jump_short() ya suma lo demas
            }
            Opcode::JCXZ => {
                self.jump_short(self.cx.get_x() == 0);

                self.cycles += 2; // jump_short() ya suma lo demas
            }
            Opcode::INT => {
                self.sw_int = true;
            }
            Opcode::INTO => {
                if self.flags.o {
                    self.sw_int = true;
                    self.cycles += 73;
                } else {
                    self.cycles += 4;
                }
            }
            Opcode::IRET => {
                self.ip = self.pop_stack_16(bus);
                self.cs = self.pop_stack_16(bus);
                let flags = self.pop_stack_16(bus);
                self.flags.set_flags(flags)
            }

            Opcode::CLC => {
                self.flags.c = false;
            }
            Opcode::CMC => {
                self.flags.c = !self.flags.c;
            }
            Opcode::STC => {
                self.flags.c = true;
            }
            Opcode::CLD => {
                self.flags.d = false;
            }
            Opcode::STD => {
                self.flags.d = true;
            }
            Opcode::CLI => {
                self.flags.i = false;
            }
            Opcode::STI => {
                self.flags.i = true;
            }

            Opcode::HLT => {
                self.halted = true;
            }

            Opcode::NOP => {}

            Opcode::None => {
                error!("ERROR DECODING");
                panic!()
            }
        }
    }
}
