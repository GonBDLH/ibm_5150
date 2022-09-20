use crate::hardware::cpu_8088::{CPU, cpu_utils::get_address};

pub fn debug(cpu: &mut CPU) {
    match get_address(cpu) {
        0xFE05B => println!("Test 1"),
        0xFE0B0 => println!("Test 2"),
        0xFE0DA => println!("Test 3"),
        0xFE158 => println!("Test 4"),
        0xFE33B => println!("Test 5"),
        0xFE235 => println!("Test 6"),
        0xFE285 => println!("Test 7"),
        0xFE352 => println!("Test 8"),
        0xFE3AF => println!("Test 9"),
        0xFE3C0 => println!("Test 10"),
        0xFE3F8 => println!("Test 11"),
        0xFE4C7 => println!("Test 12"),
        0xFE521 => println!("Test 13"),
        0xFE55C => println!("Test 14"),

        0xFE6CA => println!("ERROR"),
        0xFE630 => println!("ERR_BEEP"),
        _ => {}
    }
}
