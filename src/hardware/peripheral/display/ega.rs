use super::{crtc6845::CRTC6845, DisplayAdapter};

pub struct EGA {
    font_map: [[[bool; 8]; 14]; 256],

    crtc: CRTC6845,
}

impl DisplayAdapter for EGA {
    fn create_frame(&mut self, vram: &[u8]) -> Vec<u8> {
        todo!() 
    }

    fn inc_frame_counter(&mut self) {
        
    }
}