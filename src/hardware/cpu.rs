use std::fs::File;
use std::io::Write;

use super::bus::Bus;
use super::instr_utils::*;
use super::regs::{GPReg, Flags};
use super::cpu_utils::*;

pub struct CPU {
    // Registros de proposito general
    pub ax: GPReg,
    pub bx: GPReg,
    pub cx: GPReg,
    pub dx: GPReg,

    // Registros indices
    pub si: u16,
    pub di: u16,
    pub bp: u16,
    pub sp: u16,

    // Flags
    pub flags: Flags,

    // Registros de segmentos
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub ss: u16,

    // Instruction pointer
    pub ip: u16,
    
    // Utilizado para guardar info de la operacion que se esta decodificando
    pub instr: Instruction,

    // Archivo de logs (Igual hay que quitarlo de aqui)
    pub file: File,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            ax: GPReg::new(),
            bx: GPReg::new(),
            cx: GPReg::new(),
            dx: GPReg::new(),

            si: 0x0000,
            di: 0x0000,
            bp: 0x0000,
            sp: 0x0000,

            flags: Flags::new(),

            cs: 0xFFFF,
            ds: 0x0000,
            es: 0x0000,
            ss: 0x0000,

            ip: 0x0000,

            instr: Instruction::default(),

            file: File::create("logs/log.txt").unwrap()
        }
    }
}

impl CPU {
    pub fn step(self: &mut Self, bus: &mut Bus) {
        // 14,31818 MHz * 1/50 Hz / 3 ~= 95454 => Nº ciclos que hace la CPU en un frame 
        for _i in 0..95454 {
            if self.instr.cycles == 0 {
                self.fetch_decode_execute(bus)
            }
            
            self.instr.cycles -= 1;
            
            // self.instr_queue.push_back(read);
    
            // if self.pop {
            //     self.pop = false;
            //     self.instr = self.instr_queue.pop_front().expect("Error al extraer instruccion de la cola.");
            //     self.decode(bus);
            // }

            // self.instr_cycles -= 1;
            // if self.instr_cycles == 0 {
            //     self.pop = true;
            // }

        }
    }

    pub fn fetch(self: &mut Self, bus: &mut Bus) -> u8 {
        let dir = ((self.cs as usize) << 4) + self.ip as usize;
        self.ip = self.ip.wrapping_add(1);
        bus.read_dir(dir)
    }

