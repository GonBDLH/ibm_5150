pub fn sign_extend(value: u8) -> u16 {
    value as i8 as i16 as u16
}

pub fn to_u16(low: u8, high: u8) -> u16 {
    low as u16 + high as u16 * 0x100
}

pub fn to_2u8(val: u16) -> (u8, u8) {
    let low = val as u8;
    let high = ((val & 0xFF00) >> 8) as u8;
    
    (low, high)
}