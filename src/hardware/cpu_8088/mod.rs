pub mod instr_utils;
pub mod cpu_utils;
pub mod regs;
mod decode;
mod execute;

pub mod dissasemble;

// use std::fs::File;

use super::bus::Bus;
use instr_utils::*;
use regs::{GPReg, Flags};
use cpu_utils::*;

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

    // Ciclos que se ha ejecutado una instr.
    pub cycles: u64,

    // Interrupciones
    pub intr: bool,
    pub intr_type: u8,
    pub nmi: bool,
    // Controla de que tipo es la SW INT si existe
    pub sw_int: bool,
    pub sw_int_type: u8,

    pub halted: bool,

    // Archivo de logs (Igual hay que quitarlo de aqui)
    // pub file: File,
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

            cycles: 0,

            intr: false,
            intr_type: 0,
            nmi: false,
            sw_int: false,
            sw_int_type: 0,

            halted: false,

            // file: File::create("logs/log.txt").unwrap(),
        }
    }
}

impl CPU {
    // pub fn step(self: &mut Self, bus: &mut Bus) {
    //     // 14,31818 MHz * 1/50 Hz / 3 ~= 95454 => Nº ciclos que hace la CPU en un frame 
    //     for _i in 0..95454 {
    //         if self.ip == 0xE0EA {
    //             let _a = 0;
    //         }
    //         if self.cycles == 0 {
    //             if self.halted {
    //                 return; // TODO
    //             }
    //             self.fetch_decode_execute(bus);
    //             let _a = 0;
    //         }


    //         self.cycles -= 1;

    //         self.hw_interrup(bus);
    //     }
    // }

    pub fn fetch(self: &mut Self, bus: &mut Bus) -> u8 {
        let dir = ((self.cs as usize) << 4) + self.ip as usize;
        self.ip = (self.ip as u32 + 1) as u16;
        bus.read_dir(dir)
    }

