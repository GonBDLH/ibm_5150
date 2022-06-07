use num::ToPrimitive;

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
    cs: u16,
    ds: u16,
    es: u16,
    ss: u16,

    // Instruction pointer
    ip: u16,
    
    // Utilizado para guardar info de la operacion que se esta decodificando
    pop: bool,
    pub op: Instruction,
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

            pop: true,
            op: Instruction::default(),
        }
    }
}

impl CPU {
    pub fn step(self: &mut Self, bus: &mut Bus) {
        // 14,31818 MHz * 1/50 Hz / 3 ~= 95454 => Nº ciclos que hace la CPU en un frame 
        for _i in 0..95454 {
            self.fetch_decode_execute(bus)
            // self.instr_queue.push_back(read);
    
            // if self.pop {
            //     self.pop = false;
            //     self.op = self.instr_queue.pop_front().expect("Error al extraer instruccion de la cola.");
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

    fn fetch_decode_execute(&mut self, bus: &mut Bus) {
        let op = self.fetch(bus);

        match op {
            // 0x88..=0x8B | 0xC6 | 0xC7 | 0xB0..=0xBF | 0xA0..=0xA3 | 0x8E | 0x8C => {
            //     self.op.opcode = Opcode::MOV;
            // }
            0x88..=0x8B => {
                self.op.opcode = Opcode::MOV;
                self.op.direction = Direction::new(op);
                self.op.data_length = Length::new(op);
                decode_mod_reg_rm(self, bus);

                match self.op.data_length {
                    Length::Byte => {
                        
                    },
                    Length::Word => {
                        
                    },
                    _ => unreachable!(),
                }
            }

            _ => unreachable!(),
        }
    }

    // fn decode(self: &mut Self, bus: &mut Bus) {
    //     match self.op {
    //         0x00 => {
    //             self.add_rm8_r8(bus);
    //         },
    //         0x01 => {
    //             self.add_rm16_r16(bus);
    //         },
    //         0x02 => {
    //             self.add_r8_rm8(bus);
    //         },
    //         0x03 => {
    //             self.add_r16_rm16(bus);
    //         },
    //         0x04 => {
    //             self.add_al_d8(bus);
    //         },
    //         0x05 => {
    //             self.add_ax_d16(bus);
    //         },
    //         0x06 => {
    //             self.push_sr(bus);
    //         },
    //         0x07 => {
    //             self.pop_sr(bus);
    //         }
    //         _ => unreachable!("Instruccion no reconocida {:02X}.", self.op)
    //     }
    // }
}

// Utilidades para el set de instrucciones
impl CPU {
    fn set_reg<T: ToPrimitive>(self: &mut Self, reg: Operand, val: T) {
        match reg {
            Operand::AL => self.ax.low = val.to_u8().unwrap(),
            Operand::CL => self.cx.low = val.to_u8().unwrap(),
            Operand::DL => self.dx.low = val.to_u8().unwrap(),
            Operand::BL => self.bx.low = val.to_u8().unwrap(),
            Operand::AH => self.ax.high = val.to_u8().unwrap(),
            Operand::CH => self.cx.high = val.to_u8().unwrap(),
            Operand::DH => self.dx.high = val.to_u8().unwrap(),
            Operand::BH => self.bx.high = val.to_u8().unwrap(),
            Operand::AX => self.ax.set_x(val.to_u16().unwrap()),
            Operand::CX => self.cx.set_x(val.to_u16().unwrap()),
            Operand::DX => self.dx.set_x(val.to_u16().unwrap()),
            Operand::BX => self.bx.set_x(val.to_u16().unwrap()),
            Operand::SP => self.sp = val.to_u16().unwrap(),
            Operand::BP => self.bp = val.to_u16().unwrap(),
            Operand::SI => self.si = val.to_u16().unwrap(),
            Operand::DI => self.di = val.to_u16().unwrap(),
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn get_reg8(self: &Self, reg: Operand) -> u8 {
        match reg {
            Operand::AL => self.ax.low,
            Operand::CL => self.cx.low,
            Operand::DL => self.dx.low,
            Operand::BL => self.bx.low,
            Operand::AH => self.ax.high,
            Operand::CH => self.cx.high,
            Operand::DH => self.dx.high,
            Operand::BH => self.bx.high,
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

    fn get_segment(self: &Self, reg: u8) -> u16 {
        match reg {
            0b00 => self.es,
            0b01 => self.cs,
            0b10 => self.ss,
            0b11 => self.ds,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn set_segment(self: &mut Self, reg: u8, val: u16) {
        match reg {
            0b00 => self.es = val,
            0b01 => self.cs = val,
            0b10 => self.ss = val,
            0b11 => self.ds = val,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    // Se obtiene el offset y los ciclos que tarda en calcularse
    // fn get_offset(self: &mut Self, bus: &mut Bus) -> (u16, u8) {
    //     let (disp, cycles_extra, direct) = match self.op.addr_mode {
    //         AddrMode::Mode0 => (0, 0, true),
    //         AddrMode::Mode1 => (sign_extend(self.fetch(bus)), 4, false),
    //         AddrMode::Mode2 => (to_u16(self.fetch(bus), self.fetch(bus)), 4, false),
    //         _ => unreachable!("Aqui no deberia entrar")
    //     };

    //     match rm {
    //         0b000 => (self.bx.get_x().wrapping_add(self.si).wrapping_add(disp), 7 + cycles_extra),
    //         0b001 => (self.bx.get_x().wrapping_add(self.di).wrapping_add(disp), 8 + cycles_extra),
    //         0b010 => (self.bp.wrapping_add(self.si).wrapping_add(disp), 8 + cycles_extra),
    //         0b011 => (self.bp.wrapping_add(self.di).wrapping_add(disp), 7 + cycles_extra),
    //         0b100 => (self.si.wrapping_add(disp), 5 + cycles_extra),
    //         0b101 => (self.di.wrapping_add(disp), 5 + cycles_extra),
    //         0b110 => {
    //             if direct {
    //                 let low = self.fetch(bus);
    //                 let high = self.fetch(bus);
    //                 return (low as u16 + high as u16 * 0x100, 6);
    //             }
    //             (self.bp.wrapping_add(disp), 5 + cycles_extra)
    //         },
    //         0b111 => (self.bx.get_x().wrapping_add(disp), 5 + cycles_extra),
    //         _ => unreachable!("Aqui no deberia entrar nunca")
    //     }
    // }

    fn push_stack_8(self: &mut Self, bus: &mut Bus, val: u8) {
        bus.write_8(self.ss, self.sp, val);
        self.sp = self.sp.wrapping_add(1);
    }

    fn push_stack_16(self: &mut Self, bus: &mut Bus, val: u16) {
        let val = to_2u8(val);
        self.push_stack_8(bus, val.0);
        self.push_stack_8(bus, val.1);
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

