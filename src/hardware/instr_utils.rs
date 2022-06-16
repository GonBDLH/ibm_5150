use super::{cpu::CPU, bus::Bus, cpu_utils::{to_u16, sign_extend}};

pub struct Instruction {
    pub opcode: Opcode,
    pub operand1: OperandType,
    pub operand2: OperandType,

    // True     R/M -> R
    // False    R   -> R/M
    pub direction: Direction,
    pub data_length: Length,
    pub addr_mode: AddrMode,
    
    // Offset de la direccion en caso de que se lea memoria
    pub segment: Operand,
    pub offset: u16,
    pub ea_cycles: u8,

    // Valor inmediato en caso de que lo haya
    pub imm: u16,

    // Tipo de JMP/CALL
    pub jump_type: JumpType,

    pub cycles: u8,
}

impl Default for Instruction {
    fn default() -> Self {
        Self { 
            opcode: Opcode::None, 
            operand1: OperandType::None, 
            operand2: OperandType::None, 
            
            direction: Direction::None,
            data_length: Length::None,
            addr_mode: AddrMode::None,

            segment: Operand::None,
            offset: 0x0000,
            ea_cycles: 0x00,

            imm: 0,

            jump_type: JumpType::None,

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

#[derive(Clone, Copy, PartialEq)]
pub enum Length {
    Byte,
    Word,
    None,
}

impl Length {
    pub fn new(val: u8, pos: u8) -> Self {
        assert!(pos < 8);
        if val & (0x01 << pos) != 0 {
            Length::Word
        } else {
            Length::Byte
        }
    }
}

pub enum Opcode {
    None,
    MOV,
    PUSH,
    POP,
    XCHG,
    IN,
    OUT,
    XLAT,
    LEA,
    LDS,
    LES,
    LAHF,
    SAHF,
    PUSHF,
    POPF,
    ADD,
    ADC,
    INC,
    AAA,
    DAA,
    SUB,
    SBB,
    DEC,
    NEG,
    CMP,
    AAS,
    DAS,
    MUL,
    IMUL,
    AAM,
    DIV,
    IDIV,
    AAD,
    CBW,
    CWD,
    NOT,
    SALSHL,
    SHR,
    SAR,
    ROL,
    ROR,
    RCL,
    RCR,
    AND,
    TEST,
    OR,
    XOR,
    CALL,
    JMP,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Operand {
    None = 0,
    AL = 1,
    BL = 2,
    CL = 3,
    DL = 4,
    AH = 5,
    BH = 6,
    CH = 7,
    DH = 8,
    AX = 9,
    BX = 10,
    CX = 11,
    DX = 12,
    SI = 13,
    DI = 14,
    BP = 15,
    SP = 16,
    CS = 17,
    DS = 18,
    ES = 19,
    SS = 20,
    BXSI = 21,
    BXDI = 22,
    BPSI = 23,
    BPDI = 24,
    DispBXSI = 25,
    DispBXDI = 26,
    DispBPSI = 27,
    DispBPDI = 28,
    DispSI = 29,
    DispDI = 30,
    DispBP = 31,
    DispBX = 32,
    Disp = 33,
}

#[derive(Clone, Copy)]
pub enum OperandType {
    Register(Operand),
    SegmentRegister(Operand),
    Memory(Operand),
    Immediate,
    None,
}

#[derive(PartialEq)]
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

pub enum JumpType {
    Long(u16, u16),
    // Short(u16),
    None,
}

pub fn decode_mod(operand: u8) -> AddrMode {
    match operand & 0b11000000 {
        0b00000000 => {
            AddrMode::Mode0
        },
        0b01000000 => {
            AddrMode::Mode1
        },
        0b10000000 => {
            AddrMode::Mode2
        },
        0b11000000 => {
            AddrMode::Mode3
        },
        _ => unreachable!("Aqui no deberia entrar"),
    }
}

pub fn decode_reg(operand: u8, pos: u8, length: Length) -> OperandType {
    assert!(pos < 8);
    let reg = (operand >> pos) & 0x07;

    match reg {
        0b000 => {
            match length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            }
        },
        0b001 => {
            match length {
                Length::Byte => OperandType::Register(Operand::CL),
                Length::Word => OperandType::Register(Operand::CX),
                _ => unreachable!(),
            }
        },
        0b010 => {
            match length {
                Length::Byte => OperandType::Register(Operand::DL),
                Length::Word => OperandType::Register(Operand::DX),
                _ => unreachable!(),
            }
        },
        0b011 => {
            match length {
                Length::Byte => OperandType::Register(Operand::BL),
                Length::Word => OperandType::Register(Operand::BX),
                _ => unreachable!(),
            }
        },
        0b100 => {
            match length {
                Length::Byte => OperandType::Register(Operand::AH),
                Length::Word => OperandType::Register(Operand::SP),
                _ => unreachable!(),
            }
        },
        0b101 => {
            match length {
                Length::Byte => OperandType::Register(Operand::CH),
                Length::Word => OperandType::Register(Operand::BP),
                _ => unreachable!(),
            }
        },
        0b110 => {
            match length {
                Length::Byte => OperandType::Register(Operand::DH),
                Length::Word => OperandType::Register(Operand::SI),
                _ => unreachable!(),
            }
        },
        0b111 => {
            match length {
                Length::Byte => OperandType::Register(Operand::BH),
                Length::Word => OperandType::Register(Operand::DI),
                _ => unreachable!(),
            }
        },
        _ => unreachable!("Aqui no deberia entrar nunca")
    }
}

pub fn decode_mem(cpu: &mut CPU, bus: &mut Bus, operand: u8, pos: u8, mode: AddrMode) -> OperandType {
    assert!(pos < 8);
    let rm = (operand >> pos) & 0x07;

    match mode {
        AddrMode::Mode0 => {
            match rm {
                0b000 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.bx.get_x().wrapping_add(cpu.si);
                    cpu.instr.ea_cycles = 7;
                    OperandType::Memory(Operand::BXSI)
                },
                0b001 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.bx.get_x().wrapping_add(cpu.di);
                    cpu.instr.ea_cycles = 8;
                    OperandType::Memory(Operand::BXDI)
                },
                0b010 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::SS};
                    cpu.instr.offset = cpu.bp.wrapping_add(cpu.si);
                    cpu.instr.ea_cycles = 8;
                    OperandType::Memory(Operand::BPSI)
                },
                0b011 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::SS};
                    cpu.instr.offset = cpu.bp.wrapping_add(cpu.di);
                    cpu.instr.ea_cycles = 7;
                    OperandType::Memory(Operand::BPDI)
                },
                0b100 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.si;
                    cpu.instr.ea_cycles = 5;
                    OperandType::Memory(Operand::SI)
                },
                0b101 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.di;
                    cpu.instr.ea_cycles = 5;
                    OperandType::Memory(Operand::DI)
                },
                0b110 => {
                    let disp_low = cpu.fetch(bus);
                    let disp_high = cpu.fetch(bus);
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = to_u16(disp_low, disp_high);
                    cpu.instr.ea_cycles = 6;
                    OperandType::Memory(Operand::Disp)
                },
                0b111 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.bx.get_x();
                    cpu.instr.ea_cycles = 5;
                    OperandType::Memory(Operand::BX)
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
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.bx.get_x().wrapping_add(cpu.si).wrapping_add(disp);
                    cpu.instr.ea_cycles = 11;
                    OperandType::Memory(Operand::DispBXSI)
                },
                0b001 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.bx.get_x().wrapping_add(cpu.di).wrapping_add(disp);
                    cpu.instr.ea_cycles = 12;
                    OperandType::Memory(Operand::DispBXDI)
                },
                0b010 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::SS};
                    cpu.instr.offset = cpu.bp.wrapping_add(cpu.si).wrapping_add(disp);
                    cpu.instr.ea_cycles = 12;
                    OperandType::Memory(Operand::DispBPSI)
                },
                0b011 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::SS};
                    cpu.instr.offset = cpu.bp.wrapping_add(cpu.di).wrapping_add(disp);
                    cpu.instr.ea_cycles = 11;
                    OperandType::Memory(Operand::DispBPDI)
                },
                0b100 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.si.wrapping_add(disp);
                    cpu.instr.ea_cycles = 9;
                    OperandType::Memory(Operand::DispSI)
                },
                0b101 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.di.wrapping_add(disp);
                    cpu.instr.ea_cycles = 9;
                    OperandType::Memory(Operand::DispDI)
                },
                0b110 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::SS};
                    cpu.instr.offset = cpu.bp.wrapping_add(disp);
                    cpu.instr.ea_cycles = 9;
                    OperandType::Memory(Operand::DispBP)
                },
                0b111 => {
                    if cpu.instr.segment == Operand::None {cpu.instr.segment = Operand::DS};
                    cpu.instr.offset = cpu.bx.get_x().wrapping_add(disp);
                    cpu.instr.ea_cycles = 9;
                    OperandType::Memory(Operand::DispBX)
                },
                _ => unreachable!("Aqui no deberia entrar nunca")
            }
        },
        _ => unreachable!(),
    }
}

