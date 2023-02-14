use lazy_static::lazy_static;

use crate::hardware::cpu_8088::{CPU, cpu_utils::get_address};
use std::collections::VecDeque;
use std::sync::Mutex;

lazy_static!(
    static ref IP_QUEUE: Mutex<VecDeque<u16>> = Mutex::new(VecDeque::with_capacity(4));
);

pub fn debug_82(cpu: &mut CPU) {
    let mut ip_queue_lock = IP_QUEUE.lock().unwrap();
    ip_queue_lock.push_back(cpu.ip);
    if ip_queue_lock.len() == 4 {
        ip_queue_lock.pop_front();
    }
    
    match get_address(cpu) {
        0xFE05B => println!("8088 TEST"),
        0xFE0AE => println!("ROS CHECKSUM TEST 1"),
        0xFE0D3 => println!("8237 DMA INIT CHANNEL REGISTER TEST"),
        0xFE14B => println!("BASE 16K R/W STORAGE TEST"),
        0xFE217 => println!("8259 INTERRUPT CONTROLER TEST"),
        0xFE23F => println!("8253 TIMER CHECKOUT"),
        0xFE2AD => println!("INIT AND START CTR CONTROLLER. TEST VIDEO R/W STORAGE"),
        0xFE31E => println!("SETUP VIDEO DATA ON SCREEN FOR VIDEO LINE TEST"),
        0xFE32E => println!("CRT INTERFACE LINES TEST"),
        0xFE382 => println!("EXPANSION I/O BOX TEST"),
        0xFE3C4 => println!("ADDITIONAL R/W STORAGE TEST"),
        0xFE43B => println!("KEYBOARD TEST"),
        0xFE483 => println!("CASSETTE DATA WRAP TEST"),
        0xFE4BC => println!("CHECK FOR OPTIONAL ROM FROM C8000->F4000"),
        0xFE4DC => println!("ROS CHECKSUM 2"),
        0xFE4F1 => println!("DISKETTE ATTACHMENT TEST"),

        0xFE0AD => println!(" - ERROR 1: {:04X}", ip_queue_lock[1]),
        0xFE3BD => println!(" - EXP ERROR: {:04X}", ip_queue_lock[1]),
        0xFE809 => println!(" - ROM ERROR: {:04X}", ip_queue_lock[1]),
        0xFE6BA => println!(" - P_MSG: {:04X}", ip_queue_lock[1]),
        0xFE5CF => println!(" - ERROR BEEP SUBROUTINE: {:04X}", ip_queue_lock[1]),

        0xF6000 => println!("BASIC"),
        _ => {},
    }
}

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
        0xFE3C1 => println!("Test 10"),
        0xFE3F8 => println!("Test 11"),
        0xFE4C7 => println!("Test 12"),
        0xFE521 => println!("Test 13"),
        0xFE55C => println!("Test 14"),

        0xFE0AF => println!("ERROR_1"),
        0xFE6CA => println!("ERROR"),
        0xFE630 => println!("ERR_BEEP"),
        _ => {}
    }
}