    pub fn decode(&mut self, bus: &mut Bus,op: u8) {
        match op {
            0x88..=0x8B => {
                self.instr.opcode = Opcode::MOV;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 2,
                    (OperandType::Memory(_), OperandType::Register(_)) => 13 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 12 + self.instr.ea_cycles,
                    _ => unreachable!(),
                }
            },
            0xC6 | 0xC7 => {
                self.instr.opcode = Opcode::MOV;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand2 = OperandType::Immediate;

                let operand = self.fetch(bus);
                decode_mod_n_rm(self, bus, operand);
                read_imm(self, bus);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Memory(_), OperandType::Immediate) => 14 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Immediate) => 4,
                    _ => unreachable!(),
                }
            },
            0xB0..=0xBF => {
                self.instr.opcode = Opcode::MOV;
                self.instr.data_length = Length::new(op, 3);
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);
                self.instr.cycles = 4;
            },
            0xA0..=0xA3 => {
                self.instr.opcode = Opcode::MOV;
                self.instr.direction = if 0x02 & op == 0 {Direction::ToReg} else {Direction::FromReg};
                self.instr.data_length = Length::new(op, 0);
                if self.instr.direction == Direction::ToReg {
                    if self.instr.data_length == Length::Byte {
                        self.instr.operand1 = OperandType::Register(Operand::AL);
                    } else {
                        self.instr.operand1 = OperandType::Register(Operand::AX);
                    }
                    self.instr.operand2 = OperandType::Memory(Operand::Disp);
                } else {
                    self.instr.operand1 = OperandType::Memory(Operand::Disp);
                    if self.instr.data_length == Length::Byte {
                        self.instr.operand2 = OperandType::Register(Operand::AL);
                    } else {
                        self.instr.operand2 = OperandType::Register(Operand::AX);
                    }
                }
                read_imm_addres(self, bus);
                self.instr.cycles = 14;
            },
            0x8E | 0x8C => {
                self.instr.opcode = Opcode::MOV;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::Word;

                let operand = self.fetch(bus);
                self.instr.addr_mode = decode_mod(operand);

                match self.instr.direction {
                    Direction::ToReg => {
                        self.instr.operand1 = decode_segment(operand, 3);
                        self.instr.operand2 = decode_rm(self, bus, operand, 0);
                    },
                    Direction::FromReg => {
                        self.instr.operand1 = decode_rm(self, bus, operand, 0);
                        self.instr.operand2 = decode_segment(operand, 3);
                    },
                    _ => unreachable!(),
                }

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::SegmentRegister(_), OperandType::Register(_)) => 2,
                    (OperandType::Register(_), OperandType::Memory(_)) => 12 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::SegmentRegister(_)) => 2,
                    (OperandType::Memory(_), OperandType::Register(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                }
            },
            // Prefijo _MISC r/m16
            0xFF => {
                let operand = self.fetch(bus);

                match operand & 0b00111000 {
                    0x0 => {
                        self.instr.opcode = Opcode::INC;
                        self.instr.addr_mode = decode_mod(operand);
                        self.instr.data_length = Length::Word;
                        self.instr.operand1 = decode_rm(self, bus, operand, 0);

                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => 3,
                            OperandType::Memory(_) => 23 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    0x8 => {
                        self.instr.opcode = Opcode::DEC;
                        self.instr.addr_mode = decode_mod(operand);
                        self.instr.data_length = Length::Word;
                        self.instr.operand1 = decode_rm(self, bus, operand, 0);

                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => 3,
                            OperandType::Memory(_) => 23 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    0x10 => {
                        self.instr.opcode = Opcode::CALL;
                        self.instr.cycles = 1;
                    },
                    0x18 => {
                        self.instr.opcode = Opcode::CALL;
                        self.instr.cycles = 1;
                    },
                    0x20 => {
                        self.instr.opcode = Opcode::JMP;
                        self.instr.cycles = 1;
                    },
                    0x28 => {
                        self.instr.opcode = Opcode::JMP;
                        self.instr.cycles = 1;
                    },
                    0x30 => {
                        self.instr.opcode = Opcode::PUSH;
                        self.instr.data_length = Length::Word;
                        self.instr.addr_mode = decode_mod(operand);
                        self.instr.operand1 = decode_rm(self, bus, operand, 0);
                        
                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => 15,
                            OperandType::Memory(_) => 24 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        }
                    },
                    _ => unreachable!(),    
                }
            },
            0x50..=0x57 => {
                self.instr.opcode = Opcode::PUSH;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles = 15;
            },
            0x06 | 0x16 | 0x0E | 0x1E => {
                self.instr.opcode = Opcode::PUSH;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_segment(op, 3);
                self.instr.cycles = 14;
            },
            0x8F => {
                self.instr.opcode = Opcode::POP;
                let operand = self.fetch(bus);
                self.instr.addr_mode = decode_mod(operand);
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_rm(self, bus, operand, 0);

                self.instr.cycles = match self.instr.operand1 {
                    OperandType::Register(_) => 12,
                    OperandType::Memory(_) => 25 + self.instr.ea_cycles,
                    _ => unreachable!(),
                }
            },
            0x58..=0x5F => {
                self.instr.opcode = Opcode::POP;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles = 12;
            },
            0x07 | 0x17 | 0x0F | 0x1F => {
                self.instr.opcode = Opcode::POP;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_segment(op, 3);
                self.instr.cycles = 12;
            },
            0x86 | 0x87 => {
                self.instr.opcode = Opcode::XCHG;
                self.instr.data_length = Length::new(op, 0);
                self.instr.direction = Direction::ToReg;
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 4,
                    (OperandType::Register(_), OperandType::Memory(_)) => 25 + self.instr.ea_cycles,
                    (OperandType::Memory(_), OperandType::Register(_)) => 25 + self.instr.ea_cycles, // Creo que no hace falta, pero por si acaso
                    _ => unreachable!(),
                }
            },
            0x90..=0x97 => {
                self.instr.opcode = Opcode::XCHG;
                self.instr.data_length = Length::Word;
                self.instr.direction = Direction::ToReg;
                self.instr.operand1 = OperandType::Register(Operand::AX);
                self.instr.operand2 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles = 3;
            },
            // TODO IN
            0xE4 | 0xE5 => {
                self.instr.opcode = Opcode::IN;
                self.instr.cycles = 1;
            },
            0xEC | 0xED => {
                self.instr.opcode = Opcode::IN;
                self.instr.cycles = 1;
            },
            // TODO OUT
            0xE6 | 0xE7 => {
                self.instr.opcode = Opcode::OUT;
                self.instr.cycles = 1;
            },
            0xEE | 0xEF => {
                self.instr.opcode = Opcode::OUT;
                self.instr.cycles = 1;
            },
            0xD7 => {
                self.instr.opcode = Opcode::XLAT;
                self.instr.operand1 = OperandType::Register(Operand::AL);

                self.instr.cycles = 11;
            },
            0x8D => {
                self.instr.opcode = Opcode::LEA;
                self.instr.direction = Direction::ToReg;
                self.instr.data_length = Length::Word;
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);
                // TODO LEA
                self.instr.cycles = 1;
            },
            0xC5 => {
                // TODO LDS
                self.instr.opcode = Opcode::LDS;
                self.instr.cycles = 1;
            },
            0xC4 => {
                // TODO LES
                self.instr.opcode = Opcode::LES;
                self.instr.cycles = 1;
            },
            0x9F => {
                self.instr.opcode = Opcode::LAHF;
                self.instr.cycles = 4;
            },
            0x9E => {
                self.instr.opcode = Opcode::SAHF;
                self.instr.cycles = 4;
            },
            0x9C => {
                self.instr.opcode = Opcode::PUSHF;
                self.instr.cycles = 14;
            },
            0x9D => {
                self.instr.opcode = Opcode::POPF;
                self.instr.cycles = 12;
            },

            0x00..=0x03 => {
                self.instr.opcode = Opcode::ADD;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            // Prefijos _ALU1
            0x80..=0x83 => {
                let operand = self.fetch(bus);
                self.instr.data_length = Length::new(op, 0);
                self.instr.addr_mode = decode_mod(operand);
                self.instr.operand1 = decode_rm(self, bus, operand, 0);
                self.instr.operand2 = OperandType::Immediate;

                match op & 0b00000011 {
                    0b00..=0b10 => read_imm(self, bus),
                    0b11 => self.instr.imm = sign_extend(self.fetch(bus)),
                    _ => unreachable!(),
                }

                self.instr.cycles = match self.instr.operand1 {
                    OperandType::Register(_) => 4,
                    OperandType::Memory(_) => 23 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };

                match operand & 0b00111000 {
                    0x00 => self.instr.opcode = Opcode::ADD,
                    0x08 => self.instr.opcode = Opcode::OR,
                    0x10 => self.instr.opcode = Opcode::ADC,
                    0x18 => self.instr.opcode = Opcode::SBB,
                    0x20 => self.instr.opcode = Opcode::AND,
                    0x28 => self.instr.opcode = Opcode::SUB,
                    0x30 => self.instr.opcode = Opcode::XOR,
                    0x38 => self.instr.opcode = Opcode::CMP,
                    _ => unreachable!(),
                }
            },
            0x04 | 0x05 => {
                self.instr.opcode = Opcode::ADD;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles = 4;
            },
            0x10..=0x13 => {
                self.instr.opcode = Opcode::ADC;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x14 | 0x15 => {
                self.instr.opcode = Opcode::ADC;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles = 4;
            },
            // Prefijo _MISC r/m8
            0xFE => {
                let operand = self.fetch(bus);

                self.instr.addr_mode = decode_mod(operand);
                self.instr.data_length = Length::Byte;
                self.instr.operand1 = decode_rm(self, bus, operand, 0);

                self.instr.cycles = match self.instr.operand1 {
                    OperandType::Register(_) => 3,
                    OperandType::Memory(_) => 23 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };

                match operand & 0b00111000 {
                    0x00 => self.instr.opcode = Opcode::INC,
                    0x08 => self.instr.opcode = Opcode::DEC,
                    _ => unreachable!(),
                }
            },
            0x40..=0x47 => {
                self.instr.opcode = Opcode::INC;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles = 3;
            },
            0x37 => {
                self.instr.opcode = Opcode::AAA;
                self.instr.cycles = 8;
            },
            0x27 => {
                self.instr.opcode = Opcode::DAA;
                self.instr.cycles = 4;
            },
            0x28..=0x2B => {
                self.instr.opcode = Opcode::SUB;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x2C | 0x2D => {
                self.instr.opcode = Opcode::SUB;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles = 4;
            },
            0x18..=0x1B => {
                self.instr.opcode = Opcode::SBB;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x1C | 0x1D => {
                self.instr.opcode = Opcode::SBB;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles = 4;
            },
            // _ALU 2
            0xF6 | 0xF7 => {
                let operand = self.fetch(bus);
                self.instr.data_length = Length::new(op, 0);
                
                match operand & 0b00111000 {
                    
                    0x10 => {
                        self.instr.opcode = Opcode::NOT;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => 3,
                            OperandType::Memory(_) => 24 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        };
                    },
                    0x18 => {
                        self.instr.opcode = Opcode::NEG;
                        decode_mod_n_rm(self, bus, operand);

                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => 3,
                            OperandType::Memory(_) => 24 + self.instr.ea_cycles,
                            _ => unreachable!(),
                        };
                    },
                    0x20 => {
                        self.instr.opcode = Opcode::MUL;
                        decode_mod_n_rm(self, bus, operand);

                        // TODO no se si esto esta bien del todo
                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => if self.instr.data_length == Length::Byte {77} else {133},
                            OperandType::Memory(_) => if self.instr.data_length == Length::Byte {83 + self.instr.ea_cycles} else {139 + self.instr.ea_cycles},
                            _ => unreachable!(),
                        };
                    },
                    0x28 => {
                        self.instr.opcode = Opcode::IMUL;
                        decode_mod_n_rm(self, bus, operand);

                        // TODO no se si esto esta bien del todo
                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => if self.instr.data_length == Length::Byte {98} else {154},
                            OperandType::Memory(_) => if self.instr.data_length == Length::Byte {104 + self.instr.ea_cycles} else {160 + self.instr.ea_cycles},
                            _ => unreachable!(),
                        };
                    },
                    0x30 => {
                        self.instr.opcode = Opcode::DIV;
                        decode_mod_n_rm(self, bus, operand);

                        // TODO no se si esto esta bien del todo
                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => if self.instr.data_length == Length::Byte {90} else {162},
                            OperandType::Memory(_) => if self.instr.data_length == Length::Byte {96 + self.instr.ea_cycles} else {168 + self.instr.ea_cycles},
                            _ => unreachable!(),
                        };
                    }
                    0x38 => {
                        self.instr.opcode = Opcode::IDIV;
                        decode_mod_n_rm(self, bus, operand);

                        // TODO no se si esto esta bien del todo
                        self.instr.cycles = match self.instr.operand1 {
                            OperandType::Register(_) => if self.instr.data_length == Length::Byte {112} else {184},
                            OperandType::Memory(_) => if self.instr.data_length == Length::Byte {118 + self.instr.ea_cycles} else {190 + self.instr.ea_cycles},
                            _ => unreachable!(),
                        };
                    }
                    _ => unreachable!(),
                }
            },
            0x38..=0x3B => {
                self.instr.opcode = Opcode::CMP;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 13 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x3C | 0x3D => {
                self.instr.opcode = Opcode::CMP;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles = 4;
            },
            0x3F => {
                self.instr.opcode = Opcode::AAS;
                self.instr.cycles = 8;
            },
            0x2F => {
                self.instr.opcode = Opcode::DAS;
                self.instr.cycles = 4;
            },
            0xD4 => {
                self.instr.opcode = Opcode::AAM;
                self.instr.operand1 = OperandType::Immediate;
                self.instr.data_length = Length::Byte;
                read_imm(self, bus);
                self.instr.cycles = 83;
            },
            0xD5 => {
                self.instr.opcode = Opcode::AAD;
                self.instr.operand1 = OperandType::Immediate;
                self.instr.data_length = Length::Byte;
                read_imm(self, bus);
                self.instr.cycles = 60;
            },
            0x98 => {
                self.instr.opcode = Opcode::CBW;
                self.instr.cycles = 2;
            },
            0x99 => {
                self.instr.opcode = Opcode::CWD;
                self.instr.cycles = 5;
            },

            // _ROT 1
            0xD0..=0xD3 => {
                let operand = self.fetch(bus);

                self.instr.data_length = Length::new(op, 0);
                decode_mod_n_rm(self, bus, operand);
                self.instr.operand2 = if operand & 0x02 == 1 {
                    OperandType::Register(Operand::CL)
                } else {
                    self.instr.imm = 1;
                    OperandType::Immediate
                };

                match operand & 0b00111000 {
                    0x00 => self.instr.opcode = Opcode::ROL,
                    0x08 => self.instr.opcode = Opcode::ROR,
                    0x10 => self.instr.opcode = Opcode::RCL,
                    0x18 => self.instr.opcode = Opcode::RCR,
                    0x20 => self.instr.opcode = Opcode::SALSHL,
                    0x28 => self.instr.opcode = Opcode::SHR,
                    0x30 => self.instr.opcode = Opcode::SALSHL,
                    0x38 => self.instr.opcode = Opcode::SAR,
                    _ => unreachable!(),
                }
                
                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Immediate) => 2,
                    (OperandType::Memory(_), OperandType::Immediate) => 23 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Register(_)) => 8,
                    (OperandType::Memory(_), OperandType::Register(_)) => 28 + self.instr.ea_cycles + 4 * self.cx.low,
                    _ => unreachable!(),
                };

            },
            0x20..=0x23 => {
                self.instr.opcode = Opcode::AND;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x24 | 0x25 => {
                self.instr.opcode = Opcode::AND;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles = 4;
            },
            0x08..=0x0B => {
                self.instr.opcode = Opcode::OR;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x0C | 0x0D => {
                self.instr.opcode = Opcode::OR;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles = 4;
            },
            0x30..=0x33 => {
                self.instr.opcode = Opcode::XOR;
                self.instr.direction = Direction::new(op);
                self.instr.data_length = Length::new(op, 0);
                let operand = self.fetch(bus);
                decode_mod_reg_rm(self, bus, operand);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Register(_), OperandType::Register(_)) => 3,
                    (OperandType::Memory(_), OperandType::Register(_)) => 24 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Memory(_)) => 13 + self.instr.ea_cycles,
                    _ => unreachable!(),
                };
            },
            0x34 | 0x35 => {
                self.instr.opcode = Opcode::XOR;
                self.instr.data_length = Length::new(op, 0);
                self.instr.operand1 = match self.instr.data_length {
                    Length::Byte => OperandType::Register(Operand::AL),
                    Length::Word => OperandType::Register(Operand::AX),
                    _ => unreachable!(),
                };
                self.instr.operand2 = OperandType::Immediate;
                read_imm(self, bus);

                self.instr.cycles = 4;
            },

            0xEA => {
                self.instr.opcode = Opcode::JMP;
                let offset_low = self.fetch(bus);
                let offset_high = self.fetch(bus);
                let seg_low = self.fetch(bus);
                let seg_high = self.fetch(bus);
                let offset = to_u16(offset_low, offset_high);
                let seg = to_u16(seg_low, seg_high);
                self.instr.jump_type = JumpType::Long(offset, seg);

                self.instr.cycles = 15;
            }
            _ => {
                writeln!(&mut self.file, "Instrucción sin hacer: {:02X}", op).unwrap();

                self.instr.cycles = 3
            }
            // _ => unreachable!(),
        }
    }

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

            },
            Opcode::OUT => {

            },
            Opcode::XLAT => {
                let val = bus.read_8(self.get_segment(Operand::DS), self.get_reg16(Operand::BX) + self.get_reg8(Operand::AL));
                self.set_reg8(Operand::AL, val);
            },
            Opcode::LEA => {
                let val = self.get_val(bus, self.instr.operand2);
                self.set_val(bus, self.instr.operand1, val);
            }
            Opcode::LDS => {

            },
            Opcode::LES => {

            },
            Opcode::LAHF => {
                self.ax.high = self.flags.get_flags() as u8;
            },
            Opcode::SAHF => {
                self.flags.set_flags(self.ax.high as u16);
            },
            Opcode::PUSHF => {
                let val = self.flags.get_flags();
                self.push_stack(bus, val);
            },
            Opcode::POPF => {
                let val = self.pop_stack(bus);
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
                self.ax.high = temp_al / self.instr.imm as u8;
                self.ax.low = temp_al % self.instr.imm as u8;

                self.flags.set_aam_flags(self.ax.low);
            },
            Opcode::DIV => {
                let val2 = self.get_val(bus, self.instr.operand1);

                if val2 == 0 {
                    // TODO int 0
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
                    // TODO int 0
                    return;
                }

                match self.instr.data_length {
                    Length::Byte => {
                        let val1 = self.ax.low as i8 as i16;
                        let res = val1.wrapping_div(val2);
                        if res > 0x7F || -res > 0x80 {
                            // TODO int 0
                        } else {
                            self.set_reg(self.instr.data_length, Operand::AL, res as u16);
                            self.set_reg(self.instr.data_length, Operand::AH, val1.wrapping_rem(val2) as u16);
                        }
                    },
                    Length::Word => {
                        let val1 = to_u32(self.ax.get_x(), self.dx.get_x()) as i32;
                        let res = val1.wrapping_div(val2 as i32);
                        if res > 0x7FFF || -res > 0x8000 {
                            // TODO int 0
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
                self.ax.low = (temp_al + (temp_ah.wrapping_add(self.instr.imm as u8))) & 0xFF;

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
            }

            Opcode::JMP => {
                match self.instr.jump_type {
                    // TODO JMP SHORT
                    JumpType::Long(offset, segment) => {self.cs = segment; self.ip = offset},
                    _ => unreachable!(),
                }
            }
            _ => {}
            // _ => unreachable!(),
        }
    }

    pub fn fetch_decode_execute(&mut self, bus: &mut Bus) {
        self.instr = Instruction::default();
        let op = self.fetch(bus);
        self.decode(bus, op);
        self.execute(bus);
    }
}

