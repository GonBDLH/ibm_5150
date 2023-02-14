#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use std::time::{Instant, Duration};
    
    use ibm_5150::*;
    
    fn write_instr(sys: &mut IbmPc, op: u8) {
        sys.sys.bus.memory[0xFFFF0] = op;
    }
    
    #[test]
    fn test() {
        let mut app = IbmPc::new();
    
        app.sys.rst();
        app.sys.load_roms();
 
        let sec = 120.;
        let frames = (DESIRED_FPS * sec) as usize; // 9 Segundos

        let start = Instant::now();
        for _i in 0..frames {
            app.sys.update();
        }
        let t = Instant::now().duration_since(start);
        println!("Duracion: {} ms -> {frames}", t.as_millis());
        assert!(t.as_millis() < (sec * 1000.) as u128);
    }

    #[test]
    fn test_mov() {
        let mut sys = IbmPc::new();
        let mut instr = 0b10001000;

        for i in 0..4 {
            instr += i;
            write_instr(&mut sys, instr);
        }

    }
}