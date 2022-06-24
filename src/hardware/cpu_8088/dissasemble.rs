use crate::hardware::bus::Bus;

use super::CPU;

use std::io::Write;
use std::fs::File;

#[allow(dead_code)]
pub fn dissasemble_all(bus_: &Bus) {
    let mut cpu = CPU::new();
    let mut bus = bus_.clone();
    let mut ip = 0xFFFF0;

    let mut dis = File::create("logs/dissasembled.txt").unwrap();


    bus.memory[0xFFFF0] = 0xEA;
    bus.memory[0xFFFF1] = 0x00;
    bus.memory[0xFFFF2] = 0x00;
    bus.memory[0xFFFF3] = 0x00;
    bus.memory[0xFFFF4] = 0x00;

    cpu.ax.set_x(0xC56D);
    cpu.bx.set_x(0x1234);

    bus.memory[0x00000] = 0xF7;
    bus.memory[0x00001] = 0xFB;
    bus.memory[0x00002] = 0xD4;
    bus.memory[0x00003] = 0x0A;

    // TODO Esto es una mierda (En plan, todo el archivo XD)

    while ip < 0x100000 {
        let dir = ip;
        let op = fetch_disassembly(&mut bus, &mut ip);
        decode_dissasembly(&mut cpu, &mut bus, op, &mut ip);
        writeln!(&mut dis, "{:05X}: {} {} {}", dir, cpu.instr.opcode, cpu.instr.operand1, cpu.instr.operand2).unwrap();
    }
}


use super::instr_utils::*;
use super::cpu_utils::*;

fn fetch_disassembly(bus: &mut Bus, ip: &mut usize) -> u8 {
    let val = bus.memory[*ip % 0x100000];
    *ip += 1;
    val
}

