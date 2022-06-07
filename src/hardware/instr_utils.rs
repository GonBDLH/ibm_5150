use super::{cpu::CPU, bus::Bus, cpu_utils::{to_u16, sign_extend}};

pub struct Instruction {
    pub opcode: Opcode,
    pub operand1: Operand,
    pub operand2: Operand,

    // True     R/M -> R
    // False    R   -> R/M
    pub direction: Direction,
    pub data_length: Length,
    pub addr_mode: AddrMode,
    
    // Offset de la direccion en caso de que se lea memoria
    pub disp: u16,
    pub ea_cycles: u8,

    pub cycles: u8,
}

impl Default for Instruction {
    fn default() -> Self {
        Self { 
            opcode: Opcode::None, 
            operand1: Operand::None, 
            operand2: Operand::None, 
            
            direction: Direction::None,
            data_length: Length::None,
            addr_mode: AddrMode::None,

            disp: 0x0000,
            ea_cycles: 0x00,

            cycles: 0 
        }
    }
}

#[derive(Clone, Copy)]
pub enum AddrMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    None
}

#[derive(Clone, Copy)]
pub enum Length {
    Byte,
    Word,
    None,
}

impl Length {
    pub fn new(val: u8) -> Self {
        if val & 0x01 != 0 {
            Length::Word
        } else {
            Length::Byte
        }
    }
}

pub enum Opcode {
    None,
    MOV,

}

#[derive(Clone, Copy)]
pub enum Operand {
    AL,
    BL,
    CL,
    DL,
    AH,
    BH,
    CH,
    DH,
    AX,
    BX,
    CX,
    DX,
    SI,
    DI,
    BP,
    SP,
    CS,
    DS,
    ES,
    SS,
    BXSI,
    BXDI,
    BPSI,
    BPDI,
    DispBXSI,
    DispBXDI,
    DispBPSI,
    DispBPDI,
    DispSI,
    DispDI,
    DispBP,
    DispBX,
    Disp,
    None,
}

pub enum Direction {
    ToReg,
    FromReg,
    None,
}

impl Direction {
    pub fn new(val: u8) -> Self {
        if val & 0x02 != 0 {
            Direction::ToReg
        } else {
            Direction::FromReg
        }
    }
}

fn decode_mod(cpu: &mut CPU, bus: &mut Bus, operand: u8) {
    match operand & 0b11000000 {
        0b00000000 => {
            cpu.op.addr_mode = AddrMode::Mode0;
        },
        0b01000000 => {
            cpu.op.addr_mode = AddrMode::Mode1;
        },
        0b10000000 => {
            cpu.op.addr_mode = AddrMode::Mode2;
        },
        0b11000000 => {
            cpu.op.addr_mode = AddrMode::Mode3;
        },
        _ => unreachable!("Aqui no deberia entrar"),
    }
}

fn get_reg_operand(reg: u8, length: Length) -> Operand {
    match reg {
        0b000 => {
            match length {
                Length::Byte => Operand::AL,
                Length::Word => Operand::AX,
                _ => unreachable!(),
            }
        },
        0b001 => {
            match length {
                Length::Byte => Operand::CL,
                Length::Word => Operand::CX,
                _ => unreachable!(),
            }
        },
        0b010 => {
            match length {
                Length::Byte => Operand::DL,
                Length::Word => Operand::DX,
                _ => unreachable!(),
            }
        },
        0b011 => {
            match length {
                Length::Byte => Operand::BL,
                Length::Word => Operand::BX,
                _ => unreachable!(),
            }
        },
        0b100 => {
            match length {
                Length::Byte => Operand::AH,
                Length::Word => Operand::SP,
                _ => unreachable!(),
            }
        },
        0b101 => {
            match length {
                Length::Byte => Operand::CH,
                Length::Word => Operand::BP,
                _ => unreachable!(),
            }
        },
        0b110 => {
            match length {
                Length::Byte => Operand::DH,
                Length::Word => Operand::SI,
                _ => unreachable!(),
            }
        },
        0b111 => {
            match length {
                Length::Byte => Operand::BH,
                Length::Word => Operand::DI,
                _ => unreachable!(),
            }
        },
        _ => unreachable!("Aqui no deberia entrar nunca")
    }
}

