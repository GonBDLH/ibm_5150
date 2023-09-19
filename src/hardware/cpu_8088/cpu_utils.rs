use super::{instr_utils::*, CPU};

pub fn get_address(cpu: &CPU) -> usize {
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
    let high = (val >> 8) as u8;

    (low, high)
}

pub fn to_u32(low: u16, high: u16) -> u32 {
    low as u32 + high as u32 * 0x10000
}

pub fn to_2u16(val: u32) -> (u16, u16) {
    let low = val as u16;
    let high = (val >> 16) as u16;

    (low, high)
}

pub fn get_msb(val: u16, len: Length) -> bool {
    match len {
        Length::Byte => val as u8 & 0x80 != 0,
        Length::Word => val & 0x8000 != 0,
        _ => unreachable!(),
    }
}

pub fn get_msb_1(val: u16, len: Length) -> bool {
    match len {
        Length::Byte => val as u8 & 0x40 != 0,
        Length::Word => val & 0x4000 != 0,
        _ => unreachable!(),
    }
}

pub fn get_lsb(val: u8) -> bool {
    val & 0x01 > 0
}

pub fn rotate_left(val: u16, count: u32, len: Length) -> u16 {
    match len {
        Length::Byte => (val as u8).rotate_left(count) as u16,
        Length::Word => val.rotate_left(count),
        _ => unreachable!(),
    }
}

pub fn rotate_right(val: u16, count: u32, len: Length) -> u16 {
    match len {
        Length::Byte => (val as u8).rotate_right(count) as u16,
        Length::Word => val.rotate_right(count),
        _ => unreachable!(),
    }
}

pub fn rotate_left_carry(cpu: &mut CPU, val: u16, mut count: u32, len: Length) -> u16 {
    match len {
        Length::Byte => {
            let mut res = val as u8;

            while count != 0 {
                let temp_cf = get_msb(res as u16, len);
                res = res.wrapping_shl(1) + cpu.flags.c as u8;
                cpu.flags.c = temp_cf;
                count -= 1;
            }

            res as u16
        }
        Length::Word => {
            let mut res = val;

            while count != 0 {
                let temp_cf = get_msb(res, len);
                res = res.wrapping_mul(2) + cpu.flags.c as u16;
                cpu.flags.c = temp_cf;
                count -= 1;
            }

            res
        }
        _ => unreachable!(),
    }
}

pub fn rotate_right_carry(cpu: &mut CPU, val: u16, count: u32, len: Length) -> u16 {
    if count == 0 {
        return val;
    }

    let mut count = if count > 0x1F { 0x1F } else { count };

    match len {
        Length::Byte => {
            count %= 9;
            let mut res = val as u8;

            while count > 0 {
                let to_carry = get_lsb(res);
                let from_carry = cpu.flags.c;

                cpu.flags.c = to_carry;
                res >>= 1;
                res |= (from_carry as u8) << 7;

                count -= 1;
            }

            res as u16
        }
        Length::Word => {
            count %= 17;
            let mut res = val;

            while count > 0 {
                let to_carry = get_lsb(res as u8);
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

pub fn add(val1: u16, val2: u16, length: Length) -> (u16, bool) {
    match length {
        Length::Byte => {
            let val1 = val1 as u8;
            let val2 = val2 as u8;
            let res = val1.overflowing_add(val2);
            (res.0 as u16, res.1)
        }
        Length::Word => val1.overflowing_add(val2),
        _ => unreachable!(),
    }
}

pub fn adc(val1: u16, val2: u16, cflag: u16, length: Length) -> (u16, bool) {
    match length {
        Length::Byte => {
            let val1 = val1 as u8;
            let val2 = val2 as u8;
            let cflag = cflag as u8;
            let res_temp = val1.overflowing_add(val2);
            let res = res_temp.0.overflowing_add(cflag);
            (res.0 as u16, res.1 | res_temp.1)
        }
        Length::Word => {
            let res_temp = val1.overflowing_add(val2);
            let res = res_temp.0.overflowing_add(cflag);
            (res.0, res.1 | res_temp.1)
        }
        _ => unreachable!(),
    }
}

pub fn sub(val1: u16, val2: u16, length: Length) -> (u16, bool) {
    match length {
        Length::Byte => {
            let val1 = val1 as u8;
            let val2 = val2 as u8;
            let res = val1.overflowing_sub(val2);
            (res.0 as u16, res.1)
        }
        Length::Word => val1.overflowing_sub(val2),
        _ => unreachable!(),
    }
}

pub fn sbb(val1: u16, val2: u16, cflag: u16, length: Length) -> (u16, bool) {
    match length {
        Length::Byte => {
            let val1 = val1 as u8;
            let val2 = val2 as u8;
            let cflag = cflag as u8;
            let res_temp = val1.overflowing_sub(val2);
            let res = res_temp.0.overflowing_sub(cflag);
            (res.0 as u16, res.1 | res_temp.1)
        }
        Length::Word => {
            let res_temp = val1.overflowing_sub(val2);
            let res = res_temp.0.overflowing_sub(cflag);
            (res.0, res.1 | res_temp.1)
        }
        _ => unreachable!(),
    }
}

pub fn sar(cpu: &mut CPU, val1: u16, mut count: u32, len: Length) -> u16 {
    match len {
        Length::Byte => {
            let mut res = val1 as u8 as i8;
            while count != 0 {
                cpu.flags.c = get_lsb(res as u8);
                res = res.wrapping_shr(1);
                count -= 1;
            }

            res as u16
        }
        Length::Word => {
            let mut res = val1 as i16;
            while count != 0 {
                cpu.flags.c = get_lsb(res as u8);
                res = res.wrapping_shr(1);
                count -= 1;
            }

            res as u16
        }
        _ => unreachable!(),
    }
}

pub fn shr(cpu: &mut CPU, val1: u16, mut count: u32, len: Length) -> u16 {
    match len {
        Length::Byte => {
            let mut res = val1 as u8;
            while count != 0 {
                cpu.flags.c = get_lsb(res);
                res = res.wrapping_div(2);
                count -= 1;
            }

            res as u16
        }
        Length::Word => {
            let mut res = val1;
            while count != 0 {
                cpu.flags.c = get_lsb(res as u8);
                res = res.wrapping_div(2);
                count -= 1;
            }

            res
        }
        _ => unreachable!(),
    }
}

pub fn salshl(cpu: &mut CPU, val1: u16, mut count: u32, len: Length) -> u16 {
    match len {
        Length::Byte => {
            let mut res = val1 as u8;
            while count != 0 {
                cpu.flags.c = get_msb(res as u16, len);
                res = res.wrapping_mul(2);
                count -= 1;
            }

            res as u16
        }
        Length::Word => {
            let mut res = val1;
            while count != 0 {
                cpu.flags.c = get_msb(res, len);
                res = res.wrapping_mul(2);
                count -= 1;
            }

            res
        }
        _ => unreachable!(),
    }
}
