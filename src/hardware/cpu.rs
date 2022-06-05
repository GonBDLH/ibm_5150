use std::borrow::BorrowMut;
use std::collections::VecDeque;
 
use super::bus::Bus;
use super::regs::{GPReg, Flags};
use super::cpu_utils::*;

pub struct CPU {
    // Registros de proposito general
    ax: GPReg,
    bx: GPReg,
    cx: GPReg,
    dx: GPReg,

    // Registros indices
    si: u16,
    di: u16,
    bp: u16,
    sp: u16,

    // Flags
    pub flags: Flags,

    // Registros de segmentos
    cs: u16,
    ds: u16,
    es: u16,
    ss: u16,

    // Instruction pointer
    ip: u16,

    // Cola de instrucciones
    instr_queue: VecDeque<u8>,
    
    // Utilizado para guardar info de la operacion que se esta decodificando
    pop: bool,
    op: u8,
    instr_cycles: u8,
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

            instr_queue: VecDeque::with_capacity(4),
            pop: true,
            op: 0x00,
            instr_cycles: 0x00,
        }
    }
}

impl CPU {
    pub fn step(self: &mut Self, bus: &mut Bus) {
        // 14,31818 MHz * 1/50 Hz / 3 ~= 95454 => Nº ciclos que hace la CPU en un frame 
        for _i in 0..95454 {
            let read = self.fetch(bus);
            self.instr_queue.push_back(read);
    
            if self.pop {
                self.pop = false;
                self.op = self.instr_queue.pop_front().expect("Error al extraer instruccion de la cola.");
                self.decode(bus);
            }

            self.instr_cycles -= 1;
            if self.instr_cycles == 0 {
                self.pop = true;
            }

        }
    }

    fn fetch(self: &mut Self, bus: &mut Bus) -> u8 {
        let dir = ((self.cs as usize) << 4) + self.ip as usize;
        // self.ip += 1;
        self.ip = self.ip.wrapping_add(1);
        bus.read_dir(dir)
    }

    fn decode(self: &mut Self, bus: &mut Bus) {
        match self.op {
            0x00 => {
                self.add_rm8_r8(bus);
            },
            0x01 => {
                self.add_rm16_r16(bus);
            }
            _ => unreachable!("Instruccion no reconocida {:02X}.", self.op)
        }
    }
}

