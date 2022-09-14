use crate::hardware::bus::Bus;

use super::CPU;
use super::instr_utils::Instruction;

#[allow(dead_code)]
pub fn dissasemble_one(bus_: &Bus, cpu_: &CPU) -> Instruction {
    let mut cpu = CPU::new();
    cpu.ip = cpu_.ip;
    cpu.cs = cpu_.cs;

    let mut bus = bus_.clone();

    // let op = fetch_disassembly(&mut bus, &mut ip);
    // decode_dissasembly(&mut cpu, &mut bus, op, &mut ip);
    // cpu.instr
    let op = cpu.fetch(&mut bus);
    cpu.decode(&mut bus, op);
    cpu.instr
}
