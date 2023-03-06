use super::{instr_utils::*, CPU};

pub fn get_address(cpu: &mut CPU) -> usize {
    ((cpu.cs as usize) << 4) + cpu.ip as usize
}

pub fn sign_extend(value: u8) -> u16 {
    value as i8 as i16 as u16
}

pub fn sign_extend_32(value: u16) -> u32 {
    value as i16 as i32 as u32
}

pub fn to_u16(low: u8, high: u8) -> u16 {
    low as u16 + high as u16 * 0x100
}

pub fn to_2u8(val: u16) -> (u8, u8) {
    let low = val as u8;
    let high = ((val & 0xFF00) >> 8) as u8;

    (low, high)
}

pub fn to_u32(low: u16, high: u16) -> u32 {
    low as u32 + high as u32 * 0x10000
}

pub fn to_2u16(val: u32) -> (u16, u16) {
    let low = val as u16;
    let high = ((val & 0xFFFF0000) >> 16) as u16;

    (low, high)
}

pub fn get_msb(val: u16, len: Length) -> bool {
    match len {
        Length::Byte => val as u8 & 0x80 != 0,
        Length::Word => val & 0x8000 != 0,
        _ => unreachable!(),
    }
}

pub fn get_lsb(val: u16, length: Length) -> bool {
    match length {
        Length::Byte => val as u8 & 0x01 != 0,
        Length::Word => val & 0x0001 != 0,
        _ => unreachable!(),
    }
}

pub fn rotate_left(val: u16, count: u32, len: Length) -> (u16, bool) {
    match len {
        Length::Byte => {
            let res = (val as u8).rotate_left(count);
            let last = (0x01 & res) != 0;
            (res as u16, last)
        }
        Length::Word => {
            let res = val.rotate_left(count);
            let last = (0x0001 & res) != 0;
            (res, last)
        }
        _ => unreachable!(),
    }
}

pub fn rotate_rigth(val: u16, count: u32, len: Length) -> (u16, bool) {
    match len {
        Length::Byte => {
            let res = (val as u8).rotate_right(count);
            let last = (0x80 & res) != 0;
            (res as u16, last)
        }
        Length::Word => {
            let res = val.rotate_right(count);
            let last = (0x8000 & res) != 0;
            (res, last)
        }
        _ => unreachable!(),
    }
}

pub fn rotate_left_carry(cpu: &mut CPU, val: u16, mut count: u32, len: Length) -> u16 {
    match len {
        Length::Byte => {
            let mut res = val as u8;

            while count > 0 {
                let to_carry = (0x80 & res) != 0;
                let from_carry = cpu.flags.c;

                cpu.flags.c = to_carry;
                res <<= 1;
                res |= from_carry as u8;

                count -= 1;
            }

            res as u16
        }
        Length::Word => {
            let mut res = val;

            while count > 0 {
                let to_carry = (0x8000 & res) != 0;
                let from_carry = cpu.flags.c;

                cpu.flags.c = to_carry;
                res <<= 1;
                res |= from_carry as u16;

                count -= 1;
            }

            res
        }
        _ => unreachable!(),
    }
}

pub fn rotate_right_carry(cpu: &mut CPU, val: u16, mut count: u32, len: Length) -> u16 {
    match len {
        Length::Byte => {
            let mut res = val as u8;

            while count > 0 {
                let to_carry = (0x01 & res) != 0;
                let from_carry = cpu.flags.c;

                cpu.flags.c = to_carry;
                res >>= 1;
                res |= (from_carry as u8) << 7;

                count -= 1;
            }

            res as u16
        }
        Length::Word => {
            let mut res = val;

            while count > 0 {
                let to_carry = (0x0001 & res) != 0;
                let from_carry = cpu.flags.c;

                cpu.flags.c = to_carry;
                res >>= 1;
                res |= (from_carry as u16) << 15;

                count -= 1;
            }

            res
        }
        _ => unreachable!(),
    }
}
