use std::ops::DerefMut;

use super::Peripheral;

pub struct PIC8259 {
    isr: u8,
    imr: u8,
    pub irr: u8,

    lower_priority: u8,
    // icw: [u8; 4],
    icw_step: usize,

    icw_4_needed: bool,
    single: bool,                 // Not necessary, since IBM 5150 only has one PIC
    call_address_interval: usize, // Not necessary
    trigger_mode: TriggerMode,

    interrupt_vector: u8,

    eoi_mode: EOIMode,
    special_fully_nested_mode: bool,
    special_mask_mode: bool,
}

#[derive(Clone, Copy)]
pub enum IRQs {
    Irq0 = 0b00000001,
    Irq1 = 0b00000010,
    Irq2 = 0b00000100,
    Irq3 = 0b00001000,
    Irq4 = 0b00010000,
    Irq5 = 0b00100000,
    Irq6 = 0b01000000,
    Irq7 = 0b10000000,
}

impl PIC8259 {
    pub fn new() -> Self {
        Self {
            isr: 0,    // In-Service Register
            imr: 0xFF, // Interrupt Mask Register
            irr: 0,    // Interrupt Request Register

            lower_priority: 7,
            // icw: [0; 4],
            icw_step: 0,

            icw_4_needed: false,
            single: false,
            call_address_interval: 4,
            trigger_mode: TriggerMode::LevelTriggered,

            interrupt_vector: 0,

            eoi_mode: EOIMode::NormalEOI,
            special_fully_nested_mode: true,
            special_mask_mode: false,
        }
    }

    pub fn try_aeoi(&mut self) {
        if self.eoi_mode != EOIMode::AutoEOI {
            return;
        }

        self.non_specific_eoi();
    }

    fn specific_eoi(&mut self, val: u8) {
        let ir_level = val & 0b00000111;

        self.isr &= !(1 << ir_level);
    }

    fn non_specific_eoi(&mut self) {
        let seriviced_ints = if self.special_mask_mode {
            self.isr & !self.imr
        } else {
            self.isr
        };

        for i in 0..8 {
            let next_int = 1u8.rotate_left(((self.get_highest_prio() + i) % 8) as u32);

            if (seriviced_ints & next_int) > 0 {
                self.isr ^= next_int;
                // return;
            }
        }
    }

    fn next_icw_step(&mut self) {
        match self.icw_step {
            0 => self.icw_step = 1,
            1 => {
                if self.single {
                    self.icw_step = 3;
                }

                if !self.icw_4_needed {
                    self.icw_step = 4;
                }
            }
            2 => {
                self.icw_step = 3;
                if !self.icw_4_needed {
                    self.icw_step = 4;
                }
            }
            3 => {
                self.icw_step = 4;
            }
            _ => {}
        }
    }

    fn get_highest_prio(&self) -> u8 {
        (self.lower_priority + 1) % 8
    }

    fn has_greater_prio(&self, int: u8) -> bool {
        let highest_prio = self.get_highest_prio();
        let isr_ordered = self.isr.rotate_right(highest_prio as u32);

        int > isr_ordered
    }
}

impl Default for PIC8259 {
    fn default() -> Self {
        PIC8259::new()
    }
}

impl PIC8259 {
    pub fn get_next(&mut self) -> Option<u8> {
        let requested_ints = self.irr & !self.imr;

        if requested_ints == 0 {
            return None;
        }

        for i in 0..8 {
            let next_int = (self.get_highest_prio() + i) % 8;
            let next_int_bit = 1u8.rotate_left(next_int as u32);

            if requested_ints & next_int_bit > 0 {
                if !self.has_greater_prio(next_int_bit) {
                    continue;
                }

                self.isr |= next_int_bit;
                self.irr ^= next_int_bit;
                return Some(self.interrupt_vector + next_int);
            }
        }

        None
    }

    pub fn has_int(&mut self) -> bool {
        (!self.imr & self.irr) > 0
    }

    pub fn irq(&mut self, irq: IRQs) {
        self.irr |= irq as u8;
    }

    pub fn clear_int(&mut self, irq: IRQs) {
        self.irr &= !(irq as u8);
    }
}

impl Peripheral for PIC8259 {
    fn port_in(&mut self, port: u16) -> u16 {
        if port == 0x20 {
            self.irr as u16
        } else {
            self.imr as u16
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        let val = val as u8;

        match port {
            0x20 => {
                if val & 0b00010000 > 0 {
                    // ICW1
                    self.icw_step = 0;
                    self.icw_4_needed = val & 0b00000001 > 0;
                    self.single = val & 0b00000010 > 0;
                    self.call_address_interval = if val & 0b00000100 > 0 { 4 } else { 8 };
                    self.trigger_mode = if val & 0b00001000 > 0 {
                        TriggerMode::LevelTriggered
                    } else {
                        TriggerMode::EdgeTriggered
                    };

                    self.icw_step += 1;

                    self.imr = 0xFF;
                    // IR7 -> PRIO 7
                    self.lower_priority = 7;
                    // TODO Special Mask Mode cleared and Status Read -> IRR
                    self.special_mask_mode = false;
                } else if val & 0b00011000 == 0 {
                    // TODO OCW2
                    match val & 0b11100000 {
                        0b00100000 => {
                            // NON-SPECIFIC EOI
                            self.non_specific_eoi();
                            // println!("{}", val);
                        }
                        0b01100000 => {
                            // SPECIFIC EOI
                            self.specific_eoi(val);
                        }
                        0b10100000 => {
                            // TODO ROTATE ON NON-SPECIFIC EOI COMMAND
                        }
                        0b10000000 => {
                            // TODO ROTATE IN AEOI MODE SET
                        }
                        0b00000000 => {
                            // TODO ROTATE IN AEOI MODE CLEAR
                        }
                        0b11100000 => {
                            // TODO ROTATE ON SPECIFIC EOI COMMAND
                        }
                        0b11000000 => {
                            // SET PRIORITY COMMAND
                            let lower_prio = val & 0b00000111;
                            self.lower_priority = lower_prio;
                        }
                        _ => { /* NO OPERATION */ }
                    }
                } else if val & 0b10011000 == 0b00001000 {
                    // TODO OCW3
                }
            }
            0x21 => {
                match self.icw_step {
                    1 => {
                        // TODO ICW2
                        self.interrupt_vector = val;
                    }
                    2 => {
                        // ICW3
                        // Not needed, IBM 5150 only has one PIC
                    }
                    3 => {
                        // ICW4
                        self.eoi_mode = if val & 0b00000010 > 0 {
                            EOIMode::AutoEOI
                        } else {
                            EOIMode::NormalEOI
                        };
                        self.special_fully_nested_mode = val & 0b00010000 > 0;
                    }
                    _ => {
                        // OCW1
                        self.imr = val;
                    }
                }

                self.next_icw_step();
            }
            _ => unreachable!(),
        }
    }

    fn update(&mut self, _cycles: u32) {}
}

enum TriggerMode {
    LevelTriggered,
    EdgeTriggered,
}

#[derive(PartialEq, Eq)]
enum EOIMode {
    AutoEOI,
    NormalEOI,
}
