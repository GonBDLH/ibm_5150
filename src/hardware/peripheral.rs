pub trait Peripheral {
    fn port_in(&self, port: u16) -> u8;
    fn port_out(&mut self, val: u16, port: u16);
    fn is_connected(&self, port: u16) -> bool;
}