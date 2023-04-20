use super::Disassembler;
use crate::hardware::cpu_8088::{
    cpu_utils::{sign_extend, to_u16},
    instr_utils::{
        decode_jmp, decode_mod, decode_reg, decode_segment, Direction, Instruction, JumpType,
        Length, Opcode, Operand, OperandType, RepetitionPrefix, RetType, Segment,
    }, CPU,
};

impl Disassembler {
    pub fn decode(&mut self, instr: &mut Instruction, mem: &[u8], op: u8, cpu: &CPU) {
        match op {
            0x88..=0x8B => {
                instr.opcode = Opcode::MOV;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);

                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0xC6 | 0xC7 => {
                instr.opcode = Opcode::MOV;
                instr.data_length = Length::new(op, 0);

                let operand = self.fetch(mem);
                self.decode_mod_n_rm(instr, mem, operand);
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0xB0..=0xBF => {
                instr.opcode = Opcode::MOV;
                instr.data_length = Length::new(op, 3);
                instr.operand1 = decode_reg(op, 0, instr.data_length);
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0xA0..=0xA3 => {
                instr.opcode = Opcode::MOV;
                instr.data_length = Length::new(op, 0);

                self.read_imm_addres(instr, mem);

                if 0x02 & op == 0 {
                    if instr.data_length == Length::Byte {
                        instr.operand1 = OperandType::Register(Operand::AL);
                    } else {
                        instr.operand1 = OperandType::Register(Operand::AX);
                    }
                    instr.operand2 = OperandType::Memory(Operand::Disp(instr.offset));
                } else {
                    instr.operand1 = OperandType::Memory(Operand::Disp(instr.offset));
                    if instr.data_length == Length::Byte {
                        instr.operand2 = OperandType::Register(Operand::AL);
                    } else {
                        instr.operand2 = OperandType::Register(Operand::AX);
                    }
                }
            }
            0x8E | 0x8C => {
                instr.opcode = Opcode::MOV;
                instr.direction = Direction::new(op);
                instr.data_length = Length::Word;

                let operand = self.fetch(mem);
                instr.addr_mode = decode_mod(operand);

                match instr.direction {
                    Direction::ToReg => {
                        instr.operand1 = decode_segment(operand, 3);
                        instr.operand2 = self.decode_rm(instr, mem, operand, 0);
                    }
                    Direction::FromReg => {
                        instr.operand1 = self.decode_rm(instr, mem, operand, 0);
                        instr.operand2 = decode_segment(operand, 3);
                    }
                    _ => unreachable!(),
                }
            }
            // Prefijo _MISC r/m16
            0xFF => {
                let operand = self.fetch(mem);

                match operand & 0b00111000 {
                    0x0 => {
                        instr.opcode = Opcode::INC;
                        instr.data_length = Length::Word;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    0x8 => {
                        instr.opcode = Opcode::DEC;
                        instr.data_length = Length::Word;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    0x10 => {
                        instr.opcode = Opcode::CALL;
                        instr.data_length = Length::Word;
                        self.decode_mod_n_rm(instr, mem, operand);

                        let ip = self.get_val(cpu, instr, mem);
                        instr.jump_type = JumpType::IndWithinSegment(ip);
                    }
                    0x18 => {
                        instr.opcode = Opcode::CALL;
                        instr.data_length = Length::Word;
                        self.decode_mod_n_rm(instr, mem, operand);

                        let ip = self.read_length(
                            instr,
                            mem,
                            cpu.get_segment(instr.segment),
                            instr.offset,
                        );
                        let cs = self.read_length(
                            instr,
                            mem,
                            cpu.get_segment(instr.segment),
                            instr.offset.wrapping_add(2),
                        );

                        instr.jump_type = JumpType::IndIntersegment(cs, ip);
                    }
                    0x20 => {
                        instr.opcode = Opcode::JMP;
                        instr.data_length = Length::Word;
                        self.decode_mod_n_rm(instr, mem, operand);

                        let ip = self.get_val(cpu, instr, mem);

                        instr.jump_type = JumpType::IndWithinSegment(ip);
                    }
                    0x28 => {
                        instr.opcode = Opcode::JMP;
                        instr.data_length = Length::Word;
                        self.decode_mod_n_rm(instr, mem, operand);

                        let ip = self.read_length(
                            instr,
                            mem,
                            cpu.get_segment(instr.segment),
                            instr.offset,
                        );
                        let cs = self.read_length(
                            instr,
                            mem,
                            cpu.get_segment(instr.segment),
                            instr.offset.wrapping_add(2),
                        );


                        instr.jump_type = JumpType::IndIntersegment(cs, ip);
                    }
                    0x30 => {
                        instr.opcode = Opcode::PUSH;
                        instr.data_length = Length::Word;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    _ => log::info!("CAN'T DECODE {:02X} {:02X}", op, operand),
                }
            }
            0x50..=0x57 => {
                instr.opcode = Opcode::PUSH;
                instr.data_length = Length::Word;
                instr.operand1 = decode_reg(op, 0, instr.data_length);
            }
            0x06 | 0x16 | 0x0E | 0x1E => {
                instr.opcode = Opcode::PUSH;
                instr.data_length = Length::Word;
                instr.operand1 = decode_segment(op, 3);
            }
            0x8F => {
                instr.opcode = Opcode::POP;
                let operand = self.fetch(mem);
                instr.addr_mode = decode_mod(operand);
                instr.data_length = Length::Word;
                instr.operand1 = self.decode_rm(instr, mem, operand, 0);
            }
            0x58..=0x5F => {
                instr.opcode = Opcode::POP;
                instr.data_length = Length::Word;
                instr.operand1 = decode_reg(op, 0, instr.data_length);
            }
            0x07 | 0x17 | 0x0F | 0x1F => {
                instr.opcode = Opcode::POP;
                instr.data_length = Length::Word;
                instr.operand1 = decode_segment(op, 3);
            }
            0x86 | 0x87 => {
                instr.opcode = Opcode::XCHG;
                instr.data_length = Length::new(op, 0);
                instr.direction = Direction::ToReg;
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x90 => {
                instr.opcode = Opcode::NOP;
            }
            0x91..=0x97 => {
                instr.opcode = Opcode::XCHG;
                instr.data_length = Length::Word;
                instr.direction = Direction::ToReg;
                instr.operand1 = OperandType::Register(Operand::AX);
                instr.operand2 = decode_reg(op, 0, instr.data_length);
            }
            0xE4 | 0xE5 => {
                instr.opcode = Opcode::IN;
                instr.data_length = Length::new(op, 0);

                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.port = self.fetch(mem) as u16;
                instr.operand2 = OperandType::Memory(Operand::Disp(instr.port));
            }
            0xEC | 0xED => {
                instr.opcode = Opcode::IN;
                instr.data_length = Length::new(op, 0);

                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Memory(Operand::DX);
            }
            0xE6 | 0xE7 => {
                instr.opcode = Opcode::OUT;
                instr.data_length = Length::new(op, 0);
                instr.port = self.fetch(mem) as u16;
                instr.operand1 = OperandType::Memory(Operand::Disp(instr.port));

                instr.operand2 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
            }
            0xEE | 0xEF => {
                instr.opcode = Opcode::OUT;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = OperandType::Memory(Operand::DX);

                instr.operand2 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
            }
            0xD7 => {
                instr.opcode = Opcode::XLAT;
                instr.operand1 = OperandType::Register(Operand::AL);
            }
            0x8D => {
                instr.opcode = Opcode::LEA;
                instr.direction = Direction::ToReg;
                instr.data_length = Length::Word;
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0xC5 => {
                instr.opcode = Opcode::LDS;
                instr.direction = Direction::ToReg;
                instr.data_length = Length::Word;
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0xC4 => {
                instr.opcode = Opcode::LES;
                instr.direction = Direction::ToReg;
                instr.data_length = Length::Word;
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x9F => {
                instr.opcode = Opcode::LAHF;
            }
            0x9E => {
                instr.opcode = Opcode::SAHF;
            }
            0x9C => {
                instr.opcode = Opcode::PUSHF;
            }
            0x9D => {
                instr.opcode = Opcode::POPF;
            }

            0x00..=0x03 => {
                instr.opcode = Opcode::ADD;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand)
            }
            // Prefijos _ALU1
            0x80..=0x83 => {
                let operand = self.fetch(mem);
                instr.data_length = Length::new(op, 0);
                instr.addr_mode = decode_mod(operand);
                instr.operand1 = self.decode_rm(instr, mem, operand, 0);

                let val = match op & 0b00000011 {
                    0b00..=0b10 => self.read_imm(mem, instr),
                    0b11 => sign_extend(self.fetch(mem)),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(val);

                match operand & 0b00111000 {
                    0x00 => instr.opcode = Opcode::ADD,
                    0x08 => instr.opcode = Opcode::OR,
                    0x10 => instr.opcode = Opcode::ADC,
                    0x18 => instr.opcode = Opcode::SBB,
                    0x20 => instr.opcode = Opcode::AND,
                    0x28 => instr.opcode = Opcode::SUB,
                    0x30 => instr.opcode = Opcode::XOR,
                    0x38 => instr.opcode = Opcode::CMP,
                    _ => unreachable!(),
                }
            }
            0x04 | 0x05 => {
                instr.opcode = Opcode::ADD;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0x10..=0x13 => {
                instr.opcode = Opcode::ADC;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x14 | 0x15 => {
                instr.opcode = Opcode::ADC;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            // Prefijo _MISC r/m8
            0xFE => {
                let operand = self.fetch(mem);

                instr.data_length = Length::Byte;
                self.decode_mod_n_rm(instr, mem, operand);

                match operand & 0b00111000 {
                    0x00 => instr.opcode = Opcode::INC,
                    0x08 => instr.opcode = Opcode::DEC,
                    _ => unreachable!(),
                }
            }
            0x40..=0x47 => {
                instr.opcode = Opcode::INC;
                instr.data_length = Length::Word;
                instr.operand1 = decode_reg(op, 0, instr.data_length);
            }
            0x37 => {
                instr.opcode = Opcode::AAA;
            }
            0x27 => {
                instr.opcode = Opcode::DAA;
            }
            0x28..=0x2B => {
                instr.opcode = Opcode::SUB;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x2C | 0x2D => {
                instr.opcode = Opcode::SUB;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0x18..=0x1B => {
                instr.opcode = Opcode::SBB;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x1C | 0x1D => {
                instr.opcode = Opcode::SBB;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0x48..=0x4F => {
                instr.opcode = Opcode::DEC;
                instr.data_length = Length::Word;
                instr.operand1 = decode_reg(op, 0, instr.data_length);
            }
            // _ALU 2
            0xF6 | 0xF7 => {
                let operand = self.fetch(mem);
                instr.data_length = Length::new(op, 0);

                match operand & 0b00111000 {
                    0x00 | 0x08 => {
                        instr.opcode = Opcode::TEST;
                        self.decode_mod_n_rm(instr, mem, operand);
                        instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
                    }
                    0x10 => {
                        instr.opcode = Opcode::NOT;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    0x18 => {
                        instr.opcode = Opcode::NEG;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    0x20 => {
                        instr.opcode = Opcode::MUL;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    0x28 => {
                        instr.opcode = Opcode::IMUL;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    0x30 => {
                        instr.opcode = Opcode::DIV;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    0x38 => {
                        instr.opcode = Opcode::IDIV;
                        self.decode_mod_n_rm(instr, mem, operand);
                    }
                    _ => unreachable!(),
                }
            }
            0x38..=0x3B => {
                instr.opcode = Opcode::CMP;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x3C | 0x3D => {
                instr.opcode = Opcode::CMP;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0x3F => {
                instr.opcode = Opcode::AAS;
            }
            0x2F => {
                instr.opcode = Opcode::DAS;
            }
            0xD4 => {
                instr.opcode = Opcode::AAM;
                instr.data_length = Length::Byte;
                // instr.operand1 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0xD5 => {
                instr.opcode = Opcode::AAD;
                instr.data_length = Length::Byte;
                instr.operand1 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0x98 => {
                instr.opcode = Opcode::CBW;
            }
            0x99 => {
                instr.opcode = Opcode::CWD;
            }

            // _ROT 1
            0xD0..=0xD3 => {
                let operand = self.fetch(mem);

                instr.data_length = Length::new(op, 0);
                self.decode_mod_n_rm(instr, mem, operand);
                instr.operand2 = if op & 0x02 > 0 {
                    OperandType::Register(Operand::CL)
                } else {
                    OperandType::Immediate(1)
                };

                match operand & 0b00111000 {
                    0x00 => instr.opcode = Opcode::ROL,
                    0x08 => instr.opcode = Opcode::ROR,
                    0x10 => instr.opcode = Opcode::RCL,
                    0x18 => instr.opcode = Opcode::RCR,
                    0x20 => instr.opcode = Opcode::SALSHL,
                    0x28 => instr.opcode = Opcode::SHR,
                    0x30 => instr.opcode = Opcode::SALSHL,
                    0x38 => instr.opcode = Opcode::SAR,
                    _ => unreachable!(),
                }
            }
            0x20..=0x23 => {
                instr.opcode = Opcode::AND;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x24 | 0x25 => {
                instr.opcode = Opcode::AND;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0x08..=0x0B => {
                instr.opcode = Opcode::OR;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x0C | 0x0D => {
                instr.opcode = Opcode::OR;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0x30..=0x33 => {
                instr.opcode = Opcode::XOR;
                instr.direction = Direction::new(op);
                instr.data_length = Length::new(op, 0);
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0x34 | 0x35 => {
                instr.opcode = Opcode::XOR;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }
            0x84 | 0x85 => {
                instr.opcode = Opcode::TEST;
                instr.data_length = Length::new(op, 0);
                instr.direction = Direction::FromReg;
                let operand = self.fetch(mem);
                self.decode_mod_reg_rm(instr, mem, operand);
            }
            0xA8 | 0xA9 => {
                instr.opcode = Opcode::AND;
                instr.data_length = Length::new(op, 0);
                instr.operand1 = match instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                instr.operand2 = OperandType::Immediate(self.read_imm(mem, instr));
            }

            0x26 | 0x2E | 0x36 | 0x3E => {
                instr.segment = match op {
                    0x26 => Segment::ES,
                    0x2E => Segment::CS,
                    0x36 => Segment::SS,
                    0x3E => Segment::DS,
                    _ => unreachable!(),
                };

                let new_op = self.fetch(mem);
                self.decode(instr, mem, new_op, cpu);
            }

            0xF2 | 0xF3 => {
                instr.repetition_prefix = if op & 0x01 == 1 {
                    RepetitionPrefix::REPEZ
                } else {
                    RepetitionPrefix::REPNEZ
                };

                let new_op = self.fetch(mem);
                self.decode(instr, mem, new_op, cpu);
            }

            0xEA => {
                instr.opcode = Opcode::JMP;
                let offset_low = self.fetch(mem);
                let offset_high = self.fetch(mem);
                let seg_low = self.fetch(mem);
                let seg_high = self.fetch(mem);
                let offset = to_u16(offset_low, offset_high);
                let seg = to_u16(seg_low, seg_high);
                instr.jump_type = JumpType::DirIntersegment(offset, seg);
            }
            0xA4 => {
                instr.opcode = Opcode::MOVSB;
                instr.data_length = Length::Byte;
            }
            0xA5 => {
                instr.opcode = Opcode::MOVSW;
                instr.data_length = Length::Word;
            }
            0xA6 => {
                instr.opcode = Opcode::CMPSB;
                instr.data_length = Length::Byte;
            }
            0xA7 => {
                instr.opcode = Opcode::CMPSW;
                instr.data_length = Length::Word;
            }
            0xAE => {
                instr.opcode = Opcode::SCASB;
                instr.data_length = Length::Byte;
            }
            0xAF => {
                instr.opcode = Opcode::SCASW;
                instr.data_length = Length::Word;
            }
            0xAC => {
                instr.opcode = Opcode::LODSB;
                instr.data_length = Length::Byte;
            }
            0xAD => {
                instr.opcode = Opcode::LODSW;
                instr.data_length = Length::Word;
            }
            0xAA => {
                instr.opcode = Opcode::STOSB;
                instr.data_length = Length::Byte;
            }
            0xAB => {
                instr.opcode = Opcode::STOSW;
                instr.data_length = Length::Word;
            }

            0xE8 => {
                instr.opcode = Opcode::CALL;
                let val_low = self.fetch(mem);
                let val_high = self.fetch(mem);
                let val = to_u16(val_low, val_high);

                instr.jump_type = JumpType::DirWithinSegment(val);
            }
            0x9A => {
                instr.opcode = Opcode::CALL;

                let offset_low = self.fetch(mem);
                let offset_high = self.fetch(mem);
                let seg_low = self.fetch(mem);
                let seg_high = self.fetch(mem);
                let offset = to_u16(offset_low, offset_high);
                let seg = to_u16(seg_low, seg_high);
                instr.jump_type = JumpType::DirIntersegment(offset, seg);
            }
            0xE9 => {
                instr.opcode = Opcode::JMP;
                let val_low = self.fetch(mem);
                let val_high = self.fetch(mem);
                let val = to_u16(val_low, val_high);

                instr.jump_type = JumpType::DirWithinSegment(val);
            }
            0xEB => {
                instr.opcode = Opcode::JMP;
                let val = self.fetch(mem);

                instr.jump_type = JumpType::DirWithinSegmentShort(val);
            }
            0xC2 => {
                instr.opcode = Opcode::RET;
                let val = to_u16(self.fetch(mem), self.fetch(mem));
                instr.ret_type = RetType::NearAdd(val);
            }
            0xC3 => {
                instr.opcode = Opcode::RET;
                instr.ret_type = RetType::Near;
            }
            0xCA => {
                instr.opcode = Opcode::RET;
                let val = to_u16(self.fetch(mem), self.fetch(mem));
                instr.ret_type = RetType::FarAdd(val);
            }
            0xCB => {
                instr.opcode = Opcode::RET;
                instr.ret_type = RetType::Far;
            }
            0x74 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JEJZ, JumpType::DirWithinSegmentShort(val));
            }
            0x7C => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JLJNGE, JumpType::DirWithinSegmentShort(val));
            }
            0x7E => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JLEJNG, JumpType::DirWithinSegmentShort(val));
            }
            0x72 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JBJNAE, JumpType::DirWithinSegmentShort(val));
            }
            0x76 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JBEJNA, JumpType::DirWithinSegmentShort(val));
            }
            0x7A => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JPJPE, JumpType::DirWithinSegmentShort(val));
            }
            0x70 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JO, JumpType::DirWithinSegmentShort(val));
            }
            0x78 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JS, JumpType::DirWithinSegmentShort(val));
            }
            0x75 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JNEJNZ, JumpType::DirWithinSegmentShort(val));
            }
            0x7D => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JNLJGE, JumpType::DirWithinSegmentShort(val));
            }
            0x7F => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JNLEJG, JumpType::DirWithinSegmentShort(val));
            }
            0x73 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JNBJAE, JumpType::DirWithinSegmentShort(val));
            }
            0x77 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JNBEJA, JumpType::DirWithinSegmentShort(val));
            }
            0x7B => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JNPJPO, JumpType::DirWithinSegmentShort(val));
            }
            0x71 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JNO, JumpType::DirWithinSegmentShort(val));
            }
            0x79 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JNS, JumpType::DirWithinSegmentShort(val));
            }
            0xE2 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::LOOP, JumpType::DirWithinSegmentShort(val));
            }
            0xE1 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::LOOPZE, JumpType::DirWithinSegmentShort(val));
            }
            0xE0 => {
                let val = self.fetch(mem);
                decode_jmp(
                    instr,
                    Opcode::LOOPNZNE,
                    JumpType::DirWithinSegmentShort(val),
                );
            }
            0xE3 => {
                let val = self.fetch(mem);
                decode_jmp(instr, Opcode::JCXZ, JumpType::DirWithinSegmentShort(val));
            }

            0xCC => {
                instr.opcode = Opcode::INT;

                instr.sw_int_type = 3;
            }
            0xCD => {
                instr.opcode = Opcode::INT;

                instr.sw_int_type = self.fetch(mem);
            }
            0xCE => {
                instr.opcode = Opcode::INTO;

                instr.sw_int_type = 4;
            }
            0xCF => {
                instr.opcode = Opcode::IRET;
            }

            0xF8 => {
                instr.opcode = Opcode::CLC;
            }
            0xF5 => {
                instr.opcode = Opcode::CMC;
            }
            0xF9 => {
                instr.opcode = Opcode::STC;
            }
            0xFC => {
                instr.opcode = Opcode::CLD;
            }
            0xFD => {
                instr.opcode = Opcode::STD;
            }
            0xFA => {
                instr.opcode = Opcode::CLI;
            }
            0xFB => {
                instr.opcode = Opcode::STI;
            }
            0xF4 => {
                instr.opcode = Opcode::HLT;
            }

            _ => {
                // writeln!(&mut self.file, "Instrucción sin hacer: {:02X}", op).unwrap();
                log::info!("ERROR: {:02X}", op);
            } // _ => unreachable!(),
        }

        instr.ip = self.ip as u16;
        instr.cs = self.cs as u16;
    }
}
