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
    flags: Flags,

    // Registros de segmentos
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub ss: u16,

    // Instruction pointer
    ip: u16,
    
    // Utilizado para guardar info de la operacion que se esta decodificando
    pub instr: Instruction,
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

            cs: 0x0000,
            ds: 0x0000,
            es: 0x0000,
            ss: 0x0000,

            ip: 0x0000,

            instr: Instruction::default(),
        }
    }
}

impl CPU {
    pub fn step(self: &mut Self, bus: &mut Bus) {
        // 14,31818 MHz * 1/50 Hz / 3 ~= 95454 => Nº ciclos que hace la CPU en un frame 
        for _i in 0..95454 {
            if self.instr.cycles == 0 {
                self.fetch_decode_execute(bus)
            } else {
                self.instr.cycles -= 1;
            }
            
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
        // self.ip += 1;
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
                self.instr.operand2 = OperandType::Immediate(Operand::Imm);

                let operand = self.fetch(bus);
                decode_mod_n_rm(self, bus, operand);
                read_imm(self, bus);

                self.instr.cycles = match (self.instr.operand1, self.instr.operand2) {
                    (OperandType::Memory(_), OperandType::Immediate(_)) => 14 + self.instr.ea_cycles,
                    (OperandType::Register(_), OperandType::Immediate(_)) => 4,
                    _ => unreachable!(),
                }
            },
            0xB0..=0xBF => {
                self.instr.opcode = Opcode::MOV;
                self.instr.data_length = Length::new(op, 3);
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.operand2 = OperandType::Immediate(Operand::Imm);
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
                    },
                    0x8 => {
                        self.instr.opcode = Opcode::DEC;
                    },
                    0x10 => {
                        self.instr.opcode = Opcode::CALL;
                    },
                    0x18 => {
                        self.instr.opcode = Opcode::CALL;
                    },
                    0x20 => {
                        self.instr.opcode = Opcode::JMP;
                    },
                    0x28 => {
                        self.instr.opcode = Opcode::JMP;
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
            }

            0x50..=0x57 => {
                self.instr.opcode = Opcode::PUSH;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_reg(op, 0, self.instr.data_length);
                self.instr.cycles = 15;
            }

            0x06 | 0x16 | 0x0E | 0x1E => {
                self.instr.opcode = Opcode::PUSH;
                self.instr.data_length = Length::Word;
                self.instr.operand1 = decode_segment(op, 3);
                self.instr.cycles = 14;
            }
            _ => {}
            // _ => unreachable!(),
        }
    }

    pub fn execute(&mut self, bus: &mut Bus) {
        match self.instr.opcode {
            Opcode::MOV => {
                let val = self.get_val(bus, self.instr.operand2);

                match self.instr.operand1 {
                    OperandType::Register(operand) => self.set_reg(self.instr.data_length, operand, val),
                    OperandType::SegmentRegister(operand) => self.set_segment(operand, val),
                    OperandType::Memory(_operand) => bus.write_length(self, self.instr.data_length, self.instr.segment, self.instr.offset, val),
                    _ => unreachable!(),
                }
            },

            Opcode::PUSH => {
                let val = self.get_val(bus, self.instr.operand1);
                self.push_stack(bus, val);
            }
            _ => {}
            // _ => unreachable!(),
        }
    }

    pub fn fetch_decode_execute(&mut self, bus: &mut Bus) {
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
            OperandType::Immediate(_operand) => self.instr.imm,
            OperandType::Memory(_operand) => bus.read_length(self, self.instr.segment, self.instr.offset, self.instr.data_length),
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
}

