mod decode;

use std::collections::HashMap;

use crate::hardware::cpu_8088::{
    cpu_utils::{sign_extend, to_u16},
    instr_utils::{
        decode_mod, decode_reg, AddrMode, Direction, Instruction, Length, Operand, OperandType,
    },
    CPU,
};

#[derive(Default)]
pub struct Disassembler {
    cache: HashMap<usize, (String, String, usize)>,

    op_string: String,

    ip: usize,
    cs: usize,
}

impl Disassembler {
    pub fn run(&mut self, iters: usize, mem: &[u8], cpu: &CPU) -> Vec<(String, String, usize)> {
        self.ip = cpu.ip as usize;
        self.cs = cpu.cs as usize;

        let mut instr_vec = Vec::with_capacity(iters);

        for _ in 0..iters {
            self.op_string.clear();
            let ea = ((self.cs << 4).wrapping_add(self.ip)) & 0xFFFFF;

            if let Some(v) = self.cache.get(&ea) {
                instr_vec.push((v.0.clone(), v.1.clone(), ea));
                self.ip += v.2;
            } else {
                let op = self.fetch(mem);
                let mut instr = Instruction::default();
                self.decode(&mut instr, mem, op, cpu);
                let ea_dif = (((self.cs << 4) + self.ip) & 0xFFFFF).wrapping_sub(ea);

                instr_vec.push((format!("{}", instr), self.op_string.clone(), ea));
                self.cache.insert(
                    ea,
                    (
                        String::from(format!("{}", instr)),
                        self.op_string.clone(),
                        ea_dif,
                    ),
                );
            }
        }

        instr_vec
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Disassembler {
    pub fn fetch(&mut self, mem: &[u8]) -> u8 {
        let ea = ((self.cs << 4) + self.ip) & 0xFFFFF;
        self.ip += 1;
        let val = mem[ea];

        self.op_string.push_str(&format!("{:02X}", val));

        val
    }

    pub fn read_imm_addres(&mut self, instr: &mut Instruction, mem: &[u8]) {
        instr.offset = to_u16(self.fetch(mem), self.fetch(mem));
    }

    pub fn decode_mem(&mut self, mem: &[u8], operand: u8, pos: u8, mode: AddrMode) -> OperandType {
        assert!(pos < 8);
        let rm = (operand >> pos) & 0x07;

        match mode {
            AddrMode::Mode0 => match rm {
                0b000 => OperandType::Memory(Operand::BXSI),
                0b001 => OperandType::Memory(Operand::BXDI),
                0b010 => OperandType::Memory(Operand::BPSI),
                0b011 => OperandType::Memory(Operand::BPDI),
                0b100 => OperandType::Memory(Operand::SI),
                0b101 => OperandType::Memory(Operand::DI),
                0b110 => {
                    let disp_low = self.fetch(mem);
                    let disp_high = self.fetch(mem);

                    // TODO Esto no se si estara mal
                    OperandType::Memory(Operand::Disp(to_u16(disp_low, disp_high)))
                }
                0b111 => OperandType::Memory(Operand::BX),
                _ => unreachable!("Aqui no deberia entrar nunca"),
            },
            AddrMode::Mode1 | AddrMode::Mode2 => {
                let disp = match mode {
                    AddrMode::Mode1 => {
                        let readed = self.fetch(mem);
                        sign_extend(readed)
                    }
                    AddrMode::Mode2 => {
                        let disp_low = self.fetch(mem);
                        let disp_high = self.fetch(mem);
                        to_u16(disp_low, disp_high)
                    }
                    _ => unreachable!(),
                };

                match rm {
                    0b000 => OperandType::Memory(Operand::DispBXSI(disp)),
                    0b001 => OperandType::Memory(Operand::DispBXDI(disp)),
                    0b010 => OperandType::Memory(Operand::DispBPSI(disp)),
                    0b011 => OperandType::Memory(Operand::DispBPDI(disp)),
                    0b100 => OperandType::Memory(Operand::DispSI(disp)),
                    0b101 => OperandType::Memory(Operand::DispDI(disp)),
                    0b110 => OperandType::Memory(Operand::DispBP(disp)),
                    0b111 => OperandType::Memory(Operand::DispBX(disp)),
                    _ => unreachable!("Aqui no deberia entrar nunca"),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn decode_rm(
        &mut self,
        instr: &mut Instruction,
        mem: &[u8],
        operand: u8,
        rm_pos: u8,
    ) -> OperandType {
        match instr.addr_mode {
            AddrMode::Mode0 | AddrMode::Mode1 | AddrMode::Mode2 => {
                self.decode_mem(mem, operand, rm_pos, instr.addr_mode)
            }
            AddrMode::Mode3 => decode_reg(operand, rm_pos, instr.data_length),
            _ => unreachable!("{}", instr.opcode),
        }
    }

    pub fn decode_mod_reg_rm(&mut self, instr: &mut Instruction, mem: &[u8], operand: u8) {
        instr.addr_mode = decode_mod(operand);

        match instr.direction {
            Direction::ToReg => {
                instr.operand1 = decode_reg(operand, 3, instr.data_length);
                instr.operand2 = self.decode_rm(instr, mem, operand, 0);
            }
            Direction::FromReg => {
                instr.operand1 = self.decode_rm(instr, mem, operand, 0);
                instr.operand2 = decode_reg(operand, 3, instr.data_length);
            }
            _ => unreachable!("{}", instr.opcode),
        }
    }

    pub fn decode_mod_n_rm(&mut self, instr: &mut Instruction, mem: &[u8], operand: u8) {
        instr.addr_mode = decode_mod(operand);
        instr.operand1 = self.decode_rm(instr, mem, operand, 0)
    }

    pub fn read_imm(&mut self, mem: &[u8], instr: &mut Instruction) -> u16 {
        match instr.data_length {
            Length::Byte => self.fetch(mem) as u16,
            Length::Word => to_u16(self.fetch(mem), self.fetch(mem)),
            _ => unreachable!("{}", instr.opcode),
        }
    }

    pub fn read_length(&self, instr: &mut Instruction, mem: &[u8], segment: u16, offset: u16) -> u16 {
        let address = ((segment as usize) << 4) + offset as usize;
        match instr.data_length {
            Length::Byte => mem[address] as u16,
            Length::Word => to_u16(mem[address], mem[address.wrapping_add(1)]),
            _ => unreachable!("{}", instr.opcode)
        }
    }

    pub fn get_val(&self, cpu: &CPU, instr: &mut Instruction, mem: &[u8]) -> u16 {
        match instr.operand1 {
            OperandType::Register(operand) => cpu.get_reg(operand),
            OperandType::SegmentRegister(operand) => cpu.get_segment(operand),
            OperandType::Immediate(imm) => imm,
            OperandType::Memory(_operand) => self.read_length(
                instr,
                mem,
                cpu.get_segment(instr.segment),
                instr.offset,
            ),
            _ => unreachable!(),
        }
    }
}
