use std::fmt::Debug;

use super::{
    cpu_utils::*,
    instr_utils::{Length, Opcode},
};

// Registro de proposito general (AX, BX, CX, DX)
#[derive(Copy, Clone)]
pub struct GPReg {
    pub high: u8,
    pub low: u8,
}

impl GPReg {
    pub fn new() -> Self {
        GPReg {
            high: 0x00,
            low: 0x00,
        }
    }

    pub fn get_x(&self) -> u16 {
        to_u16(self.low, self.high)
    }

    pub fn set_x(&mut self, val: u16) {
        self.high = (val >> 8) as u8;
        self.low = val as u8;
    }
}

impl Default for GPReg {
    fn default() -> Self {
        GPReg::new()
    }
}

impl Debug for GPReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X}{:02X}", self.high, self.low)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Flags {
    pub o: bool,
    pub d: bool,
    pub i: bool,
    pub t: bool,
    pub s: bool,
    pub z: bool,
    pub a: bool,
    pub p: bool,
    pub c: bool,
}

impl Flags {
    pub fn new() -> Self {
        Flags {
            o: false,
            d: false,
            i: true,
            t: false,
            s: false,
            z: false,
            a: false,
            p: false,
            c: false,
        }
    }

    pub fn set_flags(&mut self, val: u16) {
        self.o = val & 0b0000100000000000 == 0b0000100000000000;
        self.d = val & 0b0000010000000000 == 0b0000010000000000;
        self.i = val & 0b0000001000000000 == 0b0000001000000000;
        self.t = val & 0b0000000100000000 == 0b0000000100000000;
        self.s = val & 0b0000000010000000 == 0b0000000010000000;
        self.z = val & 0b0000000001000000 == 0b0000000001000000;
        self.a = val & 0b0000000000010000 == 0b0000000000010000;
        self.p = val & 0b0000000000000100 == 0b0000000000000100;
        self.c = val & 0b0000000000000001 == 0b0000000000000001;
    }

    pub fn get_flags(&self) -> u16 {
        let o = ((self.o as u16) << 11) & 0b0000100000000000;
        let d = ((self.d as u16) << 10) & 0b0000010000000000;
        let i = ((self.i as u16) << 9) & 0b0000001000000000;
        let t = ((self.t as u16) << 8) & 0b0000000100000000;
        let s = ((self.s as u16) << 7) & 0b0000000010000000;
        let z = ((self.z as u16) << 6) & 0b0000000001000000;
        let a = ((self.a as u16) << 4) & 0b0000000000010000;
        let p = ((self.p as u16) << 2) & 0b0000000000000100;
        let c = (self.c as u16) & 0b0000000000000001;
        o + d + i + t + s + z + a + p + c
    }
}

impl Default for Flags {
    fn default() -> Self {
        Flags::new()
    }
}

#[inline]
fn check_o_add(val1: u16, val2: u16, res: u16, len: Length) -> bool {
    let sign1 = get_msb(val1, len);
    let sign2 = get_msb(val2, len);
    let sign_res = get_msb(res, len);

    matches!((sign1, sign2, sign_res), (false, false, true) | (true, true, false))
}

#[inline]
fn check_o_sub_8(val1: u8, val2: u8, res: u8) -> bool {
    let sign_1 = val1 >> 7;
    let sign_2 = val2 >> 7;
    let sign_res = res >> 7;

    // match (sign_1, sign_2, sign_res) {
    //     (0, 1, 1) | (1, 0, 0) => true,
    //     _ => false,
    // }
    matches!((sign_1, sign_2, sign_res), (0, 1, 1) | (1, 0, 0))
}

#[inline]
fn check_o_sub_16(val1: u16, val2: u16, res: u16) -> bool {
    let sign_1 = val1 >> 15;
    let sign_2 = val2 >> 15;
    let sign_res = res >> 15;

    // match (sign_1, sign_2, sign_res) {
    //     (0, 1, 1) | (1, 0, 0) => true,
    //     _ => false,
    // }
    matches!((sign_1, sign_2, sign_res), (0, 1, 1) | (1, 0, 0))
}

#[inline]
fn check_s_16(res: u16) -> bool {
    res & 0x8000 > 0
}

#[inline]
fn check_s_8(res: u8) -> bool {
    res & 0x80 > 0
}

#[inline]
fn check_s(val: u16, len: Length) -> bool {
    match len {
        Length::Byte => check_s_8(val as u8),
        Length::Word => check_s_16(val),
        _ => unreachable!()
    }
}

#[inline]
fn check_z(res: u16, len: Length) -> bool {
    match len {
        Length::Byte => res as u8 == 0,
        Length::Word => res == 0,
        _ => unreachable!(),
    }
}

#[inline]
fn check_a(val1: u16, val2: u16) -> bool {
    ((val1 as u8 & 0x0F) + (val2 as u8 & 0x0F)) & 0xF0 != 0
}

#[inline]
fn check_c_sub_16(val1: u16, val2: u16) -> bool {
    val1.overflowing_sub(val2).1
}

#[inline]
fn check_c_sub_8(val1: u8, val2: u8) -> bool {
    val1.overflowing_sub(val2).1
}

