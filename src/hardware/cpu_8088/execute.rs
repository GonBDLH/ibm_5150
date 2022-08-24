use super::CPU;
use super::Bus;
use super::instr_utils::*;
use super::cpu_utils::*;

impl CPU {
   pub fn execute(&mut self, bus: &mut Bus) {
        match self.instr.opcode {
            Opcode::MOV => {
                let val = self.get_val(bus, self.instr.operand2);
                self.set_val(bus, self.instr.operand1, val);
            },
            Opcode::PUSH => {
                let val = self.get_val(bus, self.instr.operand1);
                self.push_stack(bus, val);
            },
            Opcode::POP => {
                let val = self.pop_stack(bus);
                self.set_val(bus, self.instr.operand1, val);
            },
            Opcode::XCHG => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                
                if let OperandType::Register(operand) = self.instr.operand1 {
                    self.set_reg(self.instr.data_length, operand, val2);
                } else {panic!("No se pudo hacer esto")};
                self.set_val(bus, self.instr.operand2, val1);
            },
            Opcode::IN => {
                // TODO ES UNA PRUEBA let val = bus.port_in(self.instr.port);
                let val = 0x00;
                // let val: u16 = rand::random(); 
                self.set_val(bus, self.instr.operand1, val);
            },
            Opcode::OUT => {
                let val = self.get_val(bus, self.instr.operand2);
                bus.port_out(val, self.instr.port);
            },
            Opcode::XLAT => {
                let val = bus.read_8(self.get_segment(Operand::DS), self.get_reg16(Operand::BX) + self.get_reg8(Operand::AL));
                self.set_reg8(Operand::AL, val);
            },
            Opcode::LEA => {
                self.set_val(bus, self.instr.operand1, self.instr.offset);
            }
            Opcode::LDS => {
                self.set_val(bus, self.instr.operand1, self.instr.offset);
                self.ds = bus.read_length(self, self.instr.segment, self.instr.offset.wrapping_add(2), self.instr.data_length);
            },
            Opcode::LES => {
                self.set_val(bus, self.instr.operand1, self.instr.offset);
                self.es = bus.read_length(self, self.instr.segment, self.instr.offset.wrapping_add(2), self.instr.data_length);
            },
            Opcode::LAHF => {
                self.ax.high = self.flags.get_flags() as u8;
            },
            Opcode::SAHF => {
                self.flags.set_flags(self.ax.high as u16);
            },
            Opcode::PUSHF => {
                let val = self.flags.get_flags();
                self.push_stack_16(bus, val);
            },
            Opcode::POPF => {
                let val = self.pop_stack_16(bus);
                self.flags.set_flags(val);
            },

            Opcode::ADD => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1.wrapping_add(val2);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_add_flags(self.instr.data_length, val1, val2, res)
            },
            Opcode::ADC => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2).wrapping_add(self.flags.c as u16);
                let res = val1.wrapping_add(val2);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_add_flags(self.instr.data_length, val1, val2, res)
            },
            Opcode::INC => {
                let val = self.get_val(bus, self.instr.operand1);
                let res = val.wrapping_add(1);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_add_flags(self.instr.data_length, val, 1, res);
            },
            Opcode::AAA => {
                if (self.ax.low & 0x0F) > 9 || self.flags.a {
                    let val = self.ax.get_x();
                    self.ax.set_x(val.wrapping_add(0x106));
                    self.flags.a = true;
                    self.flags.c = true;
                } else {
                    self.flags.a = false;
                    self.flags.c = false;
                }
                let val = self.ax.low;
                self.ax.low = val & 0x0F;
            },
            Opcode::DAA => {
                let old_al = self.ax.low;
                let old_cf = self.flags.c;
                self.flags.c = false;
                if (self.ax.low & 0x0F) > 9 || self.flags.a {
                    let val = self.ax.low.overflowing_add(6);
                    self.ax.low = val.0;
                    self.flags.c = old_cf || val.1;
                    self.flags.a = true;
                } else {
                    self.flags.a = false;
                }
                if old_al > 0x99 || old_cf {
                    self.ax.low = self.ax.low.wrapping_add(0x60);
                    self.flags.c = true;
                } else {
                    self.flags.c = false;
                }
            },
            Opcode::SUB => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1.wrapping_sub(val2);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_sub_flags(self.instr.data_length, val1, val2, res);
            },
            Opcode::SBB => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2).wrapping_add(self.flags.c as u16);
                let res = val1.wrapping_sub(val2);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_sub_flags(self.instr.data_length, val1, val2, res);
            },
            Opcode::DEC => {
                let val = self.get_val(bus, self.instr.operand1);
                let res = val.wrapping_sub(1);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_sub_flags(self.instr.data_length, val, 1, res);
            },
            Opcode::NEG => {
                let val = self.get_val(bus, self.instr.operand1);
                let res = 0u16.wrapping_sub(val);
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_neg_flags(self.instr.data_length, 0, val, res);
            },
            Opcode::CMP => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2).wrapping_add(self.flags.c as u16);
                let res = val1.wrapping_sub(val2);
                self.flags.set_sub_flags(self.instr.data_length, val1, val2, res);
            },
            Opcode::AAS => {
                if (self.ax.low & 0x0F) > 9 || self.flags.a {
                    let val = self.ax.get_x();
                    self.ax.set_x(val.wrapping_sub(6));
                    self.ax.high = self.ax.high.wrapping_sub(1);
                    self.flags.a = true;
                    self.flags.c = true;
                    self.ax.low = self.ax.low & 0x0F;
                } else {
                    self.flags.a = false;
                    self.flags.c = false;
                    self.ax.low = self.ax.low & 0x0F;
                }
            },
            Opcode::DAS => {
                let old_al = self.ax.low;
                let old_cf = self.flags.c;
                self.flags.c = false;
                if (self.ax.low & 0x0F) > 9 || self.flags.a {
                    let val = self.ax.low.overflowing_sub(6);
                    self.ax.low = val.0;
                    self.flags.c = old_cf || val.1;
                    self.flags.a = true;
                } else {
                    self.flags.a = false;
                }
                if old_al > 0x99 || old_cf {
                    self.ax.low = self.ax.low.wrapping_sub(0x60);
                    self.flags.c = true;
                }
            },
            Opcode::MUL => {
                let val1 = match self.instr.data_length {
                    Length::Byte => self.ax.low as u32,
                    Length::Word => self.ax.get_x() as u32,
                    _ => unreachable!(),
                };
                let val2 = self.get_val(bus, self.instr.operand1) as u32;
                let res = val1.wrapping_mul(val2);

                match self.instr.data_length {
                    Length::Byte => self.ax.set_x(res as u16),
                    Length::Word => {
                        self.dx.set_x((res >> 16) as u16);
                        self.ax.set_x(res as u16);
                    },
                    _ => unreachable!(),
                };

                self.flags.set_mul_flags(self.instr.data_length, res);
            },
            Opcode::IMUL => {
                let val1 = match self.instr.data_length {
                    Length::Byte => self.ax.low as i8 as i16 as i32,
                    Length::Word => self.ax.get_x() as i16 as i32,
                    _ => unreachable!(),
                };
                let val2 = self.get_val(bus, self.instr.operand1) as i32;
                let res = val1.wrapping_mul(val2);

                match self.instr.data_length {
                    Length::Byte => self.ax.set_x(res as u16),
                    Length::Word => {
                        self.dx.set_x((res >> 16) as u16);
                        self.ax.set_x(res as u16);
                    },
                    _ => unreachable!(),
                };

                self.flags.set_mul_flags(self.instr.data_length, res as u32);
            },
            Opcode::AAM => {
                let temp_al = self.ax.low;
                let val = self.get_val(bus, self.instr.operand1);
                self.ax.high = temp_al / val as u8;
                self.ax.low = temp_al % val as u8;

                self.flags.set_aam_flags(self.ax.low);
            },
            Opcode::DIV => {
                let val2 = self.get_val(bus, self.instr.operand1);

                if val2 == 0 {
                    // self.interrupt(bus, 0);
                    self.sw_int = true;
                    self.sw_int_type = 0;
                    return;
                }

                match self.instr.data_length {
                    Length::Byte => {
                        let val1 = self.ax.low as u16;
                        self.set_reg(self.instr.data_length, Operand::AL, val1.wrapping_div(val2));
                        self.set_reg(self.instr.data_length, Operand::AH, val1.wrapping_rem(val2));
                    },
                    Length::Word => {
                        let val1 = self.ax.get_x();
                        self.set_reg(self.instr.data_length, Operand::AX, val1.wrapping_div(val2));
                        self.set_reg(self.instr.data_length, Operand::DX, val1.wrapping_rem(val2));
                    },
                    _ => unreachable!(),
                };
            },
            Opcode::IDIV => {
                let val2 = self.get_val(bus, self.instr.operand1) as i16;

                if val2 == 0 {
                    // self.interrupt(bus, 0);
                    self.sw_int = true;
                    self.sw_int_type = 0;
                    return;
                }

                match self.instr.data_length {
                    Length::Byte => {
                        let val1 = self.ax.low as i8 as i16;
                        let res = val1.wrapping_div(val2);
                        if res > 0x7F || -res > 0x80 {
                            // self.interrupt(bus, 0);
                            self.sw_int = true;
                            self.sw_int_type = 0;
                        } else {
                            self.set_reg(self.instr.data_length, Operand::AL, res as u16);
                            self.set_reg(self.instr.data_length, Operand::AH, val1.wrapping_rem(val2) as u16);
                        }
                    },
                    Length::Word => {
                        let val1 = to_u32(self.ax.get_x(), self.dx.get_x()) as i32;
                        let res = val1.wrapping_div(val2 as i32);
                        if res > 0x7FFF || -res > 0x8000 {
                            // self.interrupt(bus, 0);
                            self.sw_int = true;
                            self.sw_int_type = 0;
                        } else {
                            self.set_reg(self.instr.data_length, Operand::AX, res as u16);
                            self.set_reg(self.instr.data_length, Operand::DX, val1.wrapping_rem(val2 as i32) as u16);
                        }
                    },
                    _ => unreachable!(),
                };
            },
            Opcode::AAD => {
                let temp_al = self.ax.low;
                let temp_ah = self.ax.high;
                self.ax.high = 0;
                let val = self.get_val(bus, self.instr.operand1);
                self.ax.low = (temp_al + (temp_ah.wrapping_add(val as u8))) & 0xFF;

                self.flags.set_aam_flags(self.ax.low);
            },
            Opcode::CBW => {
                self.ax.set_x(sign_extend(self.ax.low));
            },
            Opcode::CWD => {
                let val = to_2u16(sign_extend_32(self.ax.get_x()));
                self.ax.set_x(val.0);
                self.dx.set_x(val.1);
            },

            Opcode::NOT => {
                let val = self.get_val(bus, self.instr.operand1);
                self.set_val(bus, self.instr.operand1, !val)
            },
            Opcode::SALSHL => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let res = val.wrapping_shl(count);

                self.set_val(bus, self.instr.operand1, res);

                self.flags.set_salshl_flags(self.instr.data_length, self.instr.operand2, count, val, res);
            },
            Opcode::SHR => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let res = val.wrapping_shr(count);

                self.set_val(bus, self.instr.operand1, res);

                self.flags.set_shr_flags(self.instr.data_length, self.instr.operand2, count, val, res);
            },
            Opcode::SAR => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let mask = if count < 16 {
                    0xFFFFu16 << (16 - count)
                } else {
                    0xFFFF
                };
                
                let res = val.wrapping_shr(count) | mask;

                self.set_val(bus, self.instr.operand1, res);

                self.flags.set_sar_flags(self.instr.data_length, self.instr.operand2, count, val, res);
            },
            Opcode::ROL => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let res = rotate(val, count, self.instr.data_length, true);

                self.set_val(bus, self.instr.operand1, res.0);

                self.flags.set_rot_flags(res.1, count, res.0, val, self.instr.data_length);
            },
            Opcode::ROR => {
                let val = self.get_val(bus, self.instr.operand1);
                let count = self.get_val(bus, self.instr.operand2) as u32;

                let res = rotate(val, count, self.instr.data_length, false);

                self.set_val(bus, self.instr.operand1, res.0);

                self.flags.set_rot_flags(res.1, count, res.0, val, self.instr.data_length);
            },
            Opcode::RCL => {
                // TODO
            },
            Opcode::RCR => {
                // TODO
            },
            Opcode::AND => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1 & val2;
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_logic_flags(self.instr.data_length, res)
            },
            Opcode::TEST => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1 & val2;
                self.flags.set_logic_flags(self.instr.data_length, res)
                
            },
            Opcode::OR => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1 | val2;
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_logic_flags(self.instr.data_length, res)
            },
            Opcode::XOR => {
                let val1 = self.get_val(bus, self.instr.operand1);
                let val2 = self.get_val(bus, self.instr.operand2);
                let res = val1 ^ val2;
                self.set_val(bus, self.instr.operand1, res);
                self.flags.set_logic_flags(self.instr.data_length, res)
            },

            Opcode::MOVSB => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.movs(bus);
                    self.adjust_string();
                } else {
                    let mut veces = 0;
                    while self.cx.get_x() != 0 {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.movs(bus);
                        self.adjust_string();

                        veces += 1;
                    }
                    self.cycles += veces * 17;
                }
            },
            Opcode::MOVSW => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.movs(bus);
                    self.adjust_string();
                } else {
                    let mut veces = 0;
                    while self.cx.get_x() != 0 {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.movs(bus);
                        self.adjust_string();

                        veces += 1;
                    }
                    self.cycles += veces * 25;
                }
            },
            Opcode::CMPSB => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.cmps(bus);
                    self.adjust_string();
                } else {
                    let mut veces = 0;
                    let mut z_bool = true; // El bucle tiene que ejecutarse una vez minimo
                    while self.cx.get_x() != 0 && z_bool {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.cmps(bus);

                        self.adjust_string();

                        z_bool = self.check_z_str();

                        veces += 1;
                    }
                    self.cycles += veces * 22;
                }
            },
            Opcode::CMPSW => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.cmps(bus);
                    self.adjust_string();
                } else {
                    let mut veces = 0;
                    let mut z_bool = true; // El bucle tiene que ejecutarse una vez minimo
                    while self.cx.get_x() != 0 && z_bool {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.cmps(bus);

                        self.adjust_string();

                        z_bool = self.check_z_str();

                        veces += 1;
                    }
                    self.cycles += veces * 30;
                }
            },
            Opcode::SCASB => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.scas(bus);
                    self.adjust_string_di();
                } else {
                    let mut veces = 0;
                    let mut z_bool = true; // El bucle tiene que ejecutarse una vez minimo
                    while self.cx.get_x() != 0 && z_bool {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.scas(bus);

                        self.adjust_string_di();

                        z_bool = self.check_z_str();

                        veces += 1;
                    }
                    self.cycles += veces * 15;
                }
            },
            Opcode::SCASW => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.scas(bus);
                    self.adjust_string_di();
                } else {
                    let mut veces = 0;
                    let mut z_bool = true; // El bucle tiene que ejecutarse una vez minimo
                    while self.cx.get_x() != 0 && z_bool {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.scas(bus);

                        self.adjust_string_di();

                        z_bool = self.check_z_str();

                        veces += 1;
                    }
                    self.cycles += veces * 19;
                }
            },
            Opcode::LODSB => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.lods(bus);
                    self.adjust_string_si();
                } else {
                    let mut veces = 0;
                    while self.cx.get_x() != 0 {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.lods(bus);
                        self.adjust_string_si();

                        veces += 1;
                    }
                    self.cycles += veces * 13;
                }
            },
            Opcode::LODSW => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.lods(bus);
                    self.adjust_string_si();
                } else {
                    let mut veces = 0;
                    while self.cx.get_x() != 0 {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.lods(bus);
                        self.adjust_string_si();

                        veces += 1;
                    }
                    self.cycles += veces * 17;
                }
            },
            Opcode::STOSB => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.stos(bus);
                    self.adjust_string_di();
                } else {
                    let mut veces = 0;
                    while self.cx.get_x() != 0 {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.stos(bus);
                        self.adjust_string_di();

                        veces += 1;
                    }
                    self.cycles += veces * 13;
                }
            },
            Opcode::STOSW => {
                if self.instr.repetition_prefix != RepetitionPrefix::None {
                    self.stos(bus);
                    self.adjust_string_di();
                } else {
                    let mut veces = 0;
                    while self.cx.get_x() != 0 {
                        // TODO Test interrupts

                        self.cx.set_x(self.cx.get_x() - 1);
                        // String op
                        self.stos(bus);
                        self.adjust_string_di();

                        veces += 1;
                    }
                    self.cycles += veces * 17;
                }
            },

            Opcode::CALL => {
                match self.instr.jump_type {
                    JumpType::DirWithinSegment(disp) => {
                        self.push_stack_16(bus, self.ip);
                        self.ip = self.ip.wrapping_add(disp);
                    },
                    JumpType::IndWithinSegment => {
                        self.push_stack_16(bus, self.ip);
                        let val = self.get_val(bus, self.instr.operand1);
                        self.ip = val;
                    },
                    JumpType::DirIntersegment(offset, segment) => {
                        self.push_stack_16(bus, self.cs);
                        self.push_stack_16(bus, self.ip);
                        self.ip = offset;
                        self.cs = segment;
                    },
                    JumpType::IndIntersegment => {
                        self.push_stack_16(bus, self.cs);
                        self.push_stack_16(bus, self.ip);
                        let ip = bus.read_length(self, self.instr.segment, self.instr.offset, self.instr.data_length);
                        let cs = bus.read_length(self, self.instr.segment, self.instr.offset.wrapping_add(2), self.instr.data_length);
                        self.ip = ip;
                        self.cs = cs;
                    },
                    _ => unreachable!(),
                }
            },
            Opcode::JMP => {
                match self.instr.jump_type {
                    JumpType::DirWithinSegment(disp) => {self.ip = self.ip.wrapping_add(disp)},
                    JumpType::DirWithinSegmentShort(disp) => {self.ip = self.ip.wrapping_add(sign_extend(disp))},
                    JumpType::IndWithinSegment => {self.ip = self.get_val(bus, self.instr.operand1);},
                    JumpType::IndIntersegment => {
                        let ip = bus.read_length(self, self.instr.segment, self.instr.offset, self.instr.data_length);
                        let cs = bus.read_length(self, self.instr.segment, self.instr.offset.wrapping_add(2), self.instr.data_length);
                        self.ip = ip;
                        self.cs = cs;
                    },
                    JumpType::DirIntersegment(offset, segment) => {self.cs = segment; self.ip = offset},
                    _ => unreachable!(),
                }
            },
            Opcode::RET => {
                match self.instr.ret_type {
                    RetType::NearAdd(val) => {
                        self.ip = self.pop_stack_16(bus);
                        self.sp = self.sp.wrapping_add(val);
                    },
                    RetType::Near => {
                        self.ip = self.pop_stack_16(bus);
                    },
                    RetType::FarAdd(val) => {
                        self.ip = self.pop_stack_16(bus);
                        self.cs = self.pop_stack_16(bus);
                        self.sp = self.sp.wrapping_add(val);
                    },
                    RetType::Far => {
                        self.ip = self.pop_stack_16(bus);
                        self.cs = self.pop_stack_16(bus);
                    },
                    _ => unreachable!(),
                }
            },
            Opcode::JEJZ => self.jump(self.flags.z),
            Opcode::JLJNGE => self.jump(self.flags.s != self.flags.o),
            Opcode::JLEJNG => self.jump(self.flags.z | (self.flags.s != self.flags.o)),
            Opcode::JBJNAE => self.jump(self.flags.c),
            Opcode::JBEJNA => self.jump(self.flags.c | self.flags.z),
            Opcode::JPJPE => self.jump(self.flags.p),
            Opcode::JO => self.jump(self.flags.o),
            Opcode::JS => self.jump(self.flags.s),
            Opcode::JNEJNZ => self.jump(!self.flags.z),
            Opcode::JNLJGE => self.jump(self.flags.s == self.flags.o),
            Opcode::JNLEJG => self.jump(!self.flags.z & (self.flags.s == self.flags.o)),
            Opcode::JNBJAE => self.jump(!self.flags.c),
            Opcode::JNBEJA => self.jump(!self.flags.c & !self.flags.z),
            Opcode::JNPJPO => self.jump(!self.flags.p),
            Opcode::JNO => self.jump(!self.flags.o),
            Opcode::JNS => self.jump(!self.flags.s),
            Opcode::LOOP => {
                let cx = self.cx.get_x().wrapping_sub(1);
                self.cx.set_x(cx);
                self.jump(cx != 0);

                self.cycles += 1; // jump() ya suma lo demas
            },
            Opcode::LOOPZE => {
                let cx = self.cx.get_x().wrapping_sub(1);
                self.cx.set_x(cx);
                self.jump((cx != 0) & self.flags.z);

                self.cycles += 2; // jump() ya suma lo demas
            },
            Opcode::LOOPNZNE => {
                let cx = self.cx.get_x().wrapping_sub(1);
                self.cx.set_x(cx);
                self.jump((cx != 0) & !self.flags.z);

                self.cycles += 2; // jump() ya suma lo demas
            },
            Opcode::JCXZ => {
                self.jump(self.cx.get_x() == 0);

                self.cycles += 2; // jump() ya suma lo demas
            },
            Opcode::INT => {
                // self.interrupt(bus, self.sw_int_type);
                self.sw_int = true;
            },
            Opcode::INTO => {
                if self.flags.o {
                    // self.interrupt(bus, self.sw_int_type);
                    self.sw_int = true;
                    self.cycles += 73;
                } else {
                    self.cycles += 4;
                }
            },
            Opcode::IRET => {
                self.ip = self.pop_stack_16(bus);
                self.cs = self.pop_stack_16(bus);
                let flags = self.pop_stack_16(bus);
                self.flags.set_flags(flags)
            },

            Opcode::CLC => {
                self.flags.c = false;
            },
            Opcode::CMC => {
                self.flags.c = !self.flags.c;
            },
            Opcode::STC => {
                self.flags.c = true;
            },
            Opcode::CLD => {
                self.flags.d = false;
            },
            Opcode::STD => {
                self.flags.d = true;
            },
            Opcode::CLI => {
                self.flags.i = false;
            },
            Opcode::STI => {
                self.flags.i = true;
            },

            Opcode::HLT => {
                self.halted = true;
            },

            _ => {}
            // _ => unreachable!(),
        }
    }
}
