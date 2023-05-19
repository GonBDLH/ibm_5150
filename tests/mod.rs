use ibm_5150::hardware::sys::System;

fn test_result(path: &str, mem: &[u8]) -> bool {
    let result = std::fs::read(path).unwrap();
    let mut pos = 0;
    let mut ret = true;

    result.iter().zip(mem).for_each(|(expected, obtained)| {
        if expected != obtained {
            println!("At {:04X} :: Expected: {:02X} - Obtained: {:02x}", pos, expected, obtained);
            ret = false;
        }

        pos += 1;
    });

    ret
}

fn run_test(sys: &mut System, path: &str) {
    sys.load_test(path);

    let mut cycles_ran = 0;

    while !sys.cpu.halted {
        sys.step(&mut cycles_ran);

        if cycles_ran > 100_000_000 {
            println!("TIMEOUT");
            return;
        }
    }
}

#[test]
fn test_186() {
    let mut res = true;
    
    let tests = [
        "add", "bcdcnv", "bitwise", "cmpneg", "control", "datatrnf", "div", "interrupt", "jmpmov", "jump1", "jump2",
        "mul", "rep", "rotate", "segpr", "shifts", "strings", "sub" 
    ];
        
    for i in tests {
        let mut sys = System::new();
        let test_path = format!("roms/tests/{i}.bin");
        let res_path = format!("roms/tests/res_{i}.bin");

        println!("\n\nTEST {i}");
        run_test(&mut sys, &test_path);
        res &= test_result(&res_path, &sys.bus.memory);
    }

    println!("\n");

    assert!(res);
}