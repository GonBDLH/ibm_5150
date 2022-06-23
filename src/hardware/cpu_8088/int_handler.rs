use crate::hardware::bus::Bus;

use super::CPU;

pub fn hw_interrup(cpu: &mut CPU, bus: &mut Bus) {
    // TODO Comprueba si hay interrupt
    if true {
        return
    }

    cpu.push_stack_16(bus, cpu.flags.get_flags());
    cpu.flags.i = false;
    cpu.flags.t = false;

    cpu.push_stack_16(bus, cpu.cs);
    cpu.push_stack(bus, cpu.ip);

}

pub fn sw_interrupt(cpu: &mut CPU, bus: &mut Bus, int_type: u8) {
    cpu.push_stack_16(bus, cpu.flags.get_flags());
    cpu.flags.i = false;
    cpu.flags.t = false;
    
    cpu.push_stack_16(bus, cpu.cs);
    cpu.cs = (4 * int_type + 2) as u16;
    cpu.push_stack(bus, cpu.ip);
    cpu.ip = (4 * int_type) as u16;
}