fn decode_dissasembly(cpu: &mut CPU, bus: &mut Bus, op: u8, ip: &mut usize) {
    match op {
        0x88..=0x8B => {
            cpu.instr.opcode = Opcode::MOV;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 2,
                (OperandType::Memory(_), OperandType::Register(_)) => 13 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 12 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            }
        },
        0xC6 | 0xC7 => {
            cpu.instr.opcode = Opcode::MOV;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand2 = OperandType::Immediate;

            let operand = fetch_disassembly(bus, ip);
            decode_mod_n_rm(cpu, bus, operand);
            read_imm(cpu, bus);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Memory(_), OperandType::Immediate) => 14 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Immediate) => 4,
                _ => unreachable!(),
            }
        },
        0xB0..=0xBF => {
            cpu.instr.opcode = Opcode::MOV;
            cpu.instr.data_length = Length::new(op, 3);
            cpu.instr.operand1 = decode_reg(op, 0, cpu.instr.data_length);
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);
            cpu.instr.cycles += 4;
        },
        0xA0..=0xA3 => {
            cpu.instr.opcode = Opcode::MOV;
            cpu.instr.direction = if 0x02 & op == 0 {Direction::ToReg} else {Direction::FromReg};
            cpu.instr.data_length = Length::new(op, 0);
            if cpu.instr.direction == Direction::ToReg {
                if cpu.instr.data_length == Length::Byte {
                    cpu.instr.operand1 = OperandType::Register(Operand::AL);
                } else {
                    cpu.instr.operand1 = OperandType::Register(Operand::AX);
                }
                cpu.instr.operand2 = OperandType::Memory(Operand::Disp);
            } else {
                cpu.instr.operand1 = OperandType::Memory(Operand::Disp);
                if cpu.instr.data_length == Length::Byte {
                    cpu.instr.operand2 = OperandType::Register(Operand::AL);
                } else {
                    cpu.instr.operand2 = OperandType::Register(Operand::AX);
                }
            }
            read_imm_addres(cpu, bus);
            cpu.instr.cycles += 14;
        },
        0x8E | 0x8C => {
            cpu.instr.opcode = Opcode::MOV;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::Word;

            let operand = fetch_disassembly(bus, ip);
            cpu.instr.addr_mode = decode_mod(operand);

            match cpu.instr.direction {
                Direction::ToReg => {
                    cpu.instr.operand1 = decode_segment(operand, 3);
                    cpu.instr.operand2 = decode_rm(cpu, bus, operand, 0);
                },
                Direction::FromReg => {
                    cpu.instr.operand1 = decode_rm(cpu, bus, operand, 0);
                    cpu.instr.operand2 = decode_segment(operand, 3);
                },
                _ => unreachable!(),
            }

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::SegmentRegister(_), OperandType::Register(_)) => 2,
                (OperandType::Register(_), OperandType::Memory(_)) => 12 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::SegmentRegister(_)) => 2,
                (OperandType::Memory(_), OperandType::Register(_)) => 13 + cpu.instr.ea_cycles,
                (OperandType::Memory(_), OperandType::SegmentRegister(_)) => 13 + cpu.instr.ea_cycles,
                (OperandType::SegmentRegister(_), OperandType::Memory(_)) => 12 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            }
        },
        // Prefijo _MISC r/m16
        0xFF => {
            let operand = fetch_disassembly(bus, ip);

            match operand & 0b00111000 {
                0x0 => {
                    cpu.instr.opcode = Opcode::INC;
                    cpu.instr.data_length = Length::Word;
                    decode_mod_n_rm(cpu, bus, operand);

                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => 3,
                        OperandType::Memory(_) => 23 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    }
                },
                0x8 => {
                    cpu.instr.opcode = Opcode::DEC;
                    cpu.instr.data_length = Length::Word;
                    decode_mod_n_rm(cpu, bus, operand);

                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => 3,
                        OperandType::Memory(_) => 23 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    }
                },
                0x10 => {
                    cpu.instr.opcode = Opcode::CALL;
                    cpu.instr.data_length = Length::Word;
                    decode_mod_n_rm(cpu, bus, operand);

                    cpu.instr.jump_type = JumpType::IndWithinSegment;

                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => 24,
                        OperandType::Memory(_) => 29 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    }
                },
                0x18 => {
                    cpu.instr.opcode = Opcode::CALL;
                    cpu.instr.data_length = Length::Word;
                    decode_mod_n_rm(cpu, bus, operand);

                    cpu.instr.jump_type = JumpType::IndIntersegment;

                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Memory(_) => 53 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    }
                },
                0x20 => {
                    cpu.instr.opcode = Opcode::JMP;
                    cpu.instr.data_length = Length::Word;
                    decode_mod_n_rm(cpu, bus, operand);

                    cpu.instr.jump_type = JumpType::IndWithinSegment;

                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => 11,
                        OperandType::Memory(_) => 18 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    }
                },
                0x28 => {
                    cpu.instr.opcode = Opcode::JMP;
                    cpu.instr.data_length = Length::Word;
                    decode_mod_n_rm(cpu, bus, operand);

                    cpu.instr.jump_type = JumpType::IndIntersegment;

                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Memory(_) => 24 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    }
                },
                0x30 => {
                    cpu.instr.opcode = Opcode::PUSH;
                    cpu.instr.data_length = Length::Word;
                    decode_mod_n_rm(cpu, bus, operand);
                    
                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => 15,
                        OperandType::Memory(_) => 24 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    }
                },
                _ => unreachable!(),    
            }
        },
        0x50..=0x57 => {
            cpu.instr.opcode = Opcode::PUSH;
            cpu.instr.data_length = Length::Word;
            cpu.instr.operand1 = decode_reg(op, 0, cpu.instr.data_length);
            cpu.instr.cycles += 15;
        },
        0x06 | 0x16 | 0x0E | 0x1E => {
            cpu.instr.opcode = Opcode::PUSH;
            cpu.instr.data_length = Length::Word;
            cpu.instr.operand1 = decode_segment(op, 3);
            cpu.instr.cycles += 14;
        },
        0x8F => {
            cpu.instr.opcode = Opcode::POP;
            let operand = fetch_disassembly(bus, ip);
            cpu.instr.addr_mode = decode_mod(operand);
            cpu.instr.data_length = Length::Word;
            cpu.instr.operand1 = decode_rm(cpu, bus, operand, 0);

            cpu.instr.cycles += match cpu.instr.operand1 {
                OperandType::Register(_) => 12,
                OperandType::Memory(_) => 25 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            }
        },
        0x58..=0x5F => {
            cpu.instr.opcode = Opcode::POP;
            cpu.instr.data_length = Length::Word;
            cpu.instr.operand1 = decode_reg(op, 0, cpu.instr.data_length);
            cpu.instr.cycles += 12;
        },
        0x07 | 0x17 | 0x0F | 0x1F => {
            cpu.instr.opcode = Opcode::POP;
            cpu.instr.data_length = Length::Word;
            cpu.instr.operand1 = decode_segment(op, 3);
            cpu.instr.cycles += 12;
        },
        0x86 | 0x87 => {
            cpu.instr.opcode = Opcode::XCHG;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.direction = Direction::ToReg;
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 4,
                (OperandType::Register(_), OperandType::Memory(_)) => 25 + cpu.instr.ea_cycles,
                (OperandType::Memory(_), OperandType::Register(_)) => 25 + cpu.instr.ea_cycles, // Creo que no hace falta, pero por si acaso
                _ => unreachable!(),
            }
        },
        0x90..=0x97 => {
            cpu.instr.opcode = Opcode::XCHG;
            cpu.instr.data_length = Length::Word;
            cpu.instr.direction = Direction::ToReg;
            cpu.instr.operand1 = OperandType::Register(Operand::AX);
            cpu.instr.operand2 = decode_reg(op, 0, cpu.instr.data_length);
            cpu.instr.cycles += 3;
        },
        // TODO IN
        0xE4 | 0xE5 => {
            cpu.instr.opcode = Opcode::IN;
            cpu.instr.cycles += 1;
        },
        0xEC | 0xED => {
            cpu.instr.opcode = Opcode::IN;
            cpu.instr.cycles += 1;
        },
        // TODO OUT
        0xE6 | 0xE7 => {
            cpu.instr.opcode = Opcode::OUT;
            cpu.instr.cycles += 1;
        },
        0xEE | 0xEF => {
            cpu.instr.opcode = Opcode::OUT;
            cpu.instr.cycles += 1;
        },
        0xD7 => {
            cpu.instr.opcode = Opcode::XLAT;
            cpu.instr.operand1 = OperandType::Register(Operand::AL);

            cpu.instr.cycles += 11;
        },
        0x8D => {
            cpu.instr.opcode = Opcode::LEA;
            cpu.instr.direction = Direction::ToReg;
            cpu.instr.data_length = Length::Word;
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);
            cpu.instr.cycles += 2 + cpu.instr.ea_cycles;
        },
        0xC5 => {
            cpu.instr.opcode = Opcode::LDS;
            cpu.instr.direction = Direction::ToReg;
            cpu.instr.data_length = Length::Word;
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);
            cpu.instr.cycles += 24 + cpu.instr.ea_cycles;
        },
        0xC4 => {
            cpu.instr.opcode = Opcode::LES;
            cpu.instr.direction = Direction::ToReg;
            cpu.instr.data_length = Length::Word;
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);
            cpu.instr.cycles += 24 + cpu.instr.ea_cycles;
        },
        0x9F => {
            cpu.instr.opcode = Opcode::LAHF;
            cpu.instr.cycles += 4;
        },
        0x9E => {
            cpu.instr.opcode = Opcode::SAHF;
            cpu.instr.cycles += 4;
        },
        0x9C => {
            cpu.instr.opcode = Opcode::PUSHF;
            cpu.instr.cycles += 14;
        },
        0x9D => {
            cpu.instr.opcode = Opcode::POPF;
            cpu.instr.cycles += 12;
        },

        0x00..=0x03 => {
            cpu.instr.opcode = Opcode::ADD;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 24 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        // Prefijos _ALU1
        0x80..=0x83 => {
            let operand = fetch_disassembly(bus, ip);
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.addr_mode = decode_mod(operand);
            cpu.instr.operand1 = decode_rm(cpu, bus, operand, 0);
            cpu.instr.operand2 = OperandType::Immediate;

            match op & 0b00000011 {
                0b00..=0b10 => read_imm(cpu, bus),
                0b11 => cpu.instr.imm = sign_extend(cpu.fetch(bus)),
                _ => unreachable!(),
            }

            cpu.instr.cycles += match cpu.instr.operand1 {
                OperandType::Register(_) => 4,
                OperandType::Memory(_) => 23 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };

            match operand & 0b00111000 {
                0x00 => cpu.instr.opcode = Opcode::ADD,
                0x08 => cpu.instr.opcode = Opcode::OR,
                0x10 => cpu.instr.opcode = Opcode::ADC,
                0x18 => cpu.instr.opcode = Opcode::SBB,
                0x20 => cpu.instr.opcode = Opcode::AND,
                0x28 => cpu.instr.opcode = Opcode::SUB,
                0x30 => cpu.instr.opcode = Opcode::XOR,
                0x38 => cpu.instr.opcode = Opcode::CMP,
                _ => unreachable!(),
            }
        },
        0x04 | 0x05 => {
            cpu.instr.opcode = Opcode::ADD;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },
        0x10..=0x13 => {
            cpu.instr.opcode = Opcode::ADC;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 24 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        0x14 | 0x15 => {
            cpu.instr.opcode = Opcode::ADC;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },
        // Prefijo _MISC r/m8
        0xFE => {
            let operand = fetch_disassembly(bus, ip);

            cpu.instr.data_length = Length::Byte;
            decode_mod_n_rm(cpu, bus, operand);

            cpu.instr.cycles += match cpu.instr.operand1 {
                OperandType::Register(_) => 3,
                OperandType::Memory(_) => 23 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };

            match operand & 0b00111000 {
                0x00 => cpu.instr.opcode = Opcode::INC,
                0x08 => cpu.instr.opcode = Opcode::DEC,
                _ => unreachable!(),
            }
        },
        0x40..=0x47 => {
            cpu.instr.opcode = Opcode::INC;
            cpu.instr.data_length = Length::Word;
            cpu.instr.operand1 = decode_reg(op, 0, cpu.instr.data_length);
            cpu.instr.cycles += 3;
        },
        0x37 => {
            cpu.instr.opcode = Opcode::AAA;
            cpu.instr.cycles += 8;
        },
        0x27 => {
            cpu.instr.opcode = Opcode::DAA;
            cpu.instr.cycles += 4;
        },
        0x28..=0x2B => {
            cpu.instr.opcode = Opcode::SUB;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 24 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        0x2C | 0x2D => {
            cpu.instr.opcode = Opcode::SUB;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },
        0x18..=0x1B => {
            cpu.instr.opcode = Opcode::SBB;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 24 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        0x1C | 0x1D => {
            cpu.instr.opcode = Opcode::SBB;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },
        0x48..=0x4F => {
            cpu.instr.opcode = Opcode::DEC;
            cpu.instr.data_length = Length::Word;
            cpu.instr.operand1 = decode_reg(op, 0, cpu.instr.data_length);
            cpu.instr.cycles += 3;
        },
        // _ALU 2
        0xF6 | 0xF7 => {
            let operand = fetch_disassembly(bus, ip);
            cpu.instr.data_length = Length::new(op, 0);
            
            match operand & 0b00111000 {
                0x00 | 0x08 => {
                    cpu.instr.opcode = Opcode::TEST;
                    decode_mod_n_rm(cpu, bus, operand);
                    cpu.instr.operand2 = OperandType::Immediate;
                    read_imm(cpu, bus);
    
                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => 5,
                        OperandType::Memory(_) => 11 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    };
                }
                0x10 => {
                    cpu.instr.opcode = Opcode::NOT;
                    decode_mod_n_rm(cpu, bus, operand);

                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => 3,
                        OperandType::Memory(_) => 24 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    };
                },
                0x18 => {
                    cpu.instr.opcode = Opcode::NEG;
                    decode_mod_n_rm(cpu, bus, operand);

                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => 3,
                        OperandType::Memory(_) => 24 + cpu.instr.ea_cycles,
                        _ => unreachable!(),
                    };
                },
                0x20 => {
                    cpu.instr.opcode = Opcode::MUL;
                    decode_mod_n_rm(cpu, bus, operand);

                    // TODO no se si esto esta bien del todo
                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => if cpu.instr.data_length == Length::Byte {77} else {133},
                        OperandType::Memory(_) => if cpu.instr.data_length == Length::Byte {83 + cpu.instr.ea_cycles} else {139 + cpu.instr.ea_cycles},
                        _ => unreachable!(),
                    };
                },
                0x28 => {
                    cpu.instr.opcode = Opcode::IMUL;
                    decode_mod_n_rm(cpu, bus, operand);

                    // TODO no se si esto esta bien del todo
                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => if cpu.instr.data_length == Length::Byte {98} else {154},
                        OperandType::Memory(_) => if cpu.instr.data_length == Length::Byte {104 + cpu.instr.ea_cycles} else {160 + cpu.instr.ea_cycles},
                        _ => unreachable!(),
                    };
                },
                0x30 => {
                    cpu.instr.opcode = Opcode::DIV;
                    decode_mod_n_rm(cpu, bus, operand);

                    // TODO no se si esto esta bien del todo
                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => if cpu.instr.data_length == Length::Byte {90} else {162},
                        OperandType::Memory(_) => if cpu.instr.data_length == Length::Byte {96 + cpu.instr.ea_cycles} else {168 + cpu.instr.ea_cycles},
                        _ => unreachable!(),
                    };
                }
                0x38 => {
                    cpu.instr.opcode = Opcode::IDIV;
                    decode_mod_n_rm(cpu, bus, operand);

                    // TODO no se si esto esta bien del todo
                    cpu.instr.cycles += match cpu.instr.operand1 {
                        OperandType::Register(_) => if cpu.instr.data_length == Length::Byte {112} else {184},
                        OperandType::Memory(_) => if cpu.instr.data_length == Length::Byte {118 + cpu.instr.ea_cycles} else {190 + cpu.instr.ea_cycles},
                        _ => unreachable!(),
                    };
                }
                _ => unreachable!(),
            }
        },
        0x38..=0x3B => {
            cpu.instr.opcode = Opcode::CMP;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 13 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        0x3C | 0x3D => {
            cpu.instr.opcode = Opcode::CMP;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },
        0x3F => {
            cpu.instr.opcode = Opcode::AAS;
            cpu.instr.cycles += 8;
        },
        0x2F => {
            cpu.instr.opcode = Opcode::DAS;
            cpu.instr.cycles += 4;
        },
        0xD4 => {
            cpu.instr.opcode = Opcode::AAM;
            cpu.instr.operand1 = OperandType::Immediate;
            cpu.instr.data_length = Length::Byte;
            read_imm(cpu, bus);
            cpu.instr.cycles += 83;
        },
        0xD5 => {
            cpu.instr.opcode = Opcode::AAD;
            cpu.instr.operand1 = OperandType::Immediate;
            cpu.instr.data_length = Length::Byte;
            read_imm(cpu, bus);
            cpu.instr.cycles += 60;
        },
        0x98 => {
            cpu.instr.opcode = Opcode::CBW;
            cpu.instr.cycles += 2;
        },
        0x99 => {
            cpu.instr.opcode = Opcode::CWD;
            cpu.instr.cycles += 5;
        },

        // _ROT 1
        0xD0..=0xD3 => {
            let operand = fetch_disassembly(bus, ip);

            cpu.instr.data_length = Length::new(op, 0);
            decode_mod_n_rm(cpu, bus, operand);
            cpu.instr.operand2 = if operand & 0x02 == 1 {
                OperandType::Register(Operand::CL)
            } else {
                cpu.instr.imm = 1;
                OperandType::Immediate
            };

            match operand & 0b00111000 {
                0x00 => cpu.instr.opcode = Opcode::ROL,
                0x08 => cpu.instr.opcode = Opcode::ROR,
                0x10 => cpu.instr.opcode = Opcode::RCL,
                0x18 => cpu.instr.opcode = Opcode::RCR,
                0x20 => cpu.instr.opcode = Opcode::SALSHL,
                0x28 => cpu.instr.opcode = Opcode::SHR,
                0x30 => cpu.instr.opcode = Opcode::SALSHL,
                0x38 => cpu.instr.opcode = Opcode::SAR,
                _ => unreachable!(),
            }
            
            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Immediate) => 2,
                (OperandType::Memory(_), OperandType::Immediate) => 23 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Register(_)) => 8,
                (OperandType::Memory(_), OperandType::Register(_)) => 28 + cpu.instr.ea_cycles + 4 * (cpu.cx.low as u64),
                _ => unreachable!(),
            };

        },
        0x20..=0x23 => {
            cpu.instr.opcode = Opcode::AND;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 24 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        0x24 | 0x25 => {
            cpu.instr.opcode = Opcode::AND;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },
        0x08..=0x0B => {
            cpu.instr.opcode = Opcode::OR;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 24 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        0x0C | 0x0D => {
            cpu.instr.opcode = Opcode::OR;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },
        0x30..=0x33 => {
            cpu.instr.opcode = Opcode::XOR;
            cpu.instr.direction = Direction::new(op);
            cpu.instr.data_length = Length::new(op, 0);
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 24 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        0x34 | 0x35 => {
            cpu.instr.opcode = Opcode::XOR;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },
        0x84 | 0x85 => {
            cpu.instr.opcode = Opcode::TEST;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.direction = Direction::FromReg;
            let operand = fetch_disassembly(bus, ip);
            decode_mod_reg_rm(cpu, bus, operand);

            cpu.instr.cycles += match (cpu.instr.operand1, cpu.instr.operand2) {
                (OperandType::Register(_), OperandType::Register(_)) => 3,
                (OperandType::Memory(_), OperandType::Register(_)) => 13 + cpu.instr.ea_cycles,
                (OperandType::Register(_), OperandType::Memory(_)) => 13 + cpu.instr.ea_cycles,
                _ => unreachable!(),
            };
        },
        0xA8 | 0xA9 => {
            cpu.instr.opcode = Opcode::AND;
            cpu.instr.data_length = Length::new(op, 0);
            cpu.instr.operand1 = match cpu.instr.data_length {
                Length::Byte => OperandType::Register(Operand::AL),
                Length::Word => OperandType::Register(Operand::AX),
                _ => unreachable!(),
            };
            cpu.instr.operand2 = OperandType::Immediate;
            read_imm(cpu, bus);

            cpu.instr.cycles += 4;
        },

        0x26 | 0x2E | 0x36 | 0x3E => {
            cpu.instr.segment = if let OperandType::SegmentRegister(operand) = decode_segment(op, 4) {
                operand
            } else {
                panic!()
            };
            
            cpu.instr.cycles += 2;

            let new_op = fetch_disassembly(bus, ip);
            cpu.decode(bus, new_op);
        },

        0xF2 | 0xF3 => {
            cpu.instr.repetition_prefix = if op & 0x01 == 1 {
                RepetitionPrefix::REPEZ
            } else {
                RepetitionPrefix::REPNEZ
            };

            let new_op = fetch_disassembly(bus, ip);
            cpu.decode(bus, new_op);
        },

        0xEA => {
            cpu.instr.opcode = Opcode::JMP;
            let offset_low = fetch_disassembly(bus, ip);
            let offset_high = fetch_disassembly(bus, ip);
            let seg_low = fetch_disassembly(bus, ip);
            let seg_high = fetch_disassembly(bus, ip);
            let offset = to_u16(offset_low, offset_high);
            let seg = to_u16(seg_low, seg_high);
            cpu.instr.jump_type = JumpType::DirIntersegment(offset, seg);

            cpu.instr.cycles += 15;
        },
        0xA4 => {
            cpu.instr.opcode = Opcode::MOVSB;
            cpu.instr.data_length = Length::Byte;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {18} else {9}
        },
        0xA5 => {
            cpu.instr.opcode = Opcode::MOVSW;
            cpu.instr.data_length = Length::Word;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {26} else {9}
        },
        0xA6 => {
            cpu.instr.opcode = Opcode::CMPSB;
            cpu.instr.data_length = Length::Byte;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {22} else {9}
        },
        0xA7 => {
            cpu.instr.opcode = Opcode::CMPSW;
            cpu.instr.data_length = Length::Word;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {30} else {9}
        },
        0xAE => {
            cpu.instr.opcode = Opcode::SCASB;
            cpu.instr.data_length = Length::Byte;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {15} else {9}
        },
        0xAF => {
            cpu.instr.opcode = Opcode::SCASW;
            cpu.instr.data_length = Length::Word;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {19} else {9}
        },
        0xAC => {
            cpu.instr.opcode = Opcode::LODSB;
            cpu.instr.data_length = Length::Byte;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {12} else {9}
        },
        0xAD => {
            cpu.instr.opcode = Opcode::LODSW;
            cpu.instr.data_length = Length::Word;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {16} else {9}
        },
        0xAA => {
            cpu.instr.opcode = Opcode::STOSB;
            cpu.instr.data_length = Length::Byte;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {11} else {9}
        },
        0xAB => {
            cpu.instr.opcode = Opcode::STOSW;
            cpu.instr.data_length = Length::Word;

            cpu.instr.cycles += if cpu.instr.repetition_prefix != RepetitionPrefix::None {15} else {9}
        },

        0xE8 => {
            cpu.instr.opcode = Opcode::CALL;
            let val_low = fetch_disassembly(bus, ip);
            let val_high = fetch_disassembly(bus, ip);
            let val = to_u16(val_low, val_high);

            cpu.instr.jump_type = JumpType::DirWithinSegment(val);

            cpu.instr.cycles += 23;
        },
        0x9A => {
            cpu.instr.opcode = Opcode::CALL;

            let offset_low = fetch_disassembly(bus, ip);
            let offset_high = fetch_disassembly(bus, ip);
            let seg_low = fetch_disassembly(bus, ip);
            let seg_high = fetch_disassembly(bus, ip);
            let offset = to_u16(offset_low, offset_high);
            let seg = to_u16(seg_low, seg_high);
            cpu.instr.jump_type = JumpType::DirIntersegment(offset, seg);

            cpu.instr.cycles += 36;
        },
        0xE9 => {
            cpu.instr.opcode = Opcode::JMP;
            let val_low = fetch_disassembly(bus, ip);
            let val_high = fetch_disassembly(bus, ip);
            let val = to_u16(val_low, val_high);

            cpu.instr.jump_type = JumpType::DirWithinSegment(val);

            cpu.instr.cycles += 15;
        },
        0xEB => {
            cpu.instr.opcode = Opcode::JMP;
            let val = fetch_disassembly(bus, ip);

            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);

            cpu.instr.cycles += 15;
        },
        0xC2 => {
            cpu.instr.opcode = Opcode::RET;
            let val = to_u16(cpu.fetch(bus), cpu.fetch(bus));
            cpu.instr.ret_type = RetType::NearAdd(val);
            cpu.instr.cycles += 24;
        },
        0xC3 => {
            cpu.instr.opcode = Opcode::RET;
            cpu.instr.ret_type = RetType::Near;
            cpu.instr.cycles += 20;
        },
        0xCA => {
            cpu.instr.opcode = Opcode::RET;
            let val = to_u16(cpu.fetch(bus), cpu.fetch(bus));
            cpu.instr.ret_type = RetType::FarAdd(val);
            cpu.instr.cycles += 34;
        },
        0xCB => {
            cpu.instr.opcode = Opcode::RET;
            cpu.instr.ret_type = RetType::Far;
            cpu.instr.cycles += 33;
        },
        0x74 => {
            cpu.instr.opcode = Opcode::JEJZ;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x7C => {
            cpu.instr.opcode = Opcode::JLJNGE;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x7E => {
            cpu.instr.opcode = Opcode::JLEJNG;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x72 => {
            cpu.instr.opcode = Opcode::JBJNAE;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x76 => {
            cpu.instr.opcode = Opcode::JBEJNA;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x7A => {
            cpu.instr.opcode = Opcode::JPJPE;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x70 => {
            cpu.instr.opcode = Opcode::JO;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x78 => {
            cpu.instr.opcode = Opcode::JS;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x75 => {
            cpu.instr.opcode = Opcode::JNEJNZ;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x7D => {
            cpu.instr.opcode = Opcode::JNLJGE;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        }
        0x7F => {
            cpu.instr.opcode = Opcode::JNLEJG;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x73 => {
            cpu.instr.opcode = Opcode::JNBJAE;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x77 => {
            cpu.instr.opcode = Opcode::JNBEJA;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x7B => {
            cpu.instr.opcode = Opcode::JNPJPO;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x71 => {
            cpu.instr.opcode = Opcode::JNO;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0x79 => {
            cpu.instr.opcode = Opcode::JNS;
            let val = fetch_disassembly(bus, ip);
            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0xE2 => {
            cpu.instr.opcode = Opcode::LOOP;
            let val = fetch_disassembly(bus, ip);

            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0xE1 => {
            cpu.instr.opcode = Opcode::LOOPZE;
            let val = fetch_disassembly(bus, ip);

            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0xE0 => {
            cpu.instr.opcode = Opcode::LOOPNZNE;
            let val = fetch_disassembly(bus, ip);

            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },
        0xE3 => {
            cpu.instr.opcode = Opcode::JCXZ;
            let val = fetch_disassembly(bus, ip);

            cpu.instr.jump_type = JumpType::DirWithinSegmentShort(val);
        },

        0xCC => {
            cpu.instr.opcode = Opcode::INT;

            cpu.sw_int_type = 3;

            cpu.instr.cycles += 72;
        },
        0xCD => {
            cpu.instr.opcode = Opcode::INT;

            cpu.sw_int_type = fetch_disassembly(bus, ip);

            cpu.instr.cycles += 71;
        },
        0xCE => {
            cpu.instr.opcode = Opcode::INTO;

            cpu.sw_int_type = 4;
        },
        0xCF =>{
            cpu.instr.opcode = Opcode::IRET;

            cpu.instr.cycles += 44;
        }

        0xF8 => {
            cpu.instr.opcode = Opcode::CLC;
            cpu.instr.cycles += 2;
        },
        0xF5 => {
            cpu.instr.opcode = Opcode::CMC;
            cpu.instr.cycles += 2;
        },
        0xF9 => {
            cpu.instr.opcode = Opcode::STC;
            cpu.instr.cycles += 2;
        },
        0xFC => {
            cpu.instr.opcode = Opcode::CLD;
            cpu.instr.cycles += 2;
        },
        0xFD => {
            cpu.instr.opcode = Opcode::STD;
            cpu.instr.cycles += 2;
        },
        0xFA => {
            cpu.instr.opcode = Opcode::CLI;
            cpu.instr.cycles += 2;
        },
        0xFB => {
            cpu.instr.opcode = Opcode::STI;
            cpu.instr.cycles += 2;
        },

        _ => {
            cpu.instr.cycles += 3
        }
        // _ => unreachable!(),
    }
}