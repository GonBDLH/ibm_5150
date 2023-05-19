use super::{
    pic_8259::{IRQs, PIC8259},
    ppi_8255::PPI8255,
    Peripheral,
};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum Mode {
    #[default]
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
}

#[derive(Default, Clone)]
pub struct TIM8253 {
    pub cycles: u32,

    count: [u16; 3],
    reload: [u16; 3],
    latched: [bool; 3],
    latch_val: [u16; 3],
    rl_mode: [u8; 3],
    mode: [Mode; 3],
    out: [bool; 3],
    active: [bool; 3],
    first_clk: [bool; 3],
    reload_clk: [bool; 3],

    toggle: [bool; 3],

    mode_reg: u8,
}

impl TIM8253 {
    pub fn new() -> Self {
        Self {
            active: [false; 3],
            first_clk: [true; 3],
            reload_clk: [false; 3],
            toggle: [true; 3],
            ..Default::default()
        }
    }

    fn output(&mut self, channel: usize, state: bool, pic: &mut PIC8259) {
        if !self.out[channel] && state && channel == 0 {
            pic.irq(IRQs::Irq0);
        }
        self.out[channel] = state;
    }

    fn mode0(&mut self, i: usize, pic: &mut PIC8259) {
        self.count[i] = self.count[i].wrapping_sub(1);
        if self.count[i] == 0 {
            self.output(i, true, pic)
        }
    }

    fn mode2(&mut self, i: usize, pic: &mut PIC8259) {
        self.count[i] = self.count[i].wrapping_sub(1);
        if self.count[i] == 1 {
            self.output(i, false, pic);
        } else {
            self.output(i, true, pic);
            if self.count[i] == 0 {
                self.count[i] = self.reload[i];
            }
        }
    }

    fn mode3(&mut self, i: usize, pic: &mut PIC8259) {
        let half = (self.reload[i] as f32 / 2.0).ceil() as u16;

        self.output(i, self.count[i] <= half, pic);

        if self.count[i] % 2 == 0 && self.out[i] && self.first_clk[i] {
            if self.reload_clk[i] {
                self.count[i] = self.count[i].wrapping_sub(3);
                self.reload_clk[i] = false;
            } else {
                self.count[i] = self.count[i].wrapping_sub(1);
            }

            self.first_clk[i] = false;
        } else {
            self.count[i] = self.count[i].wrapping_sub(2);
        }

        if self.count[i] == 0 {
            self.count[i] = self.reload[i];
            self.first_clk[i] = true;
            self.reload_clk[i] = true;
        }
    }

    pub fn update(&mut self, pic: &mut PIC8259, _ppi: &mut PPI8255) {
        while self.cycles > 3 {
            for i in 0..3 {
                if !self.active[i] {
                    continue;
                }
                match self.mode[i] {
                    Mode::Mode0 => self.mode0(i, pic),
                    Mode::Mode2 => self.mode2(i, pic),
                    Mode::Mode3 => self.mode3(i, pic),

                    _ => {} // TODO
                }
            }

            self.cycles -= 4;
        }
    }
}

impl Peripheral for TIM8253 {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;

                let val = if self.latched[channel] {
                    self.latch_val[channel]
                } else {
                    self.count[channel]
                };

                match self.rl_mode[channel] {
                    0b01 => {
                        // Invertir si latched esta activo
                        self.latched[channel] ^= self.latched[channel];
                        val as u8 as u16
                    }
                    0b10 => {
                        self.latched[channel] ^= self.latched[channel];
                        val >> 8
                    }
                    0b11 => {
                        if self.toggle[channel] {
                            self.toggle[channel] = false;
                            val as u8 as u16
                        } else {
                            self.toggle[channel] = true;
                            self.latched[channel] ^= self.latched[channel];
                            val >> 8
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => 0,
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;

                match self.rl_mode[channel] {
                    0b01 => {
                        self.reload[channel] = self.reload[channel] & 0xFF00 | val & 0x00FF;
                    }
                    0b10 => {
                        self.reload[channel] = self.reload[channel] & 0x00FF | val & 0xFF00;
                    }
                    0b11 => {
                        if self.toggle[channel] {
                            self.toggle[channel] = false;
                            self.reload[channel] = self.reload[channel] & 0xFF00 | val & 0x00FF;
                        } else {
                            self.toggle[channel] = true;
                            self.reload[channel] = self.reload[channel] & 0x00FF | val & 0xFF00;
                        }
                    }
                    _ => unreachable!(),
                }

                if self.rl_mode[channel] < 0b11 || self.toggle[channel] {
                    self.count[channel] = self.reload[channel];
                    self.active[channel] = true;
                    self.out[channel] =
                        self.mode[channel] == Mode::Mode2 || self.mode[channel] == Mode::Mode3;
                }
            }
            0x43 => {
                self.mode_reg = val as u8;
                let channel = ((self.mode_reg & 0b11000000) >> 6) as usize;
                let access_mode = (self.mode_reg & 0b00110000) >> 4;

                if access_mode == 0b00 {
                    self.latch_val[channel] = self.count[channel];
                    self.latched[channel] = true;
                } else {
                    self.rl_mode[channel] = access_mode;
                    let mode = (self.mode_reg & 0b00001110) >> 1;

                    match mode {
                        0b000 => self.mode[channel] = Mode::Mode0,
                        0b001 => self.mode[channel] = Mode::Mode1,
                        0b010 | 0b110 => self.mode[channel] = Mode::Mode2,
                        0b011 | 0b111 => self.mode[channel] = Mode::Mode3,
                        0b100 => self.mode[channel] = Mode::Mode4,
                        0b101 => self.mode[channel] = Mode::Mode5,
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