fn get_mem_operand(cpu: &mut CPU, bus: &mut Bus, rm: u8, mode: AddrMode) -> Operand {
    match mode {
        AddrMode::Mode0 => {
            match rm {
                0b000 => {
                    cpu.op.disp = cpu.bx.get_x().wrapping_add(cpu.si);
                    cpu.op.ea_cycles = 7;
                    Operand::BXSI
                },
                0b001 => {
                    cpu.op.disp = cpu.bx.get_x().wrapping_add(cpu.di);
                    cpu.op.ea_cycles = 8;
                    Operand::BXDI
                },
                0b010 => {
                    cpu.op.disp = cpu.bp.wrapping_add(cpu.si);
                    cpu.op.ea_cycles = 8;
                    Operand::BPSI
                },
                0b011 => {
                    cpu.op.disp = cpu.bp.wrapping_add(cpu.di);
                    cpu.op.ea_cycles = 7;
                    Operand::BPDI
                },
                0b100 => {
                    cpu.op.disp = cpu.si;
                    cpu.op.ea_cycles = 5;
                    Operand::SI
                },
                0b101 => {
                    cpu.op.disp = cpu.di;
                    cpu.op.ea_cycles = 5;
                    Operand::DI
                },
                0b110 => {
                    let disp_low = cpu.fetch(bus);
                    let disp_high = cpu.fetch(bus);
                    cpu.op.disp = to_u16(disp_low, disp_high);
                    cpu.op.ea_cycles = 6;
                    Operand::Disp
                },
                0b111 => {
                    cpu.op.disp = cpu.bx.get_x();
                    cpu.op.ea_cycles = 5;
                    Operand::BX
                },
                _ => unreachable!("Aqui no deberia entrar nunca")
            }
        },
        AddrMode::Mode1 | AddrMode::Mode2 => {
            let disp = match mode {
                AddrMode::Mode1 => {
                    let readed = cpu.fetch(bus);
                    sign_extend(readed)
                },
                AddrMode::Mode2 => {
                    let disp_low = cpu.fetch(bus);
                    let disp_high = cpu.fetch(bus);
                    to_u16(disp_low, disp_high)
                },
                _ => unreachable!(),
            };
            
            match rm {
                0b000 => {
                    cpu.op.disp = cpu.bx.get_x().wrapping_add(cpu.si).wrapping_add(disp);
                    cpu.op.ea_cycles = 11;
                    Operand::DispBXSI
                },
                0b001 => {
                    cpu.op.disp = cpu.bx.get_x().wrapping_add(cpu.di).wrapping_add(disp);
                    cpu.op.ea_cycles = 12;
                    Operand::DispBXDI
                },
                0b010 => {
                    cpu.op.disp = cpu.bp.wrapping_add(cpu.si).wrapping_add(disp);
                    cpu.op.ea_cycles = 12;
                    Operand::DispBPSI
                },
                0b011 => {
                    cpu.op.disp = cpu.bp.wrapping_add(cpu.di).wrapping_add(disp);
                    cpu.op.ea_cycles = 11;
                    Operand::DispBPDI
                },
                0b100 => {
                    cpu.op.disp = cpu.si.wrapping_add(disp);
                    cpu.op.ea_cycles = 9;
                    Operand::DispSI
                },
                0b101 => {
                    cpu.op.disp = cpu.di.wrapping_add(disp);
                    cpu.op.ea_cycles = 9;
                    Operand::DispDI
                },
                0b110 => {
                    cpu.op.disp = cpu.bp.wrapping_add(disp);
                    cpu.op.ea_cycles = 9;
                    Operand::DispBP
                },
                0b111 => {
                    cpu.op.disp = cpu.bx.get_x().wrapping_add(disp);
                    cpu.op.ea_cycles = 9;
                    Operand::DispBX
                },
                _ => unreachable!("Aqui no deberia entrar nunca")
            }
        },
        _ => unreachable!(),
    }
}

pub fn decode_mod_reg_rm(cpu: &mut CPU, bus: &mut Bus) {
    let operand = cpu.fetch(bus);
    decode_mod(cpu, bus, operand);

    match cpu.op.direction {
        Direction::ToReg => {
            cpu.op.operand1 = get_reg_operand((operand & 0b00111000) >> 3, cpu.op.data_length);
            cpu.op.operand2 = match cpu.op.addr_mode {
                AddrMode::Mode0 | AddrMode::Mode1 | AddrMode::Mode2 => {
                    get_mem_operand(cpu, bus, operand & 0b00000111, cpu.op.addr_mode)
                },
                AddrMode::Mode3 => {
                    get_reg_operand(operand & 0b00000111, cpu.op.data_length)
                },
                _ => unreachable!("Aqui no deberia entrar"),
            }
        },
        Direction::FromReg => {
            cpu.op.operand1 = match cpu.op.addr_mode {
                AddrMode::Mode0 | AddrMode::Mode1 | AddrMode::Mode2 => {
                    get_mem_operand(cpu, bus, operand & 0b00000111, cpu.op.addr_mode)
                },
                AddrMode::Mode3 => {
                    get_reg_operand(operand & 0b00000111, cpu.op.data_length)
                },
                _ => unreachable!("Aqui no deberia entrar"),
            };
            cpu.op.operand2 = get_reg_operand((operand & 0b00111000) >> 3, cpu.op.data_length);
        },
        _ => unreachable!(),
    }


}