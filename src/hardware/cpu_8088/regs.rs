use super::{cpu_utils::*, instr_utils::{Length, OperandType}};

// Registro de proposito general (AX, BX, CX, DX)
#[derive(Copy, Clone)]
pub struct GPReg {
    pub high: u8,
    pub low: u8,
}

impl GPReg {
    pub fn new() -> Self {
        GPReg { high: 0x00, low: 0x00 }
    }

    pub fn get_x(&self) -> u16 {
        to_u16(self.low, self.high)
    }

    pub fn set_x(&mut self, val: u16) {
        self.high = (val >> 8) as u8;
        self.low = val as u8;
    }
}

#[derive(Copy, Clone)]
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
        Flags { o: false, d: false, i: true, t: false, s: false, z: false, a: false, p: false, c: false }
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
        let i = ((self.i as u16) <<  9) & 0b0000001000000000;
        let t = ((self.t as u16) <<  8) & 0b0000000100000000;
        let s = ((self.s as u16) <<  7) & 0b0000000010000000;
        let z = ((self.z as u16) <<  6) & 0b0000000001000000;
        let a = ((self.a as u16) <<  4) & 0b0000000000010000;
        let p = ((self.p as u16) <<  2) & 0b0000000000000100;
        let c = ((self.c as u16) <<  0) & 0b0000000000000001;
        o + d + i + t + s + z + a + p + c
    }
}

fn check_o_add_8(val1: u8, val2: u8, res: u8) -> bool {
    let sign_1 = val1 >> 7;
    let sign_2 = val2 >> 7;
    let sign_res = res >> 7;

    // match (sign_1, sign_2, sign_res) {
    //     (0, 0, 1) | (1, 1, 0) => true,
    //     _ => false,
    // }
    matches!((sign_1, sign_2, sign_res), (0, 0, 1) | (1, 1, 0))
}

fn check_o_add_16(val1: u16, val2: u16, res: u16) -> bool {
    let sign_1 = val1 >> 15;
    let sign_2 = val2 >> 15;
    let sign_res = res >> 15;

    // match (sign_1, sign_2, sign_res) {
    //     (0, 0, 1) | (1, 1, 0) => true,
    //     _ => false,
    // }
    matches!((sign_1, sign_2, sign_res), (0, 0, 1) | (1, 1, 0))
}

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

fn check_s_16(res: u16) -> bool {
    res >> 15 == 1
}

fn check_s_8(res: u8) -> bool {
    res >> 7 == 1
}

fn check_z(res: u16) -> bool {
    res == 0
}

fn check_a(val1: u16, val2: u16) -> bool {
    ((val1 as u8 & 0x0F) + (val2 as u8 & 0x0F)) & 0xF0 != 0
}

fn check_c_add_16(val1: u16, val2: u16) -> bool {
    val1.overflowing_add(val2).1
}

fn check_c_add_8(val1: u8, val2: u8) -> bool {
    val1.overflowing_add(val2).1
}

fn check_c_sub_16(val1: u16, val2: u16) -> bool {
    val1.overflowing_sub(val2).1
}

fn check_c_sub_8(val1: u8, val2: u8) -> bool {
    val1.overflowing_sub(val2).1
}

fn check_p(val: u16) -> bool {
    val.count_ones() % 2 == 0
}

