use std::fmt::{Debug, Display};

use super::cpu_utils::*;
use super::Bus;
use super::CPU;

#[derive(Copy, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand1: OperandType,
    pub operand2: OperandType,

    pub direction: Direction,
    pub data_length: Length,
    pub addr_mode: AddrMode,

    // Offset de la direccion en caso de que se lea memoria
    pub segment: Segment,
    pub offset: u16,
    pub ea_cycles: u32,
    pub has_segment_prefix: bool,

    // Valor inmediato en caso de que lo haya
    // pub imm: u16,

    // En caso de que sea una instr I/O
    pub port: u16,

    // Tipo de JMP/CALL
    pub jump_type: JumpType,
    pub ip: u16,
    pub cs: u16,

    // Tipo de RET
    pub ret_type: RetType,

    // INT
    pub sw_int_type: u8,

    pub repetition_prefix: RepetitionPrefix,
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

            segment: Segment::DS,
            offset: 0x0000,
            ea_cycles: 0x00,
            has_segment_prefix: false,

            //imm: 0,
            port: 0,

            jump_type: JumpType::None,
            ip: 0,
            cs: 0,

            ret_type: RetType::None,

            sw_int_type: 0,

            repetition_prefix: RepetitionPrefix::None,
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = InstructionStrBuilder::new(self)
            .add_opcode()
            .add_op1()
            .add_op2()
            .build();

        write!(f, "{}", s)
    }
}

pub struct InstructionDbg {
    instr: Instruction,
    bytecode: String,
}

impl InstructionDbg {
    pub fn new(instr: &Instruction, bytecode: String) -> Self {
        Self {
            instr: *instr,
            bytecode,
        }
    }
}

impl Display for InstructionDbg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:>10}: {}", self.bytecode, self.instr)
    }
}

struct InstructionStrBuilder<'a> {
    s: String,
    instr: &'a Instruction,
}

impl<'a> InstructionStrBuilder<'a> {
    pub fn new(instr: &'a Instruction) -> Self {
        InstructionStrBuilder {
            s: String::new(),
            instr,
        }
    }

    pub fn add_opcode(mut self) -> Self {
        self.s.push_str(&format!("{}", self.instr.opcode));
        self
    }

    pub fn add_op1(mut self) -> Self {
        if self.instr.operand1 == OperandType::None {
            if self.instr.port != 0 {
                self.s.push_str(&format!(" {}", self.instr.port));
            } else if self.instr.jump_type != JumpType::None {
                let dir = match self.instr.jump_type {
                    JumpType::DirIntersegment(off, seg) => {
                        format!(" {:05X}", ((seg as usize) << 4) + off as usize)
                    }
                    JumpType::DirWithinSegment(off) => format!(
                        " {:05X}",
                        ((self.instr.cs as usize) << 4)
                            + (self.instr.ip.wrapping_add(off) as usize)
                    ),
                    JumpType::DirWithinSegmentShort(off) => format!(
                        " {:05X}",
                        ((self.instr.cs as usize) << 4)
                            + (self.instr.ip.wrapping_add(sign_extend(off)) as usize)
                    ),
                    JumpType::IndIntersegment(seg, off) => {
                        format!(" {:05X}", ((seg as usize) << 4) + off as usize)
                    }
                    JumpType::IndWithinSegment(off) => {
                        format!(" {:05X}", ((self.instr.cs as usize) << 4) + off as usize)
                    }
                    _ => unreachable!(),
                };

                self.s.push_str(&dir);
            } else if self.instr.sw_int_type != 0 {
                self.s.push_str(&format!(" {:02X}", self.instr.sw_int_type));
            }
        } else {
            let segm = if self.instr.segment == Segment::DS {
                String::from("")
            } else {
                format!("{}", self.instr.segment)
            };

            match self.instr.operand1 {
                OperandType::Memory(a) => {
                    self.s.push_str(&format!(" {}[{}]", segm, a));
                }
                _ => self.s.push_str(&format!(" {}", self.instr.operand1)),
            }
        }

        self
    }

    pub fn add_op2(mut self) -> Self {
        if self.instr.operand2 != OperandType::None {
            let segm = if self.instr.segment == Segment::DS {
                String::from("")
            } else {
                format!("{}", self.instr.segment)
            };

            match self.instr.operand2 {
                OperandType::Memory(a) => {
                    self.s.push_str(&format!(",{}[{}]", segm, a));
                }
                _ => self.s.push_str(&format!(",{}", self.instr.operand2)),
            }
        }

        self
    }

    pub fn build(self) -> String {
        self.s
    }
}

