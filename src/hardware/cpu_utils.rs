use super::{instr_utils::{Length, Operand, RepetitionPrefix, JumpType}, cpu::CPU, bus::Bus};

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

pub fn movs(cpu: &mut CPU, bus: &mut Bus) {
    let offset_from = cpu.si;
    let offset_to = cpu.di;

    let segment_from = if cpu.instr.segment == Operand::None {
        Operand::DS
    } else {
        cpu.instr.segment
    };
    let segment_to = Operand::ES;

    let val = bus.read_length(cpu, segment_from, offset_from, cpu.instr.data_length);
    bus.write_length(cpu, cpu.instr.data_length, segment_to, offset_to, val);
}

pub fn cmps(cpu: &mut CPU, bus: &mut Bus) {
    let offset_from = cpu.si;
    let offset_to = cpu.di;

    let segment_from = if cpu.instr.segment == Operand::None {
        Operand::DS
    } else {
        cpu.instr.segment
    };
    let segment_to = Operand::ES;

    let val1 = bus.read_length(cpu, segment_from, offset_from, cpu.instr.data_length);
    let val2 = bus.read_length(cpu, segment_to, offset_to, cpu.instr.data_length);
    let res = val1.wrapping_sub(val2);
    cpu.flags.set_sub_flags(cpu.instr.data_length, val1, val2, res);
}

pub fn scas(cpu: &mut CPU, bus: &mut Bus) {
    let offset_to = cpu.di;
    let segment_to = Operand::ES;

    let val1 = match cpu.instr.data_length {
        Length::Byte => cpu.ax.low as u16,
        Length::Word => cpu.ax.get_x(),
        _ => unreachable!()
    };
    let val2 = bus.read_length(cpu, segment_to, offset_to, cpu.instr.data_length);
    let res = val1.wrapping_sub(val2);
    cpu.flags.set_sub_flags(cpu.instr.data_length, val1, val2, res);
}

pub fn lods(cpu: &mut CPU, bus: &mut Bus) {
    let offset_from = cpu.si;
    let segment_from = if cpu.instr.segment == Operand::None {
        Operand::DS
    } else {
        cpu.instr.segment
    };

    let val = bus.read_length(cpu, segment_from, offset_from, cpu.instr.data_length);

    match cpu.instr.data_length {
        Length::Byte => cpu.ax.low = val as u8,
        Length::Word => cpu.ax.set_x(val),
        _ => unreachable!()
    };
}

pub fn stos(cpu: &mut CPU, bus: &mut Bus) {
    let offset_to = cpu.di;
    let segment_to = Operand::ES;

    let val = match cpu.instr.data_length {
        Length::Byte => cpu.ax.low as u16,
        Length::Word => cpu.ax.get_x(),
        _ => unreachable!(),
    };

    bus.write_length(cpu, cpu.instr.data_length, segment_to, offset_to, val);
}

pub fn adjust_string(cpu: &mut CPU) {
    let to_change = match cpu.instr.data_length {
        Length::Byte => 1,
        Length::Word => 2,
        _ => unreachable!(),
    };

    if !cpu.flags.d {
        cpu.si = cpu.si.wrapping_add(to_change);
        cpu.di = cpu.di.wrapping_add(to_change);
    } else {
        cpu.si = cpu.si.wrapping_sub(to_change);
        cpu.di = cpu.di.wrapping_sub(to_change);
    }
}

pub fn adjust_string_di(cpu: &mut CPU) {
    let to_change = match cpu.instr.data_length {
        Length::Byte => 1,
        Length::Word => 2,
        _ => unreachable!(),
    };

    if !cpu.flags.d {
        cpu.di = cpu.di.wrapping_add(to_change);
    } else {
        cpu.di = cpu.di.wrapping_sub(to_change);
    }
}

pub fn adjust_string_si(cpu: &mut CPU) {
    let to_change = match cpu.instr.data_length {
        Length::Byte => 1,
        Length::Word => 2,
        _ => unreachable!(),
    };

    if !cpu.flags.d {
        cpu.di = cpu.si.wrapping_add(to_change);
    } else {
        cpu.di = cpu.si.wrapping_sub(to_change);
    }
}

pub fn check_z_str(cpu: &mut CPU) -> bool {
    match cpu.instr.repetition_prefix {
        RepetitionPrefix::REPEZ => {
            cpu.flags.z
        },
        RepetitionPrefix::REPNEZ => {
            !cpu.flags.z
        },
        _ => unreachable!()
    }
}

pub fn jump(cpu: &mut CPU, cond: bool) {
    if cond {
        if let JumpType::DirWithinSegmentShort(disp) = cpu.instr.jump_type {
            cpu.ip = cpu.ip.wrapping_add(disp as u16)
        }
        cpu.instr.cycles += 16;
    } else {
        cpu.instr.cycles += 4;
    }
}