impl Flags {
    pub fn set_add_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16) {
        match length {
            Length::Word => {
                self.o = check_o_add_16(val1, val2, res);
                self.s = check_s_16(res);
                self.z = check_z(res);
                self.a = check_a(val1, val2);
                self.p = check_p(res);
                self.c = check_c_add_16(val1, val2);
            },
            Length::Byte => {
                self.o = check_o_add_8(val1 as u8, val2 as u8, res as u8);
                self.s = check_s_8(res as u8);
                self.z = (res as u8) == 0;
                self.a = check_a(val1, val2);
                self.p = check_p(res);
                self.c = check_c_add_8(val1 as u8, val2 as u8);
            },
            _ => unreachable!(),
        }
    }

    pub fn set_sub_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16) {
        match length {
            Length::Word => {
                self.o = check_o_sub_16(val1, val2, res);
                self.s = check_s_16(res);
                self.z = check_z(res);
                self.a = check_a(val1, val2);
                self.p = check_p(res);
                self.c = check_c_sub_16(val1, val2);
            },
            Length::Byte => {
                self.o = check_o_sub_8(val1 as u8, val2 as u8, res as u8);
                self.s = check_s_8(res as u8);
                self.z = (res as u8) == 0;
                self.a = check_a(val1, val2);
                self.p = check_p(res);
                self.c = check_c_sub_8(val1 as u8, val2 as u8);
            },
            _ => unreachable!(),
        }
    }

    pub fn set_neg_flags(&mut self, length: Length, val1: u16, val2: u16, res: u16) {
        match length {
            Length::Word => {
                self.o = check_o_sub_16(val1, val2, res);
                self.s = check_s_16(res);
                self.z = check_z(res);
                self.a = check_a(val1, val2);
                self.p = check_p(res);
                self.c = val2 != 0;
            },
            Length::Byte => {
                self.o = check_o_sub_8(val1 as u8, val2 as u8, res as u8);
                self.s = check_s_8(res as u8);
                self.z = (res as u8) == 0;
                self.a = check_a(val1, val2);
                self.p = check_p(res);
                self.c = val2 as u8 != 0;
            },
            _ => unreachable!(),
        }
    }

    pub fn set_mul_flags(&mut self, length: Length, res: u32) {
        match length {
            Length::Word => {
                let res_high = res & 0xFFFF0000;
                self.o = res_high != 0;
                self.c = res_high != 0;
            },
            Length::Byte => {
                let res_high = res as u16 & 0xFF00;
                self.o = res_high != 0;
                self.c = res_high != 0;
            },
            _ => unreachable!(),
        }
    }

    pub fn set_aam_flags(&mut self, val1: u8) {
        self.s = check_s_8(val1);
        self.z = val1 == 0;
        self.p = check_p(val1 as u16)
    }

    pub fn set_salshl_flags(&mut self, length: Length, operand: OperandType, count: u32, val: u16, res: u16) {
        if count == 0 {
            return;
        }
        match length {
            Length::Byte => {
                if count < 8 {
                    let mask = 1u8 << count;
                    self.c = val as u8 & mask == mask;
                } else {
                    self.c = false;
                }
                self.s = check_s_8(res as u8);
            },
            Length::Word => {
                if count < 16 {
                    let mask = 1u16 << count;
                    self.c = val & mask == mask;
                } else {
                    self.c = false;
                };
                self.s = check_s_16(res);
            },
            _ => unreachable!(),
        };

        if let OperandType::Immediate(_) = operand {
            self.o = get_msb(res, length) ^ self.c;
        }
        self.z = check_z(res);
        self.p = check_p(res);
    }

    pub fn set_shr_flags(&mut self, length: Length, operand: OperandType, count: u32, val: u16, res: u16) {
        if count == 0 {
            return;
        }

        match length {
            Length::Byte => {
                if count < 8 {
                    let mask = 1u8 << (count - 1);
                    self.c = val as u8 & mask == mask;
                } else {
                    self.c = false;
                }
                self.s = check_s_8(res as u8);
            },
            Length::Word => {
                if count < 16 {
                    let mask = 1u16 << (count - 1);
                    self.c = val & mask == mask;
                } else {
                    self.c = false;
                }
                self.s = check_s_16(res);
            },
            _ => unreachable!(),
        };
        self.z = check_z(res);
        self.p = check_p(res);

        if let OperandType::Immediate(_) = operand {
            self.o = get_msb(val, length) != get_msb(res, length);
        }
    }

    pub fn set_sar_flags(&mut self, length: Length, operand: OperandType, count: u32, val: u16, res: u16) {
        if count == 0 {
            return;
        }
        match length {
            Length::Byte => {
                if count < 8 {
                    let mask = 1u8 >> count;
                    self.c = val as u8 & mask == mask;
                } else {
                    self.c = false;
                }
                check_s_8(res as u8);
            },
            Length::Word => {
                if count < 16 {
                    let mask = 1u16 >> count;
                    self.c = val & mask == mask;
                } else {
                    self.c = false;
                }
                check_s_16(res);
            },
            _ => unreachable!(),
        };
        
        if let OperandType::Immediate(_) = operand {
            self.o = get_msb(val, length) != get_msb(res, length)
        }
        
        self.z = check_z(res);
        self.p = check_p(res);
    }

    pub fn set_rot_flags(&mut self, last_bit: bool, count: u32, res: u16, val: u16, length: Length) {
        self.c = last_bit;
        if count == 1 {
            self.o = get_msb(val, length) != get_msb(res, length);
        }
    }

    pub fn set_rc_flags(&mut self, count: u32, res: u16, val: u16, length: Length) {
        if count == 1 {
            self.o = get_msb(val, length) != get_msb(res, length);
        }
    }

    pub fn set_logic_flags(&mut self, length: Length, res: u16) {
        self.o = false;
        self.c = false;
        match length {
            Length::Byte => {
                self.c = check_s_8(res as u8);
            },
            Length::Word => {
                self.c = check_s_16(res);
            },
            _ => unreachable!(),
        }
        self.z = check_z(res);
        self.p = check_p(res);
    }
}