pub fn decode_rm(cpu: &mut CPU, bus: &mut Bus, operand: u8, rm_pos: u8) -> OperandType {
    match cpu.instr.addr_mode {
        AddrMode::Mode0 | AddrMode::Mode1 | AddrMode::Mode2 => {
            decode_mem(cpu, bus, operand, rm_pos, cpu.instr.addr_mode)
        },
        AddrMode::Mode3 => {
            decode_reg(operand, rm_pos, cpu.instr.data_length)
        },
        _ => unreachable!("Aqui no deberia entrar"),
    }
}

pub fn decode_segment(operand: u8, pos: u8) -> OperandType {
    assert!(pos < 8);
    let reg = (operand >> pos) & 0x03;

    match reg {
        0b00 => OperandType::SegmentRegister(Operand::ES),
        0b01 => OperandType::SegmentRegister(Operand::CS),
        0b10 => OperandType::SegmentRegister(Operand::SS),
        0b11 => OperandType::SegmentRegister(Operand::DS),
        _ => unreachable!(),
    }
}

pub fn decode_mod_reg_rm(cpu: &mut CPU, bus: &mut Bus, operand: u8) {
    cpu.instr.addr_mode = decode_mod(operand);

    match cpu.instr.direction {
        Direction::ToReg => {
            cpu.instr.operand1 = decode_reg(operand, 3, cpu.instr.data_length);
            cpu.instr.operand2 = decode_rm(cpu, bus, operand, 0);
        },
        Direction::FromReg => {
            cpu.instr.operand1 = decode_rm(cpu, bus, operand, 0);
            cpu.instr.operand2 = decode_reg(operand, 3, cpu.instr.data_length);
        },
        _ => unreachable!(),
    }
}

pub fn decode_mod_n_rm(cpu: &mut CPU, bus: &mut Bus, operand: u8) {
    cpu.instr.addr_mode = decode_mod(operand);
    cpu.instr.operand1 = decode_rm(cpu, bus, operand, 0)
}

pub fn read_imm(cpu: &mut CPU, bus: &mut Bus) {
    match cpu.instr.data_length {
        Length::Byte => cpu.instr.imm = cpu.fetch(bus) as u16,
        Length::Word => cpu.instr.imm = to_u16(cpu.fetch(bus), cpu.fetch(bus)),
        _ => unreachable!(),
    }
}

pub fn read_imm_addres(cpu: &mut CPU, bus: &mut Bus) {
    cpu.instr.offset = to_u16(cpu.fetch(bus), cpu.fetch(bus));
    cpu.instr.segment = Operand::DS;
}