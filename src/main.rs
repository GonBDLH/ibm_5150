mod hardware;

use hardware::sys::System;

fn main() {
    let mut sys = System::new();

    sys.run();
}

#[cfg(test)]
mod tests {
    use crate::hardware::{cpu::CPU, bus::Bus};

    fn ini(cpu: &mut CPU) {
        cpu.cs = 0;
        cpu.ip = 0;
    }

    #[test]
    fn mov_al_bl() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0b10001010;
        bus.memory[1] = 0b11000011;
        
        cpu.bx.low = 0x69;
        cpu.fetch_decode_execute(&mut bus);

        assert_eq!(cpu.ax.low, 0x69);
    }

    #[test]
    fn mov_ax_bx() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0b10001011;
        bus.memory[1] = 0b11000011;

        cpu.bx.set_x(0x6942);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.get_x(), 0x6942);
    }

    #[test]
    fn mov_al_bxdi1000() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0b10001010;
        bus.memory[1] = 0b10000001;
        bus.memory[2] = 0b11101000;
        bus.memory[3] = 0b00000011;

        bus.write_8(0, 1000, 0x42);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.low, 0x42);
    }

    #[test]
    fn mov_bxdi1000_al() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0b10001000;
        bus.memory[1] = 0b10000001;
        bus.memory[2] = 0b11101000;
        bus.memory[3] = 0b00000011;

        //bus.write_8(0, 1000, 0x42);
        cpu.ax.low = 0x42;
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(bus.read_8(0, 1000), 0x42);
    }

    #[test]
    fn mov_al_0x42() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0b11000110;
        bus.memory[1] = 0b11000000;
        bus.memory[2] = 0x42;

        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.low, 0x42);
    }
    
    #[test]
    fn mov_bpsi_0x42() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0b11000110;
        bus.memory[1] = 0b00000010;
        bus.memory[2] = 0x42;

        cpu.bp = 10;
        cpu.si = 10;

        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(bus.read_8(0, 20), 0x42);
    }

    #[test]
    fn mov_ax_0x6942() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0b11000111;
        bus.memory[1] = 0b11000000;
        bus.memory[2] = 0x42;
        bus.memory[3] = 0x69;

        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.get_x(), 0x6942);
    }

    // Mem to AL
    #[test]
    fn mov_al_0x1234() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0xA0;
        bus.memory[1] = 0x34;
        bus.memory[2] = 0x12;

        bus.memory[0x1234] = 0x69;

        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.low, 0x69)
    }

    #[test]
    // AX to Mem
    fn mov_0x1234_al() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0xA3;
        bus.memory[1] = 0x34;
        bus.memory[2] = 0x12;

        cpu.ax.set_x(0x6942);

        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(bus.read_16(cpu.ds, 0x1234), 0x6942)
    }

    #[test]
    fn mov_ds_bx() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0x8E;
        bus.memory[1] = 0xDB;

        cpu.bx.set_x(0x6942);

        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ds, 0x6942);
    }

    #[test]
    fn push_ax() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        bus.memory[0] = 0xFF;
        bus.memory[1] = 0b11110000;

        cpu.ax.set_x(0x6942);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(bus.read_16(cpu.ss, cpu.sp - 2), 0x6942);
    }

    #[test]
    fn add_ax_0x7fimm() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();
        ini(&mut cpu);

        cpu.ax.set_x(0x1000);

        bus.memory[0] = 0x83;
        bus.memory[1] = 0b11000000;
        bus.memory[2] = 0x7F;
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.get_x(), 0x107F);
    }
}