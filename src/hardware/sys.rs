use std::thread::sleep;
use std::time::{Instant, Duration};

use super::cpu::CPU;
use super::bus::Bus;

pub struct System {
    pub cpu: CPU,
    bus: Bus,
}

impl System {
    pub fn new() -> Self {
        System { 
            cpu: CPU::new(),
            bus: Bus::new(),
        }
    }
}

impl System {
    pub fn clock(self: &mut Self) {
        loop {
            let start = Instant::now();
            // for _i in 0..286364 {
            //     if self.clock_cycles % 3 == 0 {
            //         self.cpu.step();
            //     }

            //     if self.clock_cycles % 4 == 0 {
            //         // Aqui CGA
            //     }

            //     if self.clock_cycles % 12 == 0 {
            //         // Aqui timers 8253
            //     }
                
            //     self.clock_cycles += 1;
            // }
            self.cpu.step(&mut self.bus);

            let t = Duration::new(0, 20_000_000).checked_sub(Duration::from(Instant::now() - start)).unwrap_or_default();
            println!("{}", t.as_micros());
            sleep(t);
        }
    }
}