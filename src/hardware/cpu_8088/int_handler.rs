use crate::hardware::bus::Bus;

use super::CPU;

impl CPU {
    pub fn hw_interrup(&mut self, bus: &mut Bus) {
        // TODO Comprueba si hay interrupt
        if true {
            return
        }
    
        self.push_stack_16(bus, self.flags.get_flags());
        self.flags.i = false;
        self.flags.t = false;
    
        self.push_stack_16(bus, self.cs);
        self.push_stack(bus, self.ip);
    
    }
    
    pub fn interrupt(&mut self, bus: &mut Bus, int_type: u8) {
        self.push_stack_16(bus, self.flags.get_flags());
        self.flags.i = false;
        self.flags.t = false;
        
        self.push_stack_16(bus, self.cs);
        self.cs = (4 * int_type + 2) as u16;
        self.push_stack(bus, self.ip);
        self.ip = (4 * int_type) as u16;
    }
}
