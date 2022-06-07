use super::cpu_utils::*;

// Registro de proposito general (AX, BX, CX, DX)
pub struct GPReg {
    pub high: u8,
    pub low: u8,
}

impl GPReg {
    pub fn new() -> Self {
        GPReg { high: 0x00, low: 0x00 }
    }

    pub fn get_x(self: &Self) -> u16 {
        to_u16(self.low, self.high)
    }

    pub fn set_x(self: &mut Self, val: u16) {
        self.high = (val >> 8) as u8;
        self.low = val as u8;
    }
}

pub struct Flags {
    pub o: bool,
    pub d: bool,
    pub i: bool,
    pub t: bool,
    pub s: bool,
    pub z: bool,
    pub a: bool,
    pub p: bool,
    pub c: bool
}

impl Flags {
    pub fn new() -> Self {
        Flags { o: false, d: false, i: false, t: false, s: false, z: false, a: false, p: false, c: false }
    }

    pub fn set_flags(self: &mut Self, val: u16) {
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

    pub fn get_flags(self: &Self) -> u16 {
        self.o as u16 & 0b0000100000000000 +
        self.d as u16 & 0b0000010000000000 +
        self.i as u16 & 0b0000001000000000 +
        self.t as u16 & 0b0000000100000000 +
        self.s as u16 & 0b0000000010000000 +
        self.z as u16 & 0b0000000001000000 +
        self.a as u16 & 0b0000000000010000 +
        self.p as u16 & 0b0000000000000100 +
        self.c as u16 & 0b0000000000000001
    }

    pub fn set_o_8(self: &mut Self, src: u8, dst: u8) {
        /*
        Si b6 y b7 generan carry, o b6 no generan carry -> false
        Si solo 1 de los 2 genera carry -> true
        */
        self.o = (src & 0x7F + dst & 0x7F) == 0x80 && dst.overflowing_add(src).1 || 
                 !((src & 0x7F + dst & 0x7F) == 0x80 && dst.overflowing_add(src).1);

    }

    pub fn set_o_16(self: &mut Self, src: u16, dst: u16) {
        /*
        Si b6 y b7 generan carry, o b6 no generan carry -> false
        Si solo 1 de los 2 genera carry -> true
        */
        self.o = (src & 0x7FFF + dst & 0x7FFF) == 0x8000 && dst.overflowing_add(src).1 || 
                 !((src & 0x7FFF + dst & 0x7FFF) == 0x8000 && dst.overflowing_add(src).1);

    }

    pub fn set_s_8(self: &mut Self, val: u8) {
        self.z = (val & 0x80) != 0;
    }

    pub fn set_s_16(self: &mut Self, val: u16) {
        self.z = (val & 0x8000) != 0;
    }

    pub fn set_z_8(self: &mut Self, val: u8) {
        self.z = val == 0;
    }

    pub fn set_z_16(self: &mut Self, val: u16) {
        self.z = val == 0;
    }

    pub fn set_a_8(self: &mut Self, src: u8, dst: u8) {
        self.a = ((src & 0x0F) + (dst & 0x0F)) & 0xF0 != 0;
    }

    pub fn set_a_16(self: &mut Self, src: u16, dst: u16) {
        // No se si esta bien
        self.a = ((src & 0x000F) + (dst & 0x000F)) & 0x00F0 != 0;
    }

    pub fn set_p_8(self: &mut Self, val: u8) {
        let mut a = 0;

        for x in 0..8 {
            if (1 << x) & val != 0 {
                a += 1;
            }
        }
        self.p = a % 2 == 0;
    }

    pub fn set_p_16(self: &mut Self, val: u16) {
        let mut a = 0;

        for x in 0..16 {
            if (1 << x) & val != 0 {
                a += 1;
            }
        }
        self.p = a % 2 == 0;
    }

    pub fn set_c_8(self: &mut Self, dst: u8, val: u8) {
        self.c = (dst & 0x80) != (val & 0x80);
    }

    pub fn set_c_16(self: &mut Self, dst: u16, val: u16) {
        self.c = (dst & 0x8000) != (val & 0x8000);
    }

}

