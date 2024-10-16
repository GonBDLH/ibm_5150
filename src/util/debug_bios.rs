use lazy_static::lazy_static;

use crate::hardware::cpu_8088::{cpu_utils::get_address, CPU};
use log::{info, warn};
use std::collections::VecDeque;
use std::sync::Mutex;

lazy_static! {
    pub static ref IP_QUEUE: Mutex<VecDeque<u16>> = Mutex::new(VecDeque::with_capacity(4));
    pub static ref ADDR_QUEUE: Mutex<VecDeque<usize>> = Mutex::new(VecDeque::with_capacity(4));
}

pub fn debug_82(cpu: &mut CPU) {
    let mut ip_queue_lock = IP_QUEUE.lock().unwrap();
    ip_queue_lock.push_back(cpu.ip);
    if ip_queue_lock.len() == 10 {
        ip_queue_lock.pop_front();
    }

    let mut addr_queue_lock = ADDR_QUEUE.lock().unwrap();
    addr_queue_lock.push_back(get_address(cpu));
    if addr_queue_lock.len() == 10 {
        addr_queue_lock.pop_front();
    }

    match get_address(cpu) {
        0xFE05B => warn!("8088 TEST"),
        0xFE0AE => warn!("ROS CHECKSUM TEST 1"),
        0xFE0D3 => warn!("8237 DMA INIT CHANNEL REGISTER TEST"),
        0xFE14B => warn!("BASE 16K R/W STORAGE TEST"),
        0xFE217 => warn!("8259 INTERRUPT CONTROLER TEST"),
        0xFE23F => warn!("8253 TIMER CHECKOUT"),
        0xFE2AD => warn!("INIT AND START CTR CONTROLLER. TEST VIDEO R/W STORAGE"),
        0xFE31E => warn!("SETUP VIDEO DATA ON SCREEN FOR VIDEO LINE TEST"),
        0xFE32E => warn!("CRT INTERFACE LINES TEST"),
        0xFE382 => warn!("EXPANSION I/O BOX TEST"),
        0xFE3C4 => warn!("ADDITIONAL R/W STORAGE TEST"),
        0xFE43B => warn!("KEYBOARD TEST"),
        0xFE483 => warn!("CASSETTE DATA WRAP TEST"),
        0xFE4BC => warn!("CHECK FOR OPTIONAL ROM FROM C8000->F4000"),
        0xFE4DC => warn!("ROS CHECKSUM 2"),
        0xFE4F1 => warn!("DISKETTE ATTACHMENT TEST"),

        0xFE0AD => warn!(" - ERROR 1: {:04X?}", ip_queue_lock),
        0xFE3BD => warn!(" - EXP ERROR: {:04X?}", ip_queue_lock),
        0xFE809 => warn!(" - ROM ERROR: {:04X?}", ip_queue_lock),
        0xFE6BA => warn!(" - P_MSG: {:04X?}", ip_queue_lock),
        0xFE5CF => warn!(" - ERROR BEEP SUBROUTINE: {:04X?}", ip_queue_lock),

        0xF6000 => warn!("BASIC"),
        0x07C00 => warn!("DOS"),
        _ => {}
    }
}

pub fn debug(cpu: &mut CPU) {
    match get_address(cpu) {
        0xFE05B => info!("Test 1"),
        0xFE0B0 => info!("Test 2"),
        0xFE0DA => info!("Test 3"),
        0xFE158 => info!("Test 4"),
        0xFE33B => info!("Test 5"),
        0xFE235 => info!("Test 6"),
        0xFE285 => info!("Test 7"),
        0xFE352 => info!("Test 8"),
        0xFE3AF => info!("Test 9"),
        0xFE3C1 => info!("Test 10"),
        0xFE3F8 => info!("Test 11"),
        0xFE4C7 => info!("Test 12"),
        0xFE521 => info!("Test 13"),
        0xFE55C => info!("Test 14"),

        0xFE0AF => info!("ERROR_1"),
        0xFE6CA => info!("ERROR"),
        0xFE630 => info!("ERR_BEEP"),
        _ => {}
    }
}
