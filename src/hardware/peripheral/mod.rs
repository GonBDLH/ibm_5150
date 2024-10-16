use pic_8259::PIC8259;

pub mod display;
pub mod dma_8237;
pub mod fdc_necupd765;
pub mod pic_8259;
pub mod pit_8253;
pub mod ppi_8255;

pub trait Peripheral {
    fn port_in(&mut self, port: u16) -> u16;
    fn port_out(&mut self, val: u16, port: u16);
    fn update(&mut self, cycles: u32);
}
