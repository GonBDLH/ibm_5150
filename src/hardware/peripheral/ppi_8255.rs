// use ggez::event::{self, KeyCode};

use ggez::event::KeyCode;

use super::{Peripheral, pic_8259::{PIC8259, IRQs}};

// IMPORTANTE: ESTAN AL REVES, LA POSICION 1 ES EL BIT 0.
//             ON = 0, OFF = 1
// const SW1: u8 = 0b00110000;
const SW1: u8 = 0b00110100;
// const SW2: u8 = 0b00001111;
const SW2: u8 = 0b11100000;

const KBD_RESET_CYCLES: u32 = 47700; // 20 ms
const KBD_RESET_CYCLE_DELAY: u32 = 100;

#[derive(Clone)]
pub struct PPI8255 {
    key_code: u8,
    pub port_b: u8,
    pub port_c: u8,
    mode_reg: u8,

    kbd: Keyboard,
}

#[derive(Clone)]
pub struct Keyboard {
    clear: bool,
    reset:bool,

    clk_low: bool,
    counting_low: bool,
    low_count: u32,

    count_until_reset: u32,
    resets_counter: u32,
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
        }
    }
}

fn decode_key(keycode: KeyCode) -> u8 {
    println!("{:?}", keycode);
    match keycode {
        KeyCode::Escape => 1,
        KeyCode::Key1 => 2, 
        KeyCode::Key2 => 3,
        KeyCode::Key3 => 4,
        KeyCode::Key4 => 5,
        KeyCode::Key5 => 6,
        KeyCode::Key6 => 7,
        KeyCode::Key7 => 8,
        KeyCode::Key8 => 9,
        KeyCode::Key9 => 10,
        KeyCode::Key0 => 11,
        KeyCode::Minus => 12,
        KeyCode::Equals => 13,
        KeyCode::Back => 14,
        KeyCode::Tab => 15,
        KeyCode::Q => 16,
        KeyCode::W => 17,
        KeyCode::E => 18,
        KeyCode::R => 19,
        KeyCode::T => 20,
        KeyCode::Y => 21,
        KeyCode::U => 22,
        KeyCode::I => 23,
        KeyCode::O => 24,
        KeyCode::P => 25,

        _ => 0,
    }
}

impl PPI8255 {
    pub fn new() -> Self {
        PPI8255 { 
            key_code: 0x00,
            port_b: 0x00,
            port_c: 0x00,
            mode_reg: 0x00,

            kbd: Keyboard::new(),
        }
    }

    pub fn key_up(&mut self, keycode: KeyCode, pic: &mut PIC8259) {
        // if self.keyboard_enabled {
        let key_code = decode_key(keycode) + 0x80;
        self.key_input(key_code, pic);
        // }
    }

    pub fn key_down(&mut self, keycode: KeyCode, pic: &mut PIC8259) {
        // if self.keyboard_enabled {
        let key_code = decode_key(keycode);
        self.key_input(key_code, pic);
        // }
    }
    
    pub fn key_input(&mut self, key_code: u8, pic: &mut PIC8259) {
        self.key_code = key_code;
        pic.irq(IRQs::Irq1);
    }
    
    fn read_pa(&mut self) -> u8 {
        if self.port_b & 0x80 == 0x80 {
            SW1
        } else {
            self.key_code
        }
    }

    pub fn read_pc(&mut self) -> u8 {
        if self.port_b & 0x04 == 0x04 {
            SW2 & 0x0F | self.port_c & 0xF0
        } else {
            (SW2 >> 4) & 0x01 | self.port_c & 0xF0
        }
    }

    // ESTO SIRVE PARA EL KBD_RESET
    pub fn update(&mut self, pic: &mut PIC8259, cycles: u32) {
        if self.kbd.clear {
            self.kbd.clear = false;
            self.key_code = 0;
            pic.clear_int(IRQs::Irq1);
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
            2 => {},
            1 => {
                let val = val as u8;
                self.port_b = val;
                
                if val & 0x80 != 0 {
                    self.kbd.clear = true;
                };
            },
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
}