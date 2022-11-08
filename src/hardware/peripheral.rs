pub trait Peripheral {
    fn port_in(&mut self, port: u16) -> u16;
    fn port_out(&mut self, val: u16, port: u16);
}