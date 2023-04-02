use std::thread;
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::Print,
    terminal::{Clear, ClearType},
};

use crate::System;

use crate::hardware::{
    bus::Bus,
    cpu_8088::{dissasemble::*, CPU},
};

pub fn display(sys: &System) {
    let ip = sys.cpu.ip as usize;
    let dir = ((sys.cpu.cs as usize) << 4) + ip as usize;
    queue!(
        stdout(),
        MoveTo(0,0),
        // Esto es un shitpost total. Ya no, pero este comentario es gracioso asi que se queda.
        Print(
            "*===================================*=================================================*================================*\
             |                                   |                                                 |                                |\
             |               CPU                 |                  MEMORIA                        |          Instruccion           |\
             |                                   |                                                 |                                |\
             |   Registros         Flags         |   Segmentos           Direccion  Valor          |   Opcode:                      |\
             |                                   |                                                 |   Operand1:                    |\
             |       H L           O             |   CS                  ->                        |   Operand2:                    |\
             |   AX                D             |   DS                                            |                                |\
             |       H L           I             |   ES                                            |                                |\
             |   BX                T             |   SS                                            |                                |\
             |       H L           S             |                                                 |                                |\
             |   CX                Z             |                                                 |                                |\
             |       H L           A             |                                                 |                                |\
             |   DX                P             |                                                 |                                |\
             |                     C             |                                                 |                                |\
             |                                   |                                                 |                                |\
             |   SI                              |                                                 |                                |\
             |   BP                              |                                                 |                                |\
             |   DI                              |                                                 |                                |\
             |   SP                              |                                                 |                                |\
             |                                   |                                                 |                                |\
             |   IP                              |                                                 |                                |\
             |                                   |                                                 |                                |\
             |                                   |                                                 |                                |\
             |                                   |                                                 |                                |\
             |                                   |                                                 |                                |\
             |                                   |                                                 |                                |\
             *===================================*=================================================*================================*\
             |   >>                              |                                                                                  |\
             *===================================*=================================================*================================*"
        ),
    ).unwrap();

    queue!(
        stdout(),
        MoveTo(7, 7),
        Print(format!("{:02X}", sys.cpu.ax.high)),
        Print(format!("{:02X}", sys.cpu.ax.low)),
        MoveTo(7, 9),
        Print(format!("{:02X}", sys.cpu.bx.high)),
        Print(format!("{:02X}", sys.cpu.bx.low)),
        MoveTo(7, 11),
        Print(format!("{:02X}", sys.cpu.cx.high)),
        Print(format!("{:02X}", sys.cpu.cx.low)),
        MoveTo(7, 13),
        Print(format!("{:02X}", sys.cpu.dx.high)),
        Print(format!("{:02X}", sys.cpu.dx.low)),
        MoveTo(7, 16),
        Print(format!("{:04X}", sys.cpu.si)),
        MoveTo(7, 17),
        Print(format!("{:04X}", sys.cpu.bp)),
        MoveTo(7, 18),
        Print(format!("{:04X}", sys.cpu.di)),
        MoveTo(7, 19),
        Print(format!("{:04X}", sys.cpu.sp)),
        MoveTo(7, 21),
        Print(format!("{:04X}", sys.cpu.ip)),
    )
    .unwrap();

    queue!(
        stdout(),
        MoveTo(24, 6),
        Print(format!("{}", sys.cpu.flags.o as i32)),
        MoveTo(24, 7),
        Print(format!("{}", sys.cpu.flags.d as i32)),
        MoveTo(24, 8),
        Print(format!("{}", sys.cpu.flags.i as i32)),
        MoveTo(24, 9),
        Print(format!("{}", sys.cpu.flags.t as i32)),
        MoveTo(24, 10),
        Print(format!("{}", sys.cpu.flags.s as i32)),
        MoveTo(24, 11),
        Print(format!("{}", sys.cpu.flags.z as i32)),
        MoveTo(24, 12),
        Print(format!("{}", sys.cpu.flags.a as i32)),
        MoveTo(24, 13),
        Print(format!("{}", sys.cpu.flags.p as i32)),
        MoveTo(24, 14),
        Print(format!("{}", sys.cpu.flags.c as i32)),
    )
    .unwrap();

    queue!(
        stdout(),
        MoveTo(45, 6),
        Print(format!("{:04X}", sys.cpu.cs)),
        MoveTo(45, 7),
        Print(format!("{:04X}", sys.cpu.ds)),
        MoveTo(45, 8),
        Print(format!("{:04X}", sys.cpu.es)),
        MoveTo(45, 9),
        Print(format!("{:04X}", sys.cpu.ss)),
    )
    .unwrap();

    queue!(
        stdout(),
        MoveTo(64, 6),
        Print(format!(
            "{:05X}     {:02X}",
            dir % 0x100000,
            sys.bus.memory[dir % 0x100000]
        )),
        MoveTo(64, 7),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 1) % 0x100000,
            sys.bus.memory[(dir + 1) % 0x100000]
        )),
        MoveTo(64, 8),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 2) % 0x100000,
            sys.bus.memory[(dir + 2) % 0x100000]
        )),
        MoveTo(64, 9),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 3) % 0x100000,
            sys.bus.memory[(dir + 3) % 0x100000]
        )),
        MoveTo(64, 10),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 4) % 0x100000,
            sys.bus.memory[(dir + 4) % 0x100000]
        )),
        MoveTo(64, 11),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 5) % 0x100000,
            sys.bus.memory[(dir + 5) % 0x100000]
        )),
        MoveTo(64, 12),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 6) % 0x100000,
            sys.bus.memory[(dir + 6) % 0x100000]
        )),
        MoveTo(64, 13),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 7) % 0x100000,
            sys.bus.memory[(dir + 7) % 0x100000]
        )),
        MoveTo(64, 14),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 8) % 0x100000,
            sys.bus.memory[(dir + 8) % 0x100000]
        )),
        MoveTo(64, 15),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 9) % 0x100000,
            sys.bus.memory[(dir + 9) % 0x100000]
        )),
        MoveTo(64, 16),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 10) % 0x100000,
            sys.bus.memory[(dir + 10) % 0x100000]
        )),
        MoveTo(64, 17),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 11) % 0x100000,
            sys.bus.memory[(dir + 11) % 0x100000]
        )),
        MoveTo(64, 18),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 12) % 0x100000,
            sys.bus.memory[(dir + 12) % 0x100000]
        )),
        MoveTo(64, 19),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 13) % 0x100000,
            sys.bus.memory[(dir + 13) % 0x100000]
        )),
        MoveTo(64, 20),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 14) % 0x100000,
            sys.bus.memory[(dir + 14) % 0x100000]
        )),
        MoveTo(64, 21),
        Print(format!(
            "{:05X}     {:02X}",
            (dir + 15) % 0x100000,
            sys.bus.memory[(dir + 15) % 0x100000]
        )),
    )
    .unwrap();

    let instr = dissasemble_one(&sys.bus, &sys.cpu);

    execute!(
        stdout(),
        MoveTo(100, 4),
        Print(format!("{}", instr.opcode)),
        MoveTo(100, 5),
        Print(format!("{}", instr.operand1)),
        MoveTo(100, 6),
        Print(format!("{}", instr.operand2)),
        MoveTo(7, 28),
    )
    .unwrap();
}

pub fn get_command(sys: &mut System) {
    let mut command = String::new();

    io::stdin().read_line(&mut command).expect("Failed");

    match command.trim_end() {
        "step" | "s" | "" => {
            sys.cpu.cycles = 0;
            sys.cpu.fetch_decode_execute(&mut sys.bus);
        }
        "quit" | "q" => {
            execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
            sys.running = false
        }
        "run" | "r" => {
            sys.cpu.halted = false;
            sys.cpu.cycles = 0;

            loop {
                let start = Instant::now();
                sys.update();
                let t = Instant::now().duration_since(start);
                let to_sleep = Duration::from_millis(20).saturating_sub(t);
                thread::sleep(to_sleep);
                if sys.cpu.halted {
                    break;
                }
            }
        }
        "reset" | "rst" => {
            sys.cpu = CPU::new();
            sys.bus = Bus::new()
        }
        "load_bios" | "lb" => sys.load_bios(),
        "s100" => {
            let mut a = 0x1000;
            while a > 0 {
                sys.cpu.cycles = 0;
                sys.cpu.fetch_decode_execute(&mut sys.bus);
                a -= 1;
            }
        }
        _ => {}
    }
}
