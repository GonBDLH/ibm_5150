use ibm_5150::*;

fn write_instr(sys: &mut IbmPc, op: u8) {
    sys.sys.bus.memory[0xFFFF0] = op;
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
