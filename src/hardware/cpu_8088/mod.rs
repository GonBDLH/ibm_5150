pub mod cpu_utils;
mod decode;
mod execute;
pub mod instr_utils;
pub mod regs;

use super::{bus::Bus, peripheral::fdc_necupd765::FloppyDiskController};
use cpu_utils::*;
use instr_utils::*;
use regs::{Flags, GPReg};

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
    pub cycles: u32,

    pub nmi: bool,
    pub nmi_enabled: bool,
    // Controla de que tipo es la SW INT si existe
    pub sw_int: bool,

    pub halted: bool,

    // Usado en instrucciones de Strings cuando tengan que repetirse
    pub to_decode: bool,
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

            //#[cfg(feature = "tests")]
            #[cfg(test)]
            cs: 0xF000,
            //#[cfg(not(feature = "tests"))]
            #[cfg(not(test))]
            cs: 0xFFFF,
            ds: 0x0000,
            es: 0x0000,
            ss: 0x0000,

            //#[cfg(feature = "tests")]
            #[cfg(test)]
            ip: 0xFFF0,
            //#[cfg(not(feature = "tests"))]
            #[cfg(not(test))]
            ip: 0x0000,

            instr: Instruction::default(),

            cycles: 0,

            nmi: false,
            nmi_enabled: false,
            sw_int: false,

            halted: false,

            to_decode: true,
        }
    }
}

impl Default for CPU {
    fn default() -> Self {
        CPU::new()
    }
}

