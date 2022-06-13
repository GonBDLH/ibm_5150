use std::io::{self, stdout};

use crossterm::{execute, style::Print, cursor::MoveTo, terminal::{Clear, ClearType}};

use crate::System;

pub fn display(sys: &System) {
    let ip = sys.cpu.ip as usize;
    let dir = ((sys.cpu.cs as usize) << 4) + ip as usize;
    execute!(
        stdout(),
        MoveTo(0,0),
        // Esto es un shitpost total
        Print(format!(
            "*===================================*==================================================================================*\
             |                                   |                                                                                  |\
             |               CPU                 |                            MEMORIA                                               |\
             |                                   |                                                                                  |\
             |   Registros         Flags         |   Segmentos           Direccion  Valor                                           |\
             |                                   |                                                                                  |\
             |       H L           O {}           |   CS   {:04X}           ->  {:05X}     {:02X}                                           |\
             |   AX {:02X}{:02X}           D {}           |   DS   {:04X}               {:05X}     {:02X}                                           |\
             |       H L           I {}           |   ES   {:04X}               {:05X}     {:02X}                                           |\
             |   BX {:02X}{:02X}           T {}           |   SS   {:04X}               {:05X}     {:02X}                                           |\
             |       H L           S {}           |                           {:05X}     {:02X}                                           |\
             |   CX {:02X}{:02X}           Z {}           |                           {:05X}     {:02X}                                           |\
             |       H L           A {}           |                           {:05X}     {:02X}                                           |\
             |   DX {:02X}{:02X}           P {}           |                           {:05X}     {:02X}                                           |\
             |                     C {}           |                           {:05X}     {:02X}                                           |\
             |                                   |                           {:05X}     {:02X}                                           |\
             |   SI {:04X}                         |                           {:05X}     {:02X}                                           |\
             |   BP {:04X}                         |                           {:05X}     {:02X}                                           |\
             |   DI {:04X}                         |                           {:05X}     {:02X}                                           |\
             |   SP {:04X}                         |                           {:05X}     {:02X}                                           |\
             |                                   |                           {:05X}     {:02X}                                           |\
             |   IP {:04X}                         |                           {:05X}     {:02X}                                           |\
             |                                   |                                                                                  |\
             |                                   |                                                                                  |\
             |                                   |                                                                                  |\
             |                                   |                                                                                  |\
             |                                   |                                                                                  |\
             *===================================*==================================================================================*\
             |   >>                                                                                                                 |\
             *======================================================================================================================*",
                                                sys.cpu.flags.o as i32, sys.cpu.cs, dir, sys.bus.memory[dir],
             sys.cpu.ax.high, sys.cpu.ax.low,   sys.cpu.flags.d as i32, sys.cpu.ds, (dir + 1) % 0x100000, sys.bus.memory[(dir + 1) % 0x100000],
                                                sys.cpu.flags.i as i32, sys.cpu.es, (dir + 2) % 0x100000, sys.bus.memory[(dir + 2) % 0x100000],
             sys.cpu.bx.high, sys.cpu.bx.low,   sys.cpu.flags.t as i32, sys.cpu.ss, (dir + 3) % 0x100000, sys.bus.memory[(dir + 3) % 0x100000],
                                                sys.cpu.flags.s as i32,             (dir + 4) % 0x100000, sys.bus.memory[(dir + 4) % 0x100000],
             sys.cpu.cx.high, sys.cpu.cx.low,   sys.cpu.flags.z as i32,             (dir + 5) % 0x100000, sys.bus.memory[(dir + 5) % 0x100000],
                                                sys.cpu.flags.a as i32,             (dir + 6) % 0x100000, sys.bus.memory[(dir + 6) % 0x100000],
             sys.cpu.dx.high, sys.cpu.dx.low,   sys.cpu.flags.p as i32,             (dir + 7) % 0x100000, sys.bus.memory[(dir + 7) % 0x100000],
                                                sys.cpu.flags.c as i32,             (dir + 8) % 0x100000, sys.bus.memory[(dir + 8) % 0x100000],
                                                                                    (dir + 9) % 0x100000, sys.bus.memory[(dir + 9) % 0x100000],
             sys.cpu.si,                                                            (dir + 10) % 0x100000, sys.bus.memory[(dir + 10) % 0x100000],
             sys.cpu.di,                                                            (dir + 11) % 0x100000, sys.bus.memory[(dir + 11) % 0x100000],
             sys.cpu.bp,                                                            (dir + 12) % 0x100000, sys.bus.memory[(dir + 12) % 0x100000],
             sys.cpu.sp,                                                            (dir + 13) % 0x100000, sys.bus.memory[(dir + 13) % 0x100000],
                                                                                    (dir + 14) % 0x100000, sys.bus.memory[(dir + 14) % 0x100000],
             sys.cpu.ip,                                                            (dir + 15) % 0x100000, sys.bus.memory[(dir + 15) % 0x100000],
             
        )),
        MoveTo(7, 28)
    ).unwrap();
}

pub fn get_command(sys: &mut System) {
    let mut command = String::new();

    io::stdin().read_line(&mut command).expect("Failed");

    match command.trim_end() {
        "step" | "s" => {sys.cpu.instr.cycles = 0; sys.cpu.fetch_decode_execute(&mut sys.bus)},
        "quit" | "q" => {execute!(stdout(), Clear(ClearType::All), MoveTo(0,0)).unwrap(); sys.running = false},
        "run" | "r" => {sys.cpu.instr.cycles = 0; sys.clock()},
        "load_bios" | "lb" => {sys.load_bios()},
        _ => {}
    }
}