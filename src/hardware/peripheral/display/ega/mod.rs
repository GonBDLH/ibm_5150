use attribute_controller::EgaAttributeController;
use crtc::EgaCrtc;
use graphics_controller::EgaGraphicsController;
use sequencer::Sequencer;

use crate::hardware::peripheral::{pic_8259::PIC8259, Peripheral};

use super::DisplayAdapter;

mod sequencer;
mod crtc;
mod graphics_controller;
mod attribute_controller;

pub struct EGA {
    font_map: [[[bool; 8]; 14]; 256],
    sequencer: Sequencer,
    crtc: EgaCrtc,
    graphics_controller: EgaGraphicsController,
    attribute_controller: EgaAttributeController,

    misc_out_reg: u8,
    feature_ctrl_reg: u8,
    input_status_reg0: u8,
    input_status_reg1: u8,
}

impl EGA {
    pub fn new() -> Self {
        todo!()
    }
}

impl DisplayAdapter for EGA {
    fn create_frame(&mut self, _vram: &[u8]) -> Vec<u8> {
        todo!() 
    }

    fn inc_frame_counter(&mut self) {
        
    }
}

impl Peripheral for EGA {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x3C2 => self.input_status_reg0 as u16,
            0x3C4 => self.sequencer.read_reg(port as usize) as u16,
            0x3BA | 0x3DA => self.input_status_reg1 as u16,
            _ => 0
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x3C2 => self.misc_out_reg = val as u8,
            0x3C4 | 0x3C5 => self.sequencer.write_reg(port as usize, val as u8),
            0x3CC | 0x3CA | 0x3CE | 0x3CF => self.sequencer.write_reg(port as usize, val as u8),
            // TODO Reset reg action
            0x3BA | 0x3DA => self.feature_ctrl_reg = val as u8,
            _ => {}
        }
    }

    fn update(&mut self, _pic: &mut PIC8259, _cycles: u32) {}
}