// Utilidades para el set de instrucciones
impl CPU {
    fn set_reg8(&mut self, reg: Operand, val: u8) {
        match reg {
            Operand::AL => self.ax.low = val,
            Operand::CL => self.cx.low = val,
            Operand::DL => self.dx.low = val,
            Operand::BL => self.bx.low = val,
            Operand::AH => self.ax.high = val,
            Operand::CH => self.cx.high = val,
            Operand::DH => self.dx.high = val,
            Operand::BH => self.bx.high = val,
            _ => unreachable!(),
        }
    }

    fn set_reg16(&mut self, reg: Operand, val: u16) {
        match reg {
            Operand::AX => self.ax.set_x(val),
            Operand::CX => self.cx.set_x(val),
            Operand::DX => self.dx.set_x(val),
            Operand::BX => self.bx.set_x(val),
            Operand::SP => self.sp = val,
            Operand::BP => self.bp = val,
            Operand::SI => self.si = val,
            Operand::DI => self.di = val,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    pub fn set_reg(self: &mut Self, length: Length, reg: Operand, val: u16) {

        match length {
            Length::Byte => self.set_reg8(reg, val as u8),
            Length::Word => self.set_reg16(reg, val),
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn get_reg8(self: &Self, reg: Operand) -> u16 {
        match reg {
            Operand::AL => self.ax.low as u16,
            Operand::CL => self.cx.low as u16,
            Operand::DL => self.dx.low as u16,
            Operand::BL => self.bx.low as u16,
            Operand::AH => self.ax.high as u16,
            Operand::CH => self.cx.high as u16,
            Operand::DH => self.dx.high as u16,
            Operand::BH => self.bx.high as u16,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn get_reg16(self: &Self, reg: Operand) -> u16 {
        match reg {
            Operand::AX => self.ax.get_x(),
            Operand::CX => self.cx.get_x(),
            Operand::DX => self.dx.get_x(),
            Operand::BX => self.bx.get_x(),
            Operand::SP => self.sp,
            Operand::BP => self.bp,
            Operand::SI => self.si,
            Operand::DI => self.di,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    pub fn get_reg(&mut self, length: Length, reg: Operand) -> u16 {
        match length {
            Length::Byte => self.get_reg8(reg),
            Length::Word => self.get_reg16(reg),
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    pub fn get_segment(&self, segment: Operand) -> u16 {
        match segment {
            Operand::ES => self.es,
            Operand::CS => self.cs,
            Operand::SS => self.ss,
            Operand::DS => self.ds,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    pub fn set_segment(&mut self, segment: Operand, val: u16) {
        match segment {
            Operand::ES => self.es = val,
            Operand::CS => self.cs = val,
            Operand::SS => self.ss = val,
            Operand::DS => self.ds = val,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn get_val(&mut self, bus: &mut Bus, operand: OperandType) -> u16 {
        match operand {
            OperandType::Register(operand) => self.get_reg(self.instr.data_length, operand),
            OperandType::SegmentRegister(operand) => self.get_segment(operand),
            OperandType::Immediate => self.instr.imm,
            OperandType::Memory(_operand) => bus.read_length(self, self.instr.segment, self.instr.offset, self.instr.data_length),
            _ => unreachable!(),
        }
    }

    fn set_val(&mut self, bus: &mut Bus, operand: OperandType, val: u16) {
        match operand {
            OperandType::Register(operand) => self.set_reg(self.instr.data_length, operand, val),
            OperandType::SegmentRegister(operand) => self.set_segment(operand, val),
            OperandType::Memory(_operand) => bus.write_length(self, self.instr.data_length, self.instr.segment, self.instr.offset, val),
            _ => unreachable!(),
        }
    }

    fn push_stack_8(self: &mut Self, bus: &mut Bus, val: u8) {
        bus.write_8(self.ss, self.sp, val);
        self.sp = self.sp.wrapping_add(1);
    }

    fn push_stack_16(self: &mut Self, bus: &mut Bus, val: u16) {
        let val = to_2u8(val);
        self.push_stack_8(bus, val.0);
        self.push_stack_8(bus, val.1);
    }

    fn push_stack(&mut self, bus: &mut Bus, val: u16) {
        match self.instr.data_length {
            Length::Byte => self.push_stack_8(bus, val as u8),
            Length::Word => self.push_stack_16(bus, val),
            _ => unreachable!(),
        }
    }

    fn pop_stack_8(self: &mut Self, bus: &mut Bus) -> u8 {
        let val = bus.read_8(self.ss, self.sp);
        self.sp = self.sp.wrapping_sub(1);
        val
    }

    fn pop_stack_16(self: &mut Self, bus: &mut Bus) -> u16 {
        let val_high = self.pop_stack_8(bus);
        let val_low = self.pop_stack_8(bus);
        to_u16(val_low, val_high)
    }

    fn pop_stack(&mut self, bus: &mut Bus) -> u16 {
        match self.instr.data_length {
            Length::Byte => self.pop_stack_8(bus) as u16,
            Length::Word => self.pop_stack_16(bus),
            _ => unreachable!(),
        }
    }
}