#[inline]
fn check_p(val: u16, len: Length) -> bool {
    match len {
        Length::Byte => (val as u8).count_ones() % 2 == 0,
        Length::Word => val.count_ones() % 2 == 0,
        _ => unreachable!()
    }
}
    
fn check_c_salshl(val: u16, count: u32, len: Length) -> bool {
    match len {
        Length::Byte => {
            let mask = (0x0100 >> count) as u8;
            mask & val as u8 > 0
        },
        Length::Word => {
            let mask = (0x010000 >> count) as u16;
            mask & val > 0
        },
        _ => unreachable!()
    }
}

fn check_c_shr(val: u16, count: u32, len: Length) -> bool {
    match len {
        Length::Byte => {
            let mask = ((1 << count) >> 1) as u8;
            mask & val as u8 > 0
        },
        Length::Word => {
            let mask = ((1 << count) >> 1) as u16;
            mask & val > 0
        },
        _ => unreachable!()
    }
}


fn check_c_sar(val: u16, count: u32, len: Length) -> bool {
    match len {
        Length::Byte => {
            if count > 7 {
                return get_msb(val, len);
            }

            let mask = ((1 << count) >> 1) as u8;
            mask & val as u8 > 0
        },
        Length::Word => {
            if count > 15 {
                return get_msb(val, len);
            }

            let mask = ((1 << count) >> 1) as u16;
            mask & val > 0
        },
        _ => unreachable!()
    }
}

impl Flags {
    pub fn set_add_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16, of: bool) {
        self.o = check_o_add(val1, val2, res, length) | of;
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val1, val2);
        self.p = check_p(res, length);
        self.c = of;
    }

    pub fn set_inc_flags(&mut self, length: Length, val: u16, res: u16) {
        self.o = check_o_add(val, 1, res, length);
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val, 1);
        self.p = check_p(res, length);
    }

    pub fn set_sub_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16) {
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val1, val2);
        self.p = check_p(res, length);

        match length {
            Length::Word => {
                self.o = check_o_sub_16(val1, val2, res);
                self.c = check_c_sub_16(val1, val2);
            }
            Length::Byte => {
                self.o = check_o_sub_8(val1 as u8, val2 as u8, res as u8);
                self.c = check_c_sub_8(val1 as u8, val2 as u8);
            }
            _ => unreachable!(),
        }
    }

    pub fn set_dec_flags(&mut self, length: Length, val: u16, res: u16) {
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val, 1);
        self.p = check_p(res, length);
        
        match length {
            Length::Word => {
                self.o = check_o_sub_16(val, 1, res);
            }
            Length::Byte => {
                self.o = check_o_sub_8(val as u8, 1, res as u8);
            }
            _ => unreachable!(),
        }
    }
    pub fn set_neg_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16) {
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val1, val2);
        self.p = check_p(res, length);

        match length {
            Length::Word => {
                self.o = check_o_sub_16(val1, val2, res);
                self.c = val2 != 0;
            }
            Length::Byte => {
                self.o = check_o_sub_8(val1 as u8, val2 as u8, res as u8);
                self.c = val2 as u8 != 0;
            }
            _ => unreachable!(),
        }
    }

    pub fn set_imul_flags(&mut self, length: Length, res: u32) {
        match length {
            Length::Word => {
                let val = !matches!(res & 0x80008000, 0x80008000 | 0x0);
                self.o = val;
                self.c = val;
            }
            Length::Byte => {
                let val = !matches!(res & 0x8080, 0x8080 | 0x0000);
                self.o = val;
                self.c = val;
            }
            _ => unreachable!(),
        }
    }

    pub fn set_mul_flags(&mut self, length: Length, res: u64) {
        match length {
            Length::Word => {
                let res_high = res & 0xFFFF0000;
                self.o = res_high != 0;
                self.c = res_high != 0;
            }
            Length::Byte => {
                let res_high = res as u16 & 0xFF00;
                self.o = res_high != 0;
                self.c = res_high != 0;
            }
            _ => unreachable!(),
        }
    }

    pub fn set_aam_flags(&mut self, val1: u8) {
        self.s = check_s_8(val1);
        self.z = val1 == 0;
        self.p = val1.count_ones() % 2 == 0;
    }

    pub fn set_shift_flags(&mut self, val: u16, count: u32, res: u16, len: Length, opcode: Opcode) {
        self.z = check_z(res, len);
        self.p = check_p(res, len);
        self.s = check_s(res, len);

        self.c = match opcode {
            Opcode::SALSHL => check_c_salshl(val, count, len),
            Opcode::SHR => check_c_shr(val, count, len),
            Opcode::SAR => check_c_sar(val, count, len),
            _ => unreachable!()
        };

        if count == 1 {
            self.o = get_msb(val, len) != get_msb(res, len);
        }
    }

    pub fn set_rotate_flags(&mut self, count: u32, len: Length, val: u16, res: u16, last_bit: bool) {
        self.c = last_bit;

        if count == 1 {
            self.o = get_msb(val, len) != get_msb(res, len);
        }
    }

    pub fn set_logic_flags(&mut self, length: Length, res: u16) {
        self.o = false;
        self.c = false;
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.p = check_p(res, length);
    }
}
