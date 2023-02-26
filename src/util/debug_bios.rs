use lazy_static::lazy_static;
use notan::log::info;

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
        0xFFFF0 => info!("POST"),
        0xFE05B => info!("8088 TEST"),
        0xFE0AE => info!("ROS CHECKSUM TEST 1"),
        0xFE0D3 => info!("8237 DMA INIT CHANNEL REGISTER TEST"),
        0xFE14B => info!("BASE 16K R/W STORAGE TEST"),
        0xFE217 => info!("8259 INTERRUPT CONTROLER TEST"),
        0xFE23F => info!("8253 TIMER CHECKOUT"),
        0xFE2AD => info!("INIT AND START CTR CONTROLLER. TEST VIDEO R/W STORAGE"),
        0xFE31E => info!("SETUP VIDEO DATA ON SCREEN FOR VIDEO LINE TEST"),
        0xFE32E => info!("CRT INTERFACE LINES TEST"),
        0xFE382 => info!("EXPANSION I/O BOX TEST"),
        0xFE3C4 => info!("ADDITIONAL R/W STORAGE TEST"),
        0xFE43B => info!("KEYBOARD TEST"),
        0xFE483 => info!("CASSETTE DATA WRAP TEST"),
        0xFE4BC => info!("CHECK FOR OPTIONAL ROM FROM C8000->F4000"),
        0xFE4DC => info!("ROS CHECKSUM 2"),
        0xFE4F1 => info!("DISKETTE ATTACHMENT TEST"),

        0xFE0AD => info!(" - ERROR 1: {:04X}", ip_queue_lock[1]),
        0xFE3BD => info!(" - EXP ERROR: {:04X}", ip_queue_lock[1]),
        0xFE809 => info!(" - ROM ERROR: {:04X}", ip_queue_lock[1]),
        0xFE6BA => info!(" - P_MSG: {:04X}", ip_queue_lock[1]),
        0xFE5CF => info!(" - ERROR BEEP SUBROUTINE: {:04X}", ip_queue_lock[1]),

        0xF6000 => info!("BASIC"),
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