#[derive(PartialEq, Clone, Copy)]
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
    None,
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

    pub fn get_num_bits(&self) -> u16 {
        match self {
            Length::Byte => 8,
            Length::Word => 16,
            Length::None => 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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
    SALC,
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
    HLT,
    NOP,
    SETMO,
    SETMOC,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Opcode::None => "NONE",
            Opcode::MOV => "MOV",
            Opcode::PUSH => "PUSH",
            Opcode::POP => "POP",
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
            Opcode::SALC => "SALC",
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
            Opcode::HLT => "HLT",
            Opcode::NOP => "NOP",
            Opcode::SETMO => "SETMO",
            Opcode::SETMOC => "SETMOC",
        };
        write!(f, "{}", val)
    }
}

#[derive(Clone, Copy, PartialEq)]
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
    BXSI,
    BXDI,
    BPSI,
    BPDI,
    DispBXSI(u16),
    DispBXDI(u16),
    DispBPSI(u16),
    DispBPDI(u16),
    DispSI(u16),
    DispDI(u16),
    DispBP(u16),
    DispBX(u16),
    Disp(u16),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Operand::AL => String::from("AL"),
            Operand::BL => String::from("BL"),
            Operand::CL => String::from("CL"),
            Operand::DL => String::from("DL"),
            Operand::AH => String::from("AH"),
            Operand::BH => String::from("BH"),
            Operand::CH => String::from("CH"),
            Operand::DH => String::from("DH"),
            Operand::AX => String::from("AX"),
            Operand::BX => String::from("BX"),
            Operand::CX => String::from("CX"),
            Operand::DX => String::from("DX"),
            Operand::SI => String::from("SI"),
            Operand::DI => String::from("DI"),
            Operand::BP => String::from("BP"),
            Operand::SP => String::from("SP"),
            Operand::BXSI => String::from("BX+SI"),
            Operand::BXDI => String::from("BX+DI"),
            Operand::BPSI => String::from("BP+SI"),
            Operand::BPDI => String::from("BP+DI"),
            Operand::DispBXSI(d) => format!("{:04X}+BX+SI", d),
            Operand::DispBXDI(d) => format!("{:04X}+BX+DI", d),
            Operand::DispBPSI(d) => format!("{:04X}+BP+SI", d),
            Operand::DispBPDI(d) => format!("{:04X}+BP+DI", d),
            Operand::DispSI(d) => format!("{:04X}+SI", d),
            Operand::DispDI(d) => format!("{:04X}+DI", d),
            Operand::DispBP(d) => format!("{:04X}+BP", d),
            Operand::DispBX(d) => format!("{:04X}+BX", d),
            Operand::Disp(d) => format!("{:04X}", d),
        };
        write!(f, "{}", val)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Segment {
    CS,
    DS,
    ES,
    SS,
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Segment::CS => String::from("CS"),
            Segment::DS => String::from("DS"),
            Segment::ES => String::from("ES"),
            Segment::SS => String::from("SS"),
        };
        write!(f, "{}", val)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum OperandType {
    Register(Operand),
    SegmentRegister(Segment),
    Memory(Operand),
    Immediate(u16),
    None,
}

impl Display for OperandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperandType::Register(r) => write!(f, "{}", r),
            OperandType::SegmentRegister(r) => write!(f, "{}", r),
            OperandType::Memory(r) => write!(f, "[{}]", r),
            OperandType::Immediate(r) => write!(f, "{:X}", r),
            OperandType::None => write!(f, ""),
        }
    }
}

impl Debug for OperandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperandType::Register(r) => write!(f, "{}", r),
            OperandType::SegmentRegister(r) => write!(f, "{}", r),
            OperandType::Memory(r) => write!(f, "[{}]", r),
            OperandType::Immediate(r) => write!(f, "{:X}", r),
            OperandType::None => write!(f, ""),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
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

#[derive(Copy, Clone)]
pub enum RetType {
    NearAdd(u16),
    Near,
    Far,
    FarAdd(u16),
    None,
}

#[derive(Copy, Clone, PartialEq)]
pub enum JumpType {
    DirIntersegment(u16, u16),
    DirWithinSegment(u16),
    DirWithinSegmentShort(u8),
    IndIntersegment(u16, u16),
    IndWithinSegment(u16),
    None,
}

impl Display for JumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JumpType::None => write!(f, ""),
            JumpType::DirIntersegment(a, b) => write!(f, "{:04X}:{:04X}", b, a),
            JumpType::DirWithinSegment(a) => write!(f, "CS:{:04X}", a),
            JumpType::DirWithinSegmentShort(a) => write!(f, "{:02X}", a),
            JumpType::IndIntersegment(_, _) => write!(f, "II"),
            JumpType::IndWithinSegment(_) => write!(f, "IWS"),
        }
    }
}

pub fn decode_mod(operand: u8) -> AddrMode {
    match operand & 0b11000000 {
        0b00000000 => AddrMode::Mode0,
        0b01000000 => AddrMode::Mode1,
        0b10000000 => AddrMode::Mode2,
        0b11000000 => AddrMode::Mode3,
        _ => unreachable!("Aqui no deberia entrar"),
    }
}

