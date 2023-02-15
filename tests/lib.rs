#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use std::time::{Instant, Duration};
    
    use ibm_5150::{*, hardware::cpu_8088::instr_utils::{Opcode, OperandType, Length}};
    
    // fn write_instr(sys: &mut IbmPc, op: u8) {
    //     sys.sys.bus.memory[0xFFFF0] = op;
    // }
    
    // #[test]
    // fn test() {
    //     let mut app = IbmPc::new();
    
    //     app.sys.rst();
    //     app.sys.load_roms();
 
    //     let sec = 120.;
    //     let frames = (DESIRED_FPS * sec) as usize; // 9 Segundos

    //     let start = Instant::now();
    //     for _i in 0..frames {
    //         app.sys.update();
    //     }
    //     let t = Instant::now().duration_since(start);
    //     println!("Duracion: {} ms -> {frames}", t.as_millis());
    //     assert!(t.as_millis() < (sec * 1000.) as u128);
    // }

    // #[test]
    // fn test_mov() {
    //     let mut sys = IbmPc::new();
    //     let mut instr = 0b10001000;

    //     for i in 0..4 {
    //         instr += i;
    //         write_instr(&mut sys, instr);
    //     }

    // }
    
    #[test]
    fn test_mul() {
        let mut sys = IbmPc::new();
        
        sys.sys.cpu.ax.low = 0x05;
        sys.sys.cpu.bx.low = 0x10;
        
        sys.sys.cpu.instr.opcode = Opcode::MUL;
        sys.sys.cpu.instr.operand1 = OperandType::Register(hardware::cpu_8088::instr_utils::Operand::BL);
        sys.sys.cpu.instr.data_length = Length::Word;

        sys.sys.cpu.execute(&mut sys.sys.bus);

        assert_eq!(sys.sys.cpu.ax.get_x(), 0x0050);        
        assert!(!sys.sys.cpu.flags.c);        
    }

    #[test]
    fn test_imul1() {
        let mut sys = IbmPc::new();
        
        sys.sys.cpu.ax.set_x(0x0001);
        sys.sys.cpu.bx.set_x(0xFFFF);
        
        sys.sys.cpu.instr.opcode = Opcode::IMUL;
        sys.sys.cpu.instr.operand1 = OperandType::Register(hardware::cpu_8088::instr_utils::Operand::BX);
        sys.sys.cpu.instr.data_length = Length::Word;

        sys.sys.cpu.execute(&mut sys.sys.bus);

        assert_eq!(sys.sys.cpu.dx.get_x(), 0xFFFF);        
        assert_eq!(sys.sys.cpu.ax.get_x(), 0xFFFF);        
        assert!(!sys.sys.cpu.flags.c);        
        assert!(!sys.sys.cpu.flags.o);        
    }
    
    
    #[test]
    fn test_imul2() {
        let mut sys = IbmPc::new();
        
        sys.sys.cpu.ax.low = 0x80;
        sys.sys.cpu.bx.low = 0xFF;
        
        sys.sys.cpu.instr.opcode = Opcode::IMUL;
        sys.sys.cpu.instr.operand1 = OperandType::Register(hardware::cpu_8088::instr_utils::Operand::BL);
        sys.sys.cpu.instr.data_length = Length::Byte;
        
        sys.sys.cpu.execute(&mut sys.sys.bus);
        
        assert_eq!(sys.sys.cpu.ax.get_x(), 0x0080);
        assert!(sys.sys.cpu.flags.c);        
        assert!(sys.sys.cpu.flags.o);        
    }

    #[test]
    fn test_imul3() {
        let mut sys = IbmPc::new();
        
        sys.sys.cpu.ax.set_x(0x0FFF);
        sys.sys.cpu.bx.set_x(0x0FFF);
        
        sys.sys.cpu.instr.opcode = Opcode::IMUL;
        sys.sys.cpu.instr.operand1 = OperandType::Register(hardware::cpu_8088::instr_utils::Operand::BX);
        sys.sys.cpu.instr.data_length = Length::Word;

        sys.sys.cpu.execute(&mut sys.sys.bus);

        assert_eq!(sys.sys.cpu.dx.get_x(), 0x00FF);        
        assert_eq!(sys.sys.cpu.ax.get_x(), 0xE001);        
        assert!(sys.sys.cpu.flags.c);        
        assert!(sys.sys.cpu.flags.o);        
    }
}