use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use super::{
    pic_8259::{IRQs, PIC8259},
    Peripheral,
};

const KBD_RESET_CYCLES: u32 = 47700; // 20 ms
const KBD_RESET_CYCLE_DELAY: u32 = 100;

static mut TEST: usize = 0;

#[derive(Default)]
pub struct PPI8255 {
    key_code: u8,
    pub port_b: u8,
    pub port_c: u8,
    mode_reg: u8,

    kbd: Keyboard,

    sw1: u8,
    sw2: u8,
}

pub struct Keyboard {
    clear: bool,
    reset: bool,

    clk_low: bool,
    counting_low: bool,
    low_count: u32,

    count_until_reset: u32,
    resets_counter: u32,

    key_queue: VecDeque<u8>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            clear: false,
            reset: false,

            clk_low: false,
            counting_low: false,
            low_count: 0,

            count_until_reset: 0,
            resets_counter: 0,

            key_queue: VecDeque::with_capacity(50),
        }
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard::new()
    }
}

impl PPI8255 {
    pub fn new(sw1: u8, sw2: u8) -> Self {
        PPI8255 {
            sw1,
            sw2,
            ..Default::default()
        }
    }

    // TODO
    pub fn key_up(&mut self, keycode: u8) {
        self.kbd.key_queue.push_front(keycode + 0x80);
    }

    // TODO
    pub fn key_down(&mut self, keycode: u8) {
        self.kbd.key_queue.push_front(keycode);
    }

    pub fn key_input(&mut self, pic: &mut PIC8259) {
        if let Some(key_code) = self.kbd.key_queue.pop_back() {
            self.key_code = key_code;
            pic.irq(IRQs::Irq1);
        }
    }

    fn read_pa(&mut self) -> u8 {
        if self.port_b & 0x80 == 0x80 {
            self.sw1
        } else {
            self.key_code
        }
    }

    pub fn read_pc(&mut self) -> u8 {
        if self.port_b & 0x04 != 0 {
            self.sw2 & 0x0F | self.port_c & 0xF0
        } else {
            (self.sw2 >> 1) & 0x0F | self.port_c & 0xF0
        }
    }
}

impl Peripheral for PPI8255 {
    fn port_in(&mut self, port: u16) -> u16 {
        let port = port & 0x3;
        match port {
            3 => self.mode_reg as u16,
            2 => self.read_pc() as u16,
            1 => self.port_b as u16,
            0 => self.read_pa() as u16,
            _ => unreachable!(),
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        // TODO
        let port = port & 0x3;
        match port {
            3 => self.mode_reg = val as u8,
            2 => {}
            1 => {
                let val = val as u8;
                self.port_b = val;

                if val & 0x80 != 0 {
                    self.kbd.clear = true;
                };
            }
            0 => self.key_code = val as u8,
            _ => unreachable!(),
        };

        if self.port_b & 0x40 == 0 {
            self.kbd.clk_low = true;
            self.kbd.counting_low = true;
        } else if self.kbd.clk_low {
            self.kbd.clk_low = false;

            if self.kbd.low_count > KBD_RESET_CYCLES {
                self.kbd.reset = true;
                self.kbd.low_count = 0;
                self.kbd.count_until_reset = 0;
            }
        } else {
            self.kbd.counting_low = false;
            self.kbd.low_count = 0;
        }
    }

    // ESTO SIRVE PARA EL KBD_RESET Y LEER EL TECLADO
    fn update(&mut self, pic: &mut PIC8259, cycles: u32) {
        if self.kbd.clear {
            self.kbd.clear = false;
            self.key_code = 0;
        }

        if self.kbd.counting_low && self.kbd.low_count < KBD_RESET_CYCLES {
            self.kbd.low_count += cycles;
        }

        if self.kbd.reset {
            self.kbd.count_until_reset += cycles;

            if self.kbd.count_until_reset > KBD_RESET_CYCLE_DELAY {
                self.kbd.reset = false;
                self.kbd.count_until_reset = 0;
                self.kbd.resets_counter += 1;

                self.key_code = 0xAA;
                pic.irq(IRQs::Irq1);
            }
        }
    }
}
