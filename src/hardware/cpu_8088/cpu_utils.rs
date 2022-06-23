use super::instr_utils::*;

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

// Dir -> true: L
//        false: R
pub fn rotate(val: u16, mut count: u32, length: Length, dir: bool) -> (u16, bool) {
    let mut last_bit = false;
    if dir {
        let mut res = val;
        while count > 0 {
            let msb = get_msb(val, length);
            (res, last_bit) = res.overflowing_shl(1);
            res |= msb as u16;
            count -= 1;
        }
        (res, last_bit)
    } else {
        match length {
            Length::Byte => {
                let mut res = val as u8;
                while count > 0 {
                    let lsb = (get_lsb(val, length) as u8) << 7;
                    (res, last_bit) = res.overflowing_shr(1);
                    res |= lsb;
                    count -= 1;
                }
                (res as u16, last_bit)
            },
            Length::Word => {
                let mut res = val;
                while count > 0 {
                    let lsb = (get_lsb(val, length) as u16) << 15;
                    (res, last_bit) = res.overflowing_shr(1);
                    res |= lsb;
                    count -= 1;
                }
                (res, last_bit)
            },
            _ => unreachable!(),
        }
    }
}

