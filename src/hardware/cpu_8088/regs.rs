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
            #[cfg(not(test))]
            //#[cfg(not(feature = "tests"))]
            i: true,
            //#[cfg(feature = "tests")]
            #[cfg(test)]
            i: false,
            t: false,
            s: false,
            z: false,
            a: false,
            p: false,
            c: false,
        }
    }

    pub fn set_flags(&mut self, val: u16) {
        self.o = val & 0b0000100000000000 > 0;
        self.d = val & 0b0000010000000000 > 0;
        self.i = val & 0b0000001000000000 > 0;
        self.t = val & 0b0000000100000000 > 0;
        self.s = val & 0b0000000010000000 > 0;
        self.z = val & 0b0000000001000000 > 0;
        self.a = val & 0b0000000000010000 > 0;
        self.p = val & 0b0000000000000100 > 0;
        self.c = val & 0b0000000000000001 > 0;
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

        0xF000 + o + d + i + t + s + z + a + p + c + 2
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

    matches!(
        (sign1, sign2, sign_res),
        (false, false, true) | (true, true, false)
    )
}

#[inline]
fn check_o_sub(val1: u16, val2: u16, res: u16, len: Length) -> bool {
    let sign1 = get_msb(val1, len);
    let sign2 = get_msb(val2, len);
    let sign_res = get_msb(res, len);

    matches!(
        (sign1, sign2, sign_res),
        (false, true, true) | (true, false, false)
    )
}

#[inline]
pub fn check_s(val: u16, len: Length) -> bool {
    get_msb(val, len)
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
fn check_a(val1: u16, val2: u16, res: u16) -> bool {
    ((res ^ val1 ^ val2) & (1 << 4)) > 0
}

#[inline]
pub fn check_p(val: u16) -> bool {
    (val as u8).count_ones() % 2 == 0
}

fn check_c_salshl(val: u16, count: u32, len: Length) -> bool {
    match len {
        Length::Byte => {
            let mask = (0x0100u16.wrapping_shr(count)) as u8;
            mask & val as u8 > 0
        }
        Length::Word => {
            let mask = (0x010000u32.wrapping_shr(count)) as u16;
            mask & val > 0
        }
        _ => unreachable!(),
    }
}

fn check_c_shr(val: u16, count: u32, len: Length) -> bool {
    match len {
        Length::Byte => {
            let mask = (1u8.wrapping_shl(count)) >> 1;
            mask & val as u8 > 0
        }
        Length::Word => {
            let mask = (1u16.wrapping_shl(count)) >> 1;
            mask & val > 0
        }
        _ => unreachable!(),
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
        }
        Length::Word => {
            if count > 15 {
                return get_msb(val, len);
            }

            let mask = ((1 << count) >> 1) as u16;
            mask & val > 0
        }
        _ => unreachable!(),
    }
}

impl Flags {
    pub fn set_add_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16, cf: bool) {
        self.o = check_o_add(val1, val2, res, length);
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val1, val2, res);
        self.p = check_p(res);
        self.c = cf;
    }

    pub fn set_inc_flags(&mut self, length: Length, val: u16, res: u16) {
        self.o = check_o_add(val, 1, res, length);
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val, 1, res);
        self.p = check_p(res);
        self.p = check_p(res);
    }

    pub fn set_sub_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16, cf: bool) {
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val1, val2, res);
        self.p = check_p(res);
        self.o = check_o_sub(val1, val2, res, length);
        self.c = cf;
    }

    pub fn set_dec_flags(&mut self, length: Length, val: u16, res: u16) {
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val, 1, res);
        self.p = check_p(res);
        self.o = check_o_sub(val, 1, res, length)
    }

    pub fn set_neg_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16) {
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.a = check_a(val1, val2, res);
        self.p = check_p(res);
        self.o = check_o_sub(val1, val2, res, length);

        match length {
            Length::Word => {
                self.c = val2 != 0;
            }
            Length::Byte => {
                self.c = val2 as u8 != 0;
            }
            _ => unreachable!(),
        }
    }

    pub fn set_imul_flags(&mut self, length: Length, res: u32) {
        match length {
            Length::Word => {
                let res_temp = sign_extend_32(res as u16);
                self.o = res_temp != res;
                self.c = res_temp != res;
            }
            Length::Byte => {
                let res_temp = sign_extend(res as u8);
                self.o = res_temp != res as u16;
                self.c = res_temp != res as u16;
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
        self.s = check_s(val1 as u16, Length::Byte);
        self.z = val1 == 0;
        self.p = val1.count_ones() % 2 == 0;
    }

    pub fn set_shift_flags(&mut self, val: u16, count: u32, res: u16, len: Length, opcode: Opcode) {
        self.z = check_z(res, len);
        self.p = check_p(res);
        self.s = check_s(res, len);

        self.c = match opcode {
            Opcode::SALSHL => check_c_salshl(val, count, len),
            Opcode::SHR => check_c_shr(val, count, len),
            Opcode::SAR => check_c_sar(val, count, len),
            _ => unreachable!(),
        };

        self.o = get_msb(val, len) != get_msb(res, len);
    }

    pub fn set_rl_flags(&mut self, count: u32, len: Length, _val: u16, res: u16, last_bit: bool) {
        // println!("{:016b}", res);
        if count != 0 {
            self.c = last_bit;
        }

        self.o = get_msb(res, len) ^ self.c;
    }

    pub fn set_rr_flags(&mut self, count: u32, len: Length, _val: u16, res: u16, last_bit: bool) {
        // println!("{:016b}", res);
        if count != 0 {
            self.c = last_bit;
        }

        self.o = get_msb(res, len) ^ get_msb_1(res, len);
    }

    pub fn set_logic_flags(&mut self, length: Length, res: u16) {
        self.o = false;
        self.c = false;
        self.s = check_s(res, length);
        self.z = check_z(res, length);
        self.p = check_p(res);
    }

    pub fn set_das_flags(&mut self, len: Length, val: u16) {
        self.p = check_p(val);
        self.s = check_s(val, len);
        self.z = check_z(val, len);
    }
}