// Set de instrucciones. No quiero ponerlo en otro archivo para que no den probelmas las
// variables privadas.
impl CPU {
    fn set_reg8(self: &mut Self, reg: u8, val: u8) {
        match reg {
            0b000 => self.ax.low = val,
            0b001 => self.cx.low = val,
            0b010 => self.dx.low = val,
            0b011 => self.bx.low = val,
            0b100 => self.ax.high = val,
            0b101 => self.cx.high = val,
            0b110 => self.dx.high = val,
            0b111 => self.bx.high = val,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn set_reg16(self: &mut Self, reg: u8, val: u16) {
        match reg {
            0b000 => self.ax.setX(val),
            0b001 => self.cx.setX(val),
            0b010 => self.dx.setX(val),
            0b011 => self.bx.setX(val),
            0b100 => self.sp = val,
            0b101 => self.bp = val,
            0b110 => self.si = val,
            0b111 => self.di = val,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn get_reg8(self: &Self, reg: u8) -> u8 {
        match reg {
            0b000 => self.ax.low,
            0b001 => self.cx.low,
            0b010 => self.dx.low,
            0b011 => self.bx.low,
            0b100 => self.ax.high,
            0b101 => self.cx.high,
            0b110 => self.dx.high,
            0b111 => self.bx.high,
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn get_reg16(self: &Self, reg: u8) -> u16 {
        match reg {
            0b000 => self.ax.getX(),
            0b001 => self.cx.getX(),
            0b010 => self.dx.getX(),
            0b011 => self.bx.getX(),
            0b100 => self.sp,
            0b101 => self.bp,
            0b110 => self.si,
            0b111 => self.di,
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

    // Se obtiene el offset y los ciclos que tarda en calcularse
    fn get_offset(self: &mut Self, bus: &mut Bus, rm: u8, mode: u8) -> (u16, u8) {
        let (disp, cycles_extra, direct) = match mode {
            0 => (0, 0, true),
            1 => (sign_extend(self.fetch(bus)), 4, false),
            2 => (to_u16(self.fetch(bus), self.fetch(bus)), 4, false),
            _ => unreachable!("Aqui no deberia entrar")
        };

        match rm {
            0b000 => (self.bx.getX().wrapping_add(self.si).wrapping_add(disp), 7 + cycles_extra),
            0b001 => (self.bx.getX().wrapping_add(self.di).wrapping_add(disp), 8 + cycles_extra),
            0b010 => (self.bp.wrapping_add(self.si).wrapping_add(disp), 8 + cycles_extra),
            0b011 => (self.bp.wrapping_add(self.di).wrapping_add(disp), 7 + cycles_extra),
            0b100 => (self.si.wrapping_add(disp), 5 + cycles_extra),
            0b101 => (self.di.wrapping_add(disp), 5 + cycles_extra),
            0b110 => {
                if direct {
                    let low = self.fetch(bus);
                    let high = self.fetch(bus);
                    return (low as u16 + high as u16 * 0x100, 6);
                }
                (self.bp.wrapping_add(disp), 5 + cycles_extra)
            },
            0b111 => (self.bx.getX().wrapping_add(disp), 5 + cycles_extra),
            _ => unreachable!("Aqui no deberia entrar nunca")
        }
    }

    fn add_rm8_r8(self: &mut Self, bus: &mut Bus) {
        // ADD r/m8,r8
        // Bastante XD esto (son las 3 am, mañana tengo que estudiar un final entero, no deberia estar haciendo esto)
        let operand1 = self.fetch(bus);
        let mod_instr = operand1 >> 6;
        let reg = (operand1 & 0b00111000) >> 3;
        let rm = operand1 & 0b00000111;

        let (ea, src, dst, sum) = if mod_instr == 0b11 {
            let src = self.get_reg8(rm);
            let dst = self.get_reg8(reg);
            
            let sum = dst.overflowing_add(src);

            self.set_reg8(reg, sum.0);

            (0, src, dst, sum)
        } else {
            let segment = self.get_segment(reg);
            let (offset, cycles) = self.get_offset(bus, rm, mod_instr);
            
            let dst = bus.read_8(segment, offset);
            let src = self.get_reg8(reg);

            let sum = src.overflowing_add(dst);

            bus.write_8(segment, offset, sum.0);

            (cycles, src, dst, sum)
        };

        self.flags.set_o(sum.1);
        self.flags.set_s_8(sum.0);
        self.flags.set_z_8(sum.0);
        self.flags.set_a_8(src, dst);
        self.flags.set_p_8(sum.0);
        self.flags.set_c_8(dst, sum.0);

        self.instr_cycles = ea + 3;
        
    }

    fn add_rm16_r16(self: &mut Self, bus: &mut Bus) {
        // ADD r/m16,r16
        let operand1 = self.fetch(bus);
        let mod_instr = operand1 >> 6;
        let reg = (operand1 & 0b00111000) >> 3;
        let rm = operand1 & 0b00000111;

        let (ea, src, dst, sum) = if mod_instr == 0b11 {
            let src = self.get_reg16(rm);
            let dst = self.get_reg16(reg);

            let sum = dst.overflowing_add(src);

            self.set_reg16(reg, sum.0);

            (0, src, dst, sum)
        } else {
            let segment = self.get_segment(reg);
            let (offset, cycles) = self.get_offset(bus, rm, mod_instr);
            
            let dst = bus.read_16(segment, offset);
            let src = self.get_reg16(reg);

            let sum = src.overflowing_add(dst);

            bus.write_16(segment, offset, sum.0);

            (cycles, src, dst, sum)
        };

        // TODO Flags
    }
}