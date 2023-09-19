pub mod dma_8237;
pub mod fdc_necupd765;
pub mod pic_8259;
pub mod ppi_8255;
pub mod timer_8253;

pub trait Peripheral {
    fn port_in(&mut self, port: u16) -> u16;
    fn port_out(&mut self, val: u16, port: u16);
}