pub fn decode_reg(operand: u8, pos: u8, length: Length) -> OperandType {
    assert!(pos < 8);
    let reg = (operand >> pos) & 0x07;

    match reg {
        0b000 => match length {
            Length::Byte => OperandType::Register(Operand::AL),
            Length::Word => OperandType::Register(Operand::AX),
            _ => unreachable!(),
        },
        0b001 => match length {
            Length::Byte => OperandType::Register(Operand::CL),
            Length::Word => OperandType::Register(Operand::CX),
            _ => unreachable!(),
        },
        0b010 => match length {
            Length::Byte => OperandType::Register(Operand::DL),
            Length::Word => OperandType::Register(Operand::DX),
            _ => unreachable!(),
        },
        0b011 => match length {
            Length::Byte => OperandType::Register(Operand::BL),
            Length::Word => OperandType::Register(Operand::BX),
            _ => unreachable!(),
        },
        0b100 => match length {
            Length::Byte => OperandType::Register(Operand::AH),
            Length::Word => OperandType::Register(Operand::SP),
            _ => unreachable!(),
        },
        0b101 => match length {
            Length::Byte => OperandType::Register(Operand::CH),
            Length::Word => OperandType::Register(Operand::BP),
            _ => unreachable!(),
        },
        0b110 => match length {
            Length::Byte => OperandType::Register(Operand::DH),
            Length::Word => OperandType::Register(Operand::SI),
            _ => unreachable!(),
        },
        0b111 => match length {
            Length::Byte => OperandType::Register(Operand::BH),
            Length::Word => OperandType::Register(Operand::DI),
            _ => unreachable!(),
        },
        _ => unreachable!("Aqui no deberia entrar nunca"),
    }
}

pub fn decode_mem(
    cpu: &mut CPU,
    bus: &mut Bus,
    operand: u8,
    pos: u8,
    mode: AddrMode,
) -> OperandType {
    assert!(pos < 8);
    let rm = (operand >> pos) & 0x07;

    match mode {
        AddrMode::Mode0 => match rm {
            0b000 => {
                cpu.instr.offset = cpu.bx.get_x().wrapping_add(cpu.si);
                cpu.instr.ea_cycles = 7;
                OperandType::Memory(Operand::BXSI)
            }
            0b001 => {
                cpu.instr.offset = cpu.bx.get_x().wrapping_add(cpu.di);
                cpu.instr.ea_cycles = 8;
                OperandType::Memory(Operand::BXDI)
            }
            0b010 => {
                cpu.instr.offset = cpu.bp.wrapping_add(cpu.si);
                cpu.instr.ea_cycles = 8;
                if !cpu.instr.has_segment_prefix {
                    cpu.instr.segment = Segment::SS;
                }
                OperandType::Memory(Operand::BPSI)
            }
            0b011 => {
                cpu.instr.offset = cpu.bp.wrapping_add(cpu.di);
                cpu.instr.ea_cycles = 7;
                if !cpu.instr.has_segment_prefix {
                    cpu.instr.segment = Segment::SS;
                }
                OperandType::Memory(Operand::BPDI)
            }
            0b100 => {
                cpu.instr.offset = cpu.si;
                cpu.instr.ea_cycles = 5;
                OperandType::Memory(Operand::SI)
            }
            0b101 => {
                cpu.instr.offset = cpu.di;
                cpu.instr.ea_cycles = 5;
                OperandType::Memory(Operand::DI)
            }
            0b110 => {
                let disp_low = cpu.fetch(bus);
                let disp_high = cpu.fetch(bus);

                // cpu.instr.segment = Segment::Dir(disp_high as u16 + ds);
                cpu.instr.offset = to_u16(disp_low, disp_high);
                cpu.instr.ea_cycles = 6;

                // TODO Esto no se si estara mal
                OperandType::Memory(Operand::Disp(to_u16(disp_low, disp_high)))
            }
            0b111 => {
                cpu.instr.offset = cpu.bx.get_x();
                cpu.instr.ea_cycles = 5;
                OperandType::Memory(Operand::BX)
            }
            _ => unreachable!("Aqui no deberia entrar nunca"),
        },
        AddrMode::Mode1 | AddrMode::Mode2 => {
            let disp = match mode {
                AddrMode::Mode1 => {
                    let readed = cpu.fetch(bus);
                    sign_extend(readed)
                }
                AddrMode::Mode2 => {
                    let disp_low = cpu.fetch(bus);
                    let disp_high = cpu.fetch(bus);
                    to_u16(disp_low, disp_high)
                }
                _ => unreachable!(),
            };

            match rm {
                0b000 => {
                    cpu.instr.offset = cpu.bx.get_x().wrapping_add(cpu.si).wrapping_add(disp);
                    cpu.instr.ea_cycles = 11;
                    OperandType::Memory(Operand::DispBXSI(disp))
                }
                0b001 => {
                    cpu.instr.offset = cpu.bx.get_x().wrapping_add(cpu.di).wrapping_add(disp);
                    cpu.instr.ea_cycles = 12;
                    OperandType::Memory(Operand::DispBXDI(disp))
                }
                0b010 => {
                    cpu.instr.offset = cpu.bp.wrapping_add(cpu.si).wrapping_add(disp);
                    cpu.instr.ea_cycles = 12;
                    if !cpu.instr.has_segment_prefix {
                        cpu.instr.segment = Segment::SS;
                    }
                    OperandType::Memory(Operand::DispBPSI(disp))
                }
                0b011 => {
                    cpu.instr.offset = cpu.bp.wrapping_add(cpu.di).wrapping_add(disp);
                    cpu.instr.ea_cycles = 11;
                    if !cpu.instr.has_segment_prefix {
                        cpu.instr.segment = Segment::SS;
                    }
                    OperandType::Memory(Operand::DispBPDI(disp))
                }
                0b100 => {
                    cpu.instr.offset = cpu.si.wrapping_add(disp);
                    cpu.instr.ea_cycles = 9;
                    OperandType::Memory(Operand::DispSI(disp))
                }
                0b101 => {
                    cpu.instr.offset = cpu.di.wrapping_add(disp);
                    cpu.instr.ea_cycles = 9;
                    OperandType::Memory(Operand::DispDI(disp))
                }
                0b110 => {
                    cpu.instr.offset = cpu.bp.wrapping_add(disp);
                    cpu.instr.ea_cycles = 9;
                    if !cpu.instr.has_segment_prefix {
                        cpu.instr.segment = Segment::SS;
                    }
                    OperandType::Memory(Operand::DispBP(disp))
                }
                0b111 => {
                    cpu.instr.offset = cpu.bx.get_x().wrapping_add(disp);
                    cpu.instr.ea_cycles = 9;
                    OperandType::Memory(Operand::DispBX(disp))
                }
                _ => unreachable!("Aqui no deberia entrar nunca"),
            }
        }
        _ => unreachable!(),
    }
}

