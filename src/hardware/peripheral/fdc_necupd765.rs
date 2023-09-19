use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
};

use crate::hardware::{bus::Bus, cpu_8088::CPU};

use super::Peripheral;

pub struct FloppyDiskController {
    dor_register: u8,
    status_register: u8,
    data_register: Vec<u8>,

    disks: [Disk; 4],

    pub sector_buffer: [u8; 512],

    pub last_ah: u8,
    pub last_cf: bool,
}

impl Peripheral for FloppyDiskController {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x3F4 => self.status_register as u16,
            // TODO DATA REGISTER
            0x3F5 => self.data_register[0] as u16,
            _ => 0,
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x3F2 => self.dor_register = val as u8,
            // TODO DATA REGISTER
            0x3F5 => self.data_register[0] = val as u8,
            _ => {}
        }
    }
}

impl Default for FloppyDiskController {
    fn default() -> Self {
        Self {
            dor_register: 0,
            status_register: 0,
            data_register: Vec::new(),

            disks: [
                Disk::default(),
                Disk::default(),
                Disk::default(),
                Disk::default(),
            ],
            sector_buffer: [0x00; 512],
            last_ah: 0,
            last_cf: false,
        }
    }
}

impl FloppyDiskController {
    fn is_enabled(&self) -> bool {
        self.dor_register & 0b00000100 != 0
    }

    fn get_selected_drive(&self) -> usize {
        (self.dor_register & 0b00000011) as usize
    }

    fn is_dma_irq_mode(&self) -> bool {
        self.dor_register & 0b00001000 != 0
    }

    fn is_motor_started(&self, drive: u8) -> bool {
        self.dor_register >> (4 + drive) != 0
    }
}

#[derive(Default)]
struct Disk {
    pub inserted: bool,
    filesize: usize,
    cylinders: usize,
    sectors: usize,
    heads: usize,

    file: Option<File>,
}

impl FloppyDiskController {
    pub fn get_hdd_count(&self) -> u8 {
        let mut num = 0;

        for i in 2..4 {
            if self.disks[i].inserted {
                num += 1;
            }
        }

        num
    }

    pub fn read(
        &mut self,
        cpu: &mut CPU,
        bus: &mut Bus,
        drive_number: usize,
        segment: u16,
        mut offset: u16,
        head: usize,
        cyl: usize,
        sector: usize,
        num_sectors: usize,
    ) {
        let lba = (cyl * self.disks[drive_number].heads + head) * self.disks[drive_number].sectors
            + (sector - 1);
        let fileoffset = lba * 512;

        if fileoffset > self.disks[drive_number].filesize {
            return;
        }

        let mut sectors_readed = 0;

        let mut file_ref = self.disks[drive_number].file.as_ref().unwrap();

        file_ref.seek(SeekFrom::Start(fileoffset as u64)).unwrap();
        while sectors_readed < num_sectors {
            if file_ref.read(&mut self.sector_buffer).unwrap() < 512 {
                break;
            }

            let dir = (segment as usize * 0x10 + offset as usize) % 0x100000;
            bus.memory[dir..(dir + 512)].copy_from_slice(&self.sector_buffer);
            offset += 512;
            sectors_readed += 1;
        }

        cpu.ax.low = sectors_readed as u8;
        cpu.flags.c = false;
        cpu.ax.high = 0;
    }

    pub fn write(
        &mut self,
        cpu: &mut CPU,
        bus: &mut Bus,
        drive_number: usize,
        segment: u16,
        mut offset: u16,
        head: usize,
        cyl: usize,
        sector: usize,
        num_sectors: usize,
    ) {
        let lba = (cyl * self.disks[drive_number].heads + head) * self.disks[drive_number].sectors
            + sector
            - 1;
        let fileoffset = lba * 512;

        if fileoffset > self.disks[drive_number].filesize {
            return;
        }

        let mut file_ref = self.disks[drive_number].file.as_ref().unwrap();

        file_ref.seek(SeekFrom::Start(fileoffset as u64)).unwrap();
        for _ in 0..num_sectors {
            for sector_offset in 0..512 {
                self.sector_buffer[sector_offset] = bus.read_8(segment, offset);
                offset += 1;
            }
            file_ref.write_all(&self.sector_buffer).unwrap();
        }

        cpu.ax.low = num_sectors as u8;
        cpu.flags.c = false;
        cpu.ax.high = 0;
    }

    pub fn eject_disk(&mut self, num: usize) {
        self.disks[num].inserted = false;
    }

