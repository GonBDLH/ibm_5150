mod hardware;

use hardware::sys::System;

fn main() {
    let mut sys = System::new();

    sys.clock();

}

#[cfg(test)]
mod tests {
    use crate::hardware::{cpu::CPU, bus::Bus};

    #[test]
    fn mov_al_bl() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        bus.memory[0] = 0b10001010;
        bus.memory[1] = 0b11000011;
        
        cpu.bx.low = 0x69;
        assert_eq!(cpu.ax.low, 0);
        cpu.fetch_decode_execute(&mut bus);

        assert_eq!(cpu.ax.low, 0x69);
    }

    #[test]
    fn mov_ax_bx() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        bus.memory[0] = 0b10001011;
        bus.memory[1] = 0b11000011;

        cpu.bx.set_x(0x6942);
        assert_eq!(cpu.ax.low, 0);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.get_x(), 0x6942);
    }

    #[test]
    fn mov_al_bxdi1000() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        bus.memory[0] = 0b10001010;
        bus.memory[1] = 0b10000001;
        bus.memory[2] = 0b11101000;
        bus.memory[3] = 0b00000011;

        bus.write_8(0, 1000, 0x42);
        assert_eq!(cpu.ax.get_x(), 0);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.low, 0x42);
    }

    #[test]
    fn mov_bxdi1000_al() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        bus.memory[0] = 0b10001000;
        bus.memory[1] = 0b10000001;
        bus.memory[2] = 0b11101000;
        bus.memory[3] = 0b00000011;

        //bus.write_8(0, 1000, 0x42);
        cpu.ax.low = 0x42;
        assert_eq!(bus.read_8(0, 1000), 0);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(bus.read_8(0, 1000), 0x42);
    }

    #[test]
    fn mov_al_0x42() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        bus.memory[0] = 0b11000110;
        bus.memory[1] = 0b11000000;
        bus.memory[2] = 0x42;

        assert_eq!(cpu.ax.low, 0);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.low, 0x42);
    }
    
    #[test]
    fn mov_bpsi_0x42() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        bus.memory[0] = 0b11000110;
        bus.memory[1] = 0b00000010;
        bus.memory[2] = 0x42;

        cpu.bp = 10;
        cpu.si = 10;

        assert_eq!(bus.read_8(0, 20), 0);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(bus.read_8(0, 20), 0x42);
    }

    #[test]
    fn mov_ax_0x6942() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        bus.memory[0] = 0b11000111;
        bus.memory[1] = 0b11000000;
        bus.memory[2] = 0x42;
        bus.memory[3] = 0x69;

        assert_eq!(cpu.ax.get_x(), 0);
        cpu.fetch_decode_execute(&mut bus);
        assert_eq!(cpu.ax.get_x(), 0x6942);
    }
}