pub fn decode_rm(cpu: &mut CPU, bus: &mut Bus, operand: u8, rm_pos: u8) -> OperandType {
    match cpu.instr.addr_mode {
        AddrMode::Mode0 | AddrMode::Mode1 | AddrMode::Mode2 => {
            decode_mem(cpu, bus, operand, rm_pos, cpu.instr.addr_mode)
        }
        AddrMode::Mode3 => decode_reg(operand, rm_pos, cpu.instr.data_length),
        _ => unreachable!("Aqui no deberia entrar"),
    }
}

pub fn decode_segment(operand: u8, pos: u8) -> OperandType {
    assert!(pos < 8);
    let reg = (operand >> pos) & 0x03;

    match reg {
        0b00 => OperandType::SegmentRegister(Segment::ES),
        0b01 => OperandType::SegmentRegister(Segment::CS),
        0b10 => OperandType::SegmentRegister(Segment::SS),
        0b11 => OperandType::SegmentRegister(Segment::DS),
        _ => unreachable!(),
    }
}

pub fn decode_mod_reg_rm(cpu: &mut CPU, bus: &mut Bus, operand: u8) {
    cpu.instr.addr_mode = decode_mod(operand);

    match cpu.instr.direction {
        Direction::ToReg => {
            cpu.instr.operand1 = decode_reg(operand, 3, cpu.instr.data_length);
            cpu.instr.operand2 = decode_rm(cpu, bus, operand, 0);
        }
        Direction::FromReg => {
            cpu.instr.operand1 = decode_rm(cpu, bus, operand, 0);
            cpu.instr.operand2 = decode_reg(operand, 3, cpu.instr.data_length);
        }
        _ => unreachable!(),
    }
}

pub fn decode_mod_n_rm(cpu: &mut CPU, bus: &mut Bus, operand: u8) {
    cpu.instr.addr_mode = decode_mod(operand);
    cpu.instr.operand1 = decode_rm(cpu, bus, operand, 0)
}

pub fn read_imm(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    match cpu.instr.data_length {
        Length::Byte => cpu.fetch(bus) as u16,
        Length::Word => to_u16(cpu.fetch(bus), cpu.fetch(bus)),
        _ => unreachable!(),
    }
}

pub fn read_imm_addres(cpu: &mut CPU, bus: &mut Bus) {
    cpu.instr.offset = to_u16(cpu.fetch(bus), cpu.fetch(bus));
}

pub fn decode_jmp(instr: &mut Instruction, opcode: Opcode, jump_type: JumpType) {
    instr.opcode = opcode;
    instr.jump_type = jump_type;
}
