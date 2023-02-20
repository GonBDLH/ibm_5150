use ibm_5150::{IbmPc, hardware::{cpu_8088::{instr_utils::{Opcode, OperandType, Length, Operand}, CPU}, bus::Bus}};
use rand::random;

fn init_test() -> (CPU, Bus) {
    (CPU::new(), Bus::new())
}

pub fn test_mul2() {
    let (mut cpu, mut bus) = init_test();
    cpu.instr.opcode = Opcode::MUL;
    cpu.instr.data_length = Length::Word;
    cpu.instr.operand1 = OperandType::Register(Operand::CX);

    for _ in 0..100 {
        let i = rand::random::<u16>();
        let j = rand::random::<u16>();

        cpu.ax.set_x(i);
        cpu.cx.set_x(j);
        cpu.execute(&mut bus);
        let res = (i as u128 * j as u128) as u32;

        assert_eq!(((cpu.dx.get_x() as u32) << 16) + (cpu.ax.get_x() as u32), res);
            
        assert_eq!(cpu.flags.o && cpu.flags.c, res & 0xFFFF0000 > 0);
    }
}

pub fn test_mul1() {
    let (mut cpu, mut bus) = init_test();
    cpu.instr.opcode = Opcode::MUL;
    cpu.instr.data_length = Length::Byte;
    cpu.instr.operand1 = OperandType::Register(Operand::BL);

    for _ in 0..100 {
        let i = random();
        let j = random();
        
        cpu.ax.low = i;
        cpu.bx.low = j;
        cpu.execute(&mut bus);
        let res = (i as u128 * j as u128) as u16;

        assert_eq!(cpu.ax.get_x(), res);
        assert_eq!(cpu.flags.o && cpu.flags.c, res & 0xFF00 > 0);
    }
}

pub fn test_imul1() {
    let (mut cpu, mut bus) = init_test();
    cpu.instr.opcode = Opcode::IMUL;
    cpu.instr.operand1 = OperandType::Register(Operand::DX);
    cpu.instr.data_length = Length::Word;

    for _ in 0..100 {
        let i = random();
        let j = random();

        cpu.ax.set_x(i);
        cpu.dx.set_x(j);
        cpu.execute(&mut bus);
        let res = (i as i128 * j as i128) as u32;

        assert_eq!(((cpu.dx.get_x() as u32) << 16) + (cpu.ax.get_x() as u32), res);
        assert_eq!(cpu.flags.o && cpu.flags.c, res & 0xFFFF0000 > 0);
    }

    let mut sys = IbmPc::new();
    
    sys.sys.cpu.ax.set_x(0x0001);
    sys.sys.cpu.bx.set_x(0xFFFF);
    

    sys.sys.cpu.execute(&mut sys.sys.bus);

    assert_eq!(sys.sys.cpu.dx.get_x(), 0xFFFF);        
    assert_eq!(sys.sys.cpu.ax.get_x(), 0xFFFF);        
    assert!(!sys.sys.cpu.flags.c);        
    assert!(!sys.sys.cpu.flags.o);        
}

pub fn test_imul2() {
    let mut sys = IbmPc::new();
    
    sys.sys.cpu.ax.low = 0x80;
    sys.sys.cpu.bx.low = 0xFF;
    
    sys.sys.cpu.instr.opcode = Opcode::IMUL;
    sys.sys.cpu.instr.operand1 = OperandType::Register(Operand::BL);
    sys.sys.cpu.instr.data_length = Length::Byte;
    
    sys.sys.cpu.execute(&mut sys.sys.bus);
    
    assert_eq!(sys.sys.cpu.ax.get_x(), 0x0080);
    assert!(sys.sys.cpu.flags.c);        
    assert!(sys.sys.cpu.flags.o);        
}

pub fn test_imul3() {
    let mut sys = IbmPc::new();
    
    sys.sys.cpu.ax.set_x(0x0FFF);
    sys.sys.cpu.bx.set_x(0x0FFF);
    
    sys.sys.cpu.instr.opcode = Opcode::IMUL;
    sys.sys.cpu.instr.operand1 = OperandType::Register(Operand::BX);
    sys.sys.cpu.instr.data_length = Length::Word;

    sys.sys.cpu.execute(&mut sys.sys.bus);

    assert_eq!(sys.sys.cpu.dx.get_x(), 0x00FF);        
    assert_eq!(sys.sys.cpu.ax.get_x(), 0xE001);        
    assert!(sys.sys.cpu.flags.c);        
    assert!(sys.sys.cpu.flags.o);        
}