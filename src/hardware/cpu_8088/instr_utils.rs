use std::fmt::Display;

use super::CPU;
use super::Bus;
use super::cpu_utils::*;

pub struct Instruction {
    pub opcode: Opcode,
    pub operand1: OperandType,
    pub operand2: OperandType,

    pub direction: Direction,
    pub data_length: Length,
    pub addr_mode: AddrMode,
    
    // Offset de la direccion en caso de que se lea memoria
    pub segment: Operand,
    pub offset: u16,
    pub ea_cycles: u64,

    // Valor inmediato en caso de que lo haya
    pub imm: u16,

    // En caso de que sea una instr I/O
    pub port: u16,

    // Tipo de JMP/CALL
    pub jump_type: JumpType,

    // Tipo de RET
    pub ret_type: RetType,

    pub repetition_prefix: RepetitionPrefix,

    pub cycles: u64,
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

            port: 0,

            jump_type: JumpType::None,

            ret_type: RetType::None,

            repetition_prefix: RepetitionPrefix::None,

            cycles: 0 
        }
    }
}

#[derive(PartialEq)]
pub enum RepetitionPrefix {
    None,
    REPNEZ,
    REPEZ,
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
    MOVSB,
    MOVSW,
    CMPSB,
    CMPSW,
    SCASB,
    SCASW,
    LODSB,
    LODSW,
    STOSB,
    STOSW,
    CALL,
    JMP,
    RET,
    JEJZ,
    JLJNGE,
    JLEJNG,
    JBJNAE,
    JBEJNA,
    JPJPE,
    JO,
    JS,
    JNEJNZ,
    JNLJGE,
    JNLEJG,
    JNBJAE,
    JNBEJA,
    JNPJPO,
    JNO,
    JNS,
    LOOP,
    LOOPZE,
    LOOPNZNE,
    JCXZ,
    INT,
    INTO,
    IRET,
    CLC,
    CMC,
    STC,
    CLD,
    STD,
    CLI,
    STI,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Opcode::None => "NONE",
            Opcode::MOV => "MOV",
            Opcode::PUSH => "PUSH",
            Opcode::POP => "OP",
            Opcode::XCHG => "XCHG",
            Opcode::IN => "IN",
            Opcode::OUT => "OUT",
            Opcode::XLAT => "XLAT",
            Opcode::LEA => "LEA",
            Opcode::LDS => "LDS",
            Opcode::LES => "LES",
            Opcode::LAHF => "LAHF",
            Opcode::SAHF => "SAHF",
            Opcode::PUSHF => "PUSHF",
            Opcode::POPF => "POPF",
            Opcode::ADD => "ADD",
            Opcode::ADC => "ADC",
            Opcode::INC => "INC",
            Opcode::AAA => "AAA",
            Opcode::DAA => "DAA",
            Opcode::SUB => "SUB",
            Opcode::SBB => "SBB",
            Opcode::DEC => "DEC",
            Opcode::NEG => "NEG",
            Opcode::CMP => "CMP",
            Opcode::AAS => "AAS",
            Opcode::DAS => "DAS",
            Opcode::MUL => "MUL",
            Opcode::IMUL => "IMUL",
            Opcode::AAM => "AAM",
            Opcode::DIV => "DIV",
            Opcode::IDIV => "IDIV",
            Opcode::AAD => "AAD",
            Opcode::CBW => "CBW",
            Opcode::CWD => "CWD",
            Opcode::NOT => "NOT",
            Opcode::SALSHL => "SAL/SHL",
            Opcode::SHR => "SHR",
            Opcode::SAR => "SAR",
            Opcode::ROL => "ROL",
            Opcode::ROR => "ROR",
            Opcode::RCL => "RCL",
            Opcode::RCR => "RCR",
            Opcode::AND => "AND",
            Opcode::TEST => "TEST",
            Opcode::OR => "OR",
            Opcode::XOR => "XOR",
            Opcode::MOVSB => "MOVSB",
            Opcode::MOVSW => "MOVSW",
            Opcode::CMPSB => "CMPSB",
            Opcode::CMPSW => "CMPSW",
            Opcode::SCASB => "SCASB",
            Opcode::SCASW => "SCASW",
            Opcode::LODSB => "LODSB",
            Opcode::LODSW => "LODSW",
            Opcode::STOSB => "STOSB",
            Opcode::STOSW => "STOSW",
            Opcode::CALL => "CALL",
            Opcode::JMP => "JMP",
            Opcode::RET => "RET",
            Opcode::JEJZ => "JE/JZ",
            Opcode::JLJNGE => "JL/JNGE",
            Opcode::JLEJNG => "JLE/JNG",
            Opcode::JBJNAE => "JB/JNAE",
            Opcode::JBEJNA => "JBE/JNA",
            Opcode::JPJPE => "JP/JPE",
            Opcode::JO => "JO",
            Opcode::JS => "JS",
            Opcode::JNEJNZ => "JNE/JNZ",
            Opcode::JNLJGE => "JNL/JGE",
            Opcode::JNLEJG => "JNLE/JG",
            Opcode::JNBJAE => "JNB/JAE",
            Opcode::JNBEJA => "JNBE/JA",
            Opcode::JNPJPO => "JNP/JPO",
            Opcode::JNO => "JNO",
            Opcode::JNS => "JNS",
            Opcode::LOOP => "LOOP",
            Opcode::LOOPZE => "LOOPZ/LOOPE",
            Opcode::LOOPNZNE => "LOOPNZ/LOOPNE",
            Opcode::JCXZ => "JCXZ",
            Opcode::INT => "INT",
            Opcode::INTO => "INTO",
            Opcode::IRET => "IRET",
            Opcode::CLC => "CLC",
            Opcode::CMC => "CMC",
            Opcode::STC => "STC",
            Opcode::CLD => "CLD",
            Opcode::STD => "STD",
            Opcode::CLI => "CLI",
            Opcode::STI => "STI",
        };
        write!(f, "{}", val)
    }
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

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Operand::None => "None",
            Operand::AL => "AL",
            Operand::BL => "BL",
            Operand::CL => "CL",
            Operand::DL => "DL",
            Operand::AH => "AH",
            Operand::BH => "BH",
            Operand::CH => "CH",
            Operand::DH => "DH",
            Operand::AX => "AX",
            Operand::BX => "BX",
            Operand::CX => "CX",
            Operand::DX => "DX",
            Operand::SI => "SI",
            Operand::DI => "DI",
            Operand::BP => "BP",
            Operand::SP => "SP",
            Operand::CS => "CS",
            Operand::DS => "DS",
            Operand::ES => "ES",
            Operand::SS => "SS",
            Operand::BXSI => "BXSI",
            Operand::BXDI => "BXDI",
            Operand::BPSI => "BPSI",
            Operand::BPDI => "BPDI",
            Operand::DispBXSI => "DispBXSI",
            Operand::DispBXDI => "DispBXDI",
            Operand::DispBPSI => "DispBPSI",
            Operand::DispBPDI => "DispBPDI",
            Operand::DispSI => "DispSI",
            Operand::DispDI => "DispDI",
            Operand::DispBP => "DispBP",
            Operand::DispBX => "DispBX",
            Operand::Disp => "Disp",
        };
        write!(f, "{}", val)
    }
}

#[derive(Clone, Copy)]
pub enum OperandType {
    Register(Operand),
    SegmentRegister(Operand),
    Memory(Operand),
    Immediate,
    None,
}

impl Display for OperandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperandType::Register(r) => write!(f, "{}", r),
            OperandType::SegmentRegister(r) => write!(f, "{}", r),
            OperandType::Memory(r) => write!(f, "{}", r),
            OperandType::Immediate => write!(f, "Imm"),
            OperandType::None => write!(f, "None"),
        }
    }
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

pub enum RetType {
    NearAdd(u16),
    Near,
    Far,
    FarAdd(u16),
    None
}

pub enum JumpType {
    DirIntersegment(u16, u16),
    DirWithinSegment(u16),
    DirWithinSegmentShort(u8),
    IndIntersegment,
    IndWithinSegment,
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