    pub fn fetch_decode_execute(&mut self, bus: &mut Bus) -> u64 {
        // if self.ip == 0xE0A9 {
        //     let _a = 0;
        // }

        self.cycles = 0;
        // self.instr = Instruction::default();
        let op = self.fetch(bus);
        self.decode(bus, op);
        self.execute(bus);
        self.cycles
    }

    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        if self.flags.i && self.sw_int {
            self.interrupt(bus, (self.sw_int_type * 0x04) as u16);
            self.sw_int = false;
        } else if self.nmi {
            // Si hay una NON-MASKABLE INTERRUPT
            self.interrupt(bus, 0x0008);
            self.nmi = false;
        } else if self.flags.i && self.intr {
            self.interrupt(bus, (self.intr_type * 0x04) as u16);
            self.intr = false;
        } 
    }

    pub fn interrupt(&mut self, bus: &mut Bus, ip_location: u16) {
        self.push_stack_16(bus, self.flags.get_flags());
        self.push_stack_16(bus, self.cs);
        self.push_stack_16(bus, self.ip);

        self.ip = bus.read_16(0, ip_location);
        self.cs = bus.read_16(0, ip_location + 2);
        
        self.flags.i = false;
        self.flags.t = false;
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
            Operand::None => 0,
            _ => unreachable!(),
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
            OperandType::Immediate(imm) => imm,
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
        self.sp = self.sp.wrapping_sub(1);
        bus.write_8(self.ss, self.sp, val);
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
        self.sp = self.sp.wrapping_add(1);
        val
    }

    fn pop_stack_16(self: &mut Self, bus: &mut Bus) -> u16 {
        let val_low = self.pop_stack_8(bus);
        let val_high = self.pop_stack_8(bus);
        to_u16(val_low, val_high)
    }

    fn pop_stack(&mut self, bus: &mut Bus) -> u16 {
        match self.instr.data_length {
            Length::Byte => self.pop_stack_8(bus) as u16,
            Length::Word => self.pop_stack_16(bus),
            _ => unreachable!(),
        }
    }

    pub fn movs(&mut self, bus: &mut Bus) {
        let offset_from = self.si;
        let offset_to = self.di;
    
        let segment_from = if self.instr.segment == Operand::None {
            Operand::DS
        } else {
            self.instr.segment
        };
        let segment_to = Operand::ES;
    
        let val = bus.read_length(self, segment_from, offset_from, self.instr.data_length);
        bus.write_length(self, self.instr.data_length, segment_to, offset_to, val);
    }
    
    pub fn cmps(&mut self, bus: &mut Bus) {
        let offset_from = self.si;
        let offset_to = self.di;
    
        let segment_from = if self.instr.segment == Operand::None {
            Operand::DS
        } else {
            self.instr.segment
        };
        let segment_to = Operand::ES;
    
        let val1 = bus.read_length(self, segment_from, offset_from, self.instr.data_length);
        let val2 = bus.read_length(self, segment_to, offset_to, self.instr.data_length);
        let res = val1.wrapping_sub(val2);
        self.flags.set_sub_flags(self.instr.data_length, val1, val2, res);
    }
    
    pub fn scas(&mut self, bus: &mut Bus) {
        let offset_to = self.di;
        let segment_to = Operand::ES;
    
        let val1 = match self.instr.data_length {
            Length::Byte => self.ax.low as u16,
            Length::Word => self.ax.get_x(),
            _ => unreachable!()
        };
        let val2 = bus.read_length(self, segment_to, offset_to, self.instr.data_length);
        let res = val1.wrapping_sub(val2);
        self.flags.set_sub_flags(self.instr.data_length, val1, val2, res);
    }
    
    pub fn lods(&mut self, bus: &mut Bus) {
        let offset_from = self.si;
        let segment_from = if self.instr.segment == Operand::None {
            Operand::DS
        } else {
            self.instr.segment
        };
    
        let val = bus.read_length(self, segment_from, offset_from, self.instr.data_length);
    
        match self.instr.data_length {
            Length::Byte => self.ax.low = val as u8,
            Length::Word => self.ax.set_x(val),
            _ => unreachable!()
        };
    }
    
    pub fn stos(&mut self, bus: &mut Bus) {
        let offset_to = self.di;
        let segment_to = Operand::ES;
    
        let val = match self.instr.data_length {
            Length::Byte => self.ax.low as u16,
            Length::Word => self.ax.get_x(),
            _ => unreachable!(),
        };
    
        bus.write_length(self, self.instr.data_length, segment_to, offset_to, val);
    }
    
    pub fn adjust_string(&mut self) {
        let to_change = match self.instr.data_length {
            Length::Byte => 1,
            Length::Word => 2,
            _ => unreachable!(),
        };
    
        if !self.flags.d {
            self.si = self.si.wrapping_add(to_change);
            self.di = self.di.wrapping_add(to_change);
        } else {
            self.si = self.si.wrapping_sub(to_change);
            self.di = self.di.wrapping_sub(to_change);
        }
    }
    
    pub fn adjust_string_di(&mut self) {
        let to_change = match self.instr.data_length {
            Length::Byte => 1,
            Length::Word => 2,
            _ => unreachable!(),
        };
    
        if !self.flags.d {
            self.di = self.di.wrapping_add(to_change);
        } else {
            self.di = self.di.wrapping_sub(to_change);
        }
    }
    
    pub fn adjust_string_si(&mut self) {
        let to_change = match self.instr.data_length {
            Length::Byte => 1,
            Length::Word => 2,
            _ => unreachable!(),
        };
    
        if !self.flags.d {
            self.di = self.si.wrapping_add(to_change);
        } else {
            self.di = self.si.wrapping_sub(to_change);
        }
    }
    
    pub fn check_z_str(&mut self) -> bool {
        match self.instr.repetition_prefix {
            RepetitionPrefix::REPEZ => {
                self.flags.z
            },
            RepetitionPrefix::REPNEZ => {
                !self.flags.z
            },
            _ => unreachable!()
        }
    }
    
    pub fn jump(&mut self, cond: bool) {
        if cond {
            if let JumpType::DirWithinSegmentShort(disp) = self.instr.jump_type {
                self.ip = self.ip.wrapping_add(sign_extend(disp))
            }
            self.cycles += 16;
        } else {
            self.cycles += 4;
        }
    }
}