impl CPU {
    pub fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let dir = get_address(self);
        self.ip = self.ip.wrapping_add(1);
        bus.read_dir(dir)
    }

    // DEVUELVO LA IP PARA DEBUGEAR
    pub fn fetch_decode_execute(&mut self, bus: &mut Bus) -> (u32, u16) {
        self.cycles = 0;
        let ip = self.ip;

        if self.to_decode {
            self.instr = Instruction::default();
            let op = self.fetch(bus);
            self.decode(bus, op);
        }

        self.execute(bus);
        (self.cycles, ip)
    }

    pub fn handle_interrupts(
        &mut self,
        bus: &mut Bus,
        disk_ctrl: &mut FloppyDiskController,
        cycles: &mut u32,
    ) {
        if self.sw_int {
            #[cfg(not(test))]
            if self.instr.sw_int_type == 0x13 {
                disk_ctrl.int13(self, bus);
            // } else if self.instr.sw_int_type == 0x19 {
            //     disk_ctrl.int19(self, bus);
            } else {
                self.interrupt(bus, self.instr.sw_int_type as u16 * 0x04);
            }

            #[cfg(test)]
            self.interrupt(bus, self.instr.sw_int_type as u16 * 0x04);

            self.sw_int = false;
        } else if self.nmi && self.nmi_enabled {
            // Si hay una NON-MASKABLE INTERRUPT
            self.interrupt(bus, 0x0008);
            self.nmi = false;
            *cycles += 50;
        } else if self.flags.i {
            let pic_interrupt =  bus.pic.get_next();

            if pic_interrupt.is_some() {
                self.interrupt(bus, (pic_interrupt.unwrap() * 0x04) as u16);
                bus.pic.try_aeoi();
                *cycles += 61;
            }

        } else {
            // TODO ESTO IGUAL ESTA MAL
            self.nmi = false;
            self.sw_int = false;
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

    pub fn nmi_out(&mut self, val: u16) {
        self.nmi_enabled = if val == 0x80 {
            true
        } else if val == 0x00 {
            false
        } else {
            self.nmi_enabled
        };
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
            _ => unreachable!("Aqui no deberia entrar nunca"),
        }
    }

    pub fn set_reg(&mut self, length: Length, reg: Operand, val: u16) {
        match length {
            Length::Byte => self.set_reg8(reg, val as u8),
            Length::Word => self.set_reg16(reg, val),
            _ => unreachable!("Aqui no deberia entrar nunca"),
        }
    }

    pub fn get_reg(&self, reg: Operand) -> u16 {
        match reg {
            Operand::AX => self.ax.get_x(),
            Operand::CX => self.cx.get_x(),
            Operand::DX => self.dx.get_x(),
            Operand::BX => self.bx.get_x(),
            Operand::SP => self.sp,
            Operand::BP => self.bp,
            Operand::SI => self.si,
            Operand::DI => self.di,
            Operand::AL => self.ax.low as u16,
            Operand::CL => self.cx.low as u16,
            Operand::DL => self.dx.low as u16,
            Operand::BL => self.bx.low as u16,
            Operand::AH => self.ax.high as u16,
            Operand::CH => self.cx.high as u16,
            Operand::DH => self.dx.high as u16,
            Operand::BH => self.bx.high as u16,
            _ => unreachable!("Aqui no deberia entrar nunca"),
        }
    }

    pub fn get_segment(&self, segment: Segment) -> u16 {
        match segment {
            Segment::ES => self.es,
            Segment::CS => self.cs,
            Segment::SS => self.ss,
            Segment::DS => self.ds,
        }
    }

    pub fn set_segment(&mut self, segment: Segment, val: u16) {
        match segment {
            Segment::ES => self.es = val,
            Segment::CS => self.cs = val,
            Segment::SS => self.ss = val,
            Segment::DS => self.ds = val,
        }
    }

    fn get_val(&mut self, bus: &mut Bus, operand: OperandType) -> u16 {
        match operand {
            OperandType::Register(operand) => self.get_reg(operand),
            OperandType::SegmentRegister(operand) => self.get_segment(operand),
            OperandType::Immediate(imm) => imm,
            OperandType::Memory(_operand) => bus.read_length(
                self,
                self.instr.segment,
                self.instr.offset,
                self.instr.data_length,
            ),
            _ => unreachable!(),
        }
    }

    fn set_val(&mut self, bus: &mut Bus, operand: OperandType, val: u16) {
        match operand {
            OperandType::Register(operand) => self.set_reg(self.instr.data_length, operand, val),
            OperandType::SegmentRegister(operand) => self.set_segment(operand, val),
            OperandType::Memory(_operand) => bus.write_length(
                self,
                self.instr.data_length,
                self.instr.segment,
                self.instr.offset,
                val,
            ),
            _ => unreachable!(),
        }
    }

    fn push_stack_8(&mut self, bus: &mut Bus, val: u8) {
        self.sp = self.sp.wrapping_sub(1);
        bus.write_8(self.ss, self.sp, val);
    }

    fn push_stack_16(&mut self, bus: &mut Bus, val: u16) {
        let val = to_2u8(val);
        self.push_stack_8(bus, val.1);
        self.push_stack_8(bus, val.0);
    }

    fn pop_stack_8(&mut self, bus: &mut Bus) -> u8 {
        let val = bus.read_8(self.ss, self.sp);
        self.sp = self.sp.wrapping_add(1);
        val
    }

    fn pop_stack_16(&mut self, bus: &mut Bus) -> u16 {
        let val_low = self.pop_stack_8(bus);
        let val_high = self.pop_stack_8(bus);
        to_u16(val_low, val_high)
    }

    pub fn movs(&mut self, bus: &mut Bus) {
        let offset_from = self.si;
        let offset_to = self.di;

        let segment_from = self.instr.segment;
        let segment_to = Segment::ES;

        let val = bus.read_length(self, segment_from, offset_from, self.instr.data_length);
        bus.write_length(self, self.instr.data_length, segment_to, offset_to, val);
    }

    pub fn cmps(&mut self, bus: &mut Bus) {
        let offset_from = self.si;
        let offset_to = self.di;

        let segment_from = self.instr.segment;
        let segment_to = Segment::ES;

        let val1 = bus.read_length(self, segment_from, offset_from, self.instr.data_length);
        let val2 = bus.read_length(self, segment_to, offset_to, self.instr.data_length);
        let res = sub(val1, val2, self.instr.data_length);
        self.flags
            .set_sub_flags(self.instr.data_length, val1, val2, res.0, res.1);
    }

    pub fn scas(&mut self, bus: &mut Bus) {
        let offset_to = self.di;
        let segment_to = Segment::ES;

        let val1 = match self.instr.data_length {
            Length::Byte => self.ax.low as u16,
            Length::Word => self.ax.get_x(),
            _ => unreachable!(),
        };
        let val2 = bus.read_length(self, segment_to, offset_to, self.instr.data_length);
        let res = sub(val1, val2, self.instr.data_length);
        self.flags
            .set_sub_flags(self.instr.data_length, val1, val2, res.0, res.1);
    }

    pub fn lods(&mut self, bus: &mut Bus) {
        let offset_from = self.si;
        let segment_from = self.instr.segment;

        let val = bus.read_length(self, segment_from, offset_from, self.instr.data_length);

        match self.instr.data_length {
            Length::Byte => self.ax.low = val as u8,
            Length::Word => self.ax.set_x(val),
            _ => unreachable!(),
        };
    }

    pub fn stos(&mut self, bus: &mut Bus) {
        let offset_to = self.di;
        let segment_to = Segment::ES;

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

        match self.instr.opcode {
            Opcode::CMPSB | Opcode::CMPSW => {
                self.adjust_string_di(to_change);
                self.adjust_string_si(to_change);
            }
            Opcode::SCASB | Opcode::SCASW => {
                self.adjust_string_di(to_change);
            }
            Opcode::LODSB | Opcode::LODSW => {
                self.adjust_string_si(to_change);
            }
            Opcode::STOSB | Opcode::STOSW => {
                self.adjust_string_di(to_change);
            }
            Opcode::MOVSB | Opcode::MOVSW => {
                self.adjust_string_di(to_change);
                self.adjust_string_si(to_change);
            }
            _ => unreachable!(),
        }
    }

    pub fn adjust_string_di(&mut self, to_change: u16) {
        if !self.flags.d {
            self.di = self.di.wrapping_add(to_change);
        } else {
            self.di = self.di.wrapping_sub(to_change);
        }
    }

    pub fn adjust_string_si(&mut self, to_change: u16) {
        if !self.flags.d {
            self.si = self.si.wrapping_add(to_change);
        } else {
            self.si = self.si.wrapping_sub(to_change);
        }
    }

    pub fn check_z_str(&mut self) -> bool {
        match self.instr.repetition_prefix {
            RepetitionPrefix::REPEZ => self.flags.z,
            RepetitionPrefix::REPNEZ => !self.flags.z,
            _ => unreachable!(),
        }
    }

    pub fn jump_short(&mut self, cond: bool) {
        if cond {
            if let JumpType::DirWithinSegmentShort(disp) = self.instr.jump_type {
                self.ip = self.ip.wrapping_add(sign_extend(disp))
            } else {
                unreachable!()
            }
            self.cycles += 16;
        } else {
            self.cycles += 4;
        }
    }

    pub fn string_op(&mut self, bus: &mut Bus, f: fn(&mut CPU, &mut Bus), cycles: u32) {
        if self.instr.repetition_prefix == RepetitionPrefix::None {
            f(self, bus);
            self.adjust_string();
        } else if self.cx.get_x() == 0 {
            self.to_decode = true;
        } else {
            self.to_decode = false;

            self.cx.set_x(self.cx.get_x() - 1);
            f(self, bus);
            self.adjust_string();
            self.cycles = cycles;
        }
    }

    pub fn string_op_z(&mut self, bus: &mut Bus, f: fn(&mut CPU, &mut Bus), cycles: u32) {
        if self.instr.repetition_prefix == RepetitionPrefix::None {
            f(self, bus);
            self.adjust_string();
        } else if self.cx.get_x() == 0 {
            self.to_decode = true;
        } else {
            self.to_decode = false;

            self.cx.set_x(self.cx.get_x() - 1);
            f(self, bus);
            self.adjust_string();
            self.cycles = cycles;

            self.to_decode = !self.check_z_str();
        }
    }
}