    fn config_disk(&mut self, num: usize, spt: usize, tps: usize, hpc: usize) {
        self.disks[num].sectors = spt;
        self.disks[num].cylinders = tps;
        self.disks[num].heads = hpc;
    }

    // RETURNS FALSE IF FLOPPY OR ERROR OR TRUE IF HDD
    pub fn insert_disk(&mut self, bus: &mut Bus, num: usize, path: &str) {
        let read_result = fs::File::open(path);

        if read_result.is_err() {
            println!("Error reading file");
            return;
        }

        let file = read_result.unwrap();

        self.disks[num].inserted = true;
        self.disks[num].filesize = file.metadata().unwrap().len() as usize;
        self.disks[num].file = Some(file);

        if num >= 2 {
            // IT'S A HDD
            self.disks[num].sectors = 63;
            self.disks[num].heads = 16;
            self.disks[num].cylinders =
                self.disks[num].filesize / (self.disks[num].sectors * self.disks[num].heads * 512);
            bus.write_8(0x40, 0x75, self.get_hdd_count())
        } else {
            // IT'S A FLOPPY
            let kilobytes = self.disks[num].filesize / 1024;

            match kilobytes {
                160 => self.config_disk(num, 8, 40, 1),
                320 => self.config_disk(num, 8, 40, 2),
                180 => self.config_disk(num, 9, 40, 1),
                360 => self.config_disk(num, 9, 40, 2),
                // 320 => self.config_disk(num, 8, 80, 1),
                // 640 => self.config_disk(num, 8, 80, 2),
                720 => self.config_disk(num, 9, 80, 2),
                1200 => self.config_disk(num, 15, 80, 2),
                1440 => self.config_disk(num, 18, 80, 2),
                _ => panic!("WRONG DISK"), // TODO MEJORAR
            };
        }
    }

    pub fn int19(&mut self, cpu: &mut CPU, bus: &mut Bus) {
        bus.write_8(0x40, 0x75, self.get_hdd_count());

        bus.write_8(0, 0x7C00, 0xFB);
        bus.write_8(0, 0x7C01, 0xEB);
        bus.write_8(0, 0x7C02, 0xFE);

        cpu.dx.low = 0;

        // CYL SECT HEAD SECTCOUNT
        self.read(cpu, bus, 0, 0, 0x7C00, 0, 0, 1, 1);
        cpu.cs = 0;
        cpu.ip = 0x7C00;
    }

    pub fn int13(&mut self, cpu: &mut CPU, bus: &mut Bus) {
        let drive_number = cpu.dx.low as usize;
        let head_number = cpu.dx.high as usize;
        let track_number = cpu.cx.high as usize;
        let sector_number = cpu.cx.low as usize;
        let number_of_sectors = cpu.ax.low as usize;
        let segment = cpu.es;
        let offset = cpu.bx.get_x();

        if drive_number > 3 {
            cpu.ax.high = 1;
            cpu.flags.c = true;
            return;
        }

        match cpu.ax.high {
            0 => {
                // RESET DISKETTE SYSTEM
                cpu.ax.high = 0;
                cpu.flags.c = false;
            }
            1 => {
                // READ STATUS INTO AL
                cpu.ax.high = self.last_ah;
                cpu.flags.c = self.last_cf;
            }
            2 => {
                // READ SECTORS INTO MEMORY
                if self.disks[drive_number].inserted {
                    // TODO READ
                    self.read(
                        cpu,
                        bus,
                        drive_number,
                        segment,
                        offset,
                        head_number,
                        track_number + (sector_number / 64) * 256,
                        sector_number & 63,
                        number_of_sectors,
                    );
                    cpu.ax.high = 0;
                    cpu.flags.c = false;
                } else {
                    cpu.ax.high = 1;
                    cpu.flags.c = true;
                }
            }
            3 => {
                // WRITE SECTORS FROM MEMORY
                if self.disks[drive_number].inserted {
                    self.write(
                        cpu,
                        bus,
                        drive_number,
                        segment,
                        offset,
                        head_number,
                        track_number + (sector_number / 64) * 256,
                        sector_number & 63,
                        number_of_sectors,
                    );
                    cpu.ax.high = 0;
                    cpu.flags.c = false;
                } else {
                    cpu.ax.high = 1;
                    cpu.flags.c = true;
                }
            }
            4 => {
                // VERIFY SECTORS
            }
            5 => {
                // FORMAT TRACK
            }

            _ => {
                cpu.flags.c = true;
            }
        }

        self.last_ah = cpu.ax.high;
        self.last_cf = cpu.flags.c;
        if cpu.bx.low & 0x80 > 0 {
            bus.write_8(0x40, 0x74, cpu.ax.high);
        }
    }
}
