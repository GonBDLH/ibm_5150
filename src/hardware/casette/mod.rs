use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
};

use super::{bus::Bus, cpu_8088::CPU};

pub struct CasetteController {
    disks: [Disk; 4],

    sector_buffer: [u8; 512],

    last_ah: u8,
    last_cf: bool,
}

impl Default for CasetteController {
    fn default() -> Self {
        Self {
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

#[derive(Default)]
struct Disk {
    inserted: bool,
    filesize: usize,
    cylinders: usize,
    sectors: usize,
    heads: usize,

    file: Option<File>,
}

impl CasetteController {
    fn get_hdd_count(&self) -> u8 {
        let mut num = 0;

        for i in 2..4 {
            if self.disks[i].inserted {
                num += 1;
            }
        }

        num
    }

    fn read(
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
            for sector_offset in 0..512 {
                let val = self.sector_buffer[sector_offset];
                bus.write_8(segment, offset, val);
                offset += 1;
            }
            sectors_readed += 1;
        }

        cpu.ax.low = sectors_readed as u8;
        cpu.flags.c = false;
        cpu.ax.high = 0;
    }

    fn write(
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

    pub fn insert_disk(&mut self, bus: &mut Bus, num: usize, path: &str) {
        let read_result = fs::File::open(path);

        if read_result.is_err() {
            log::error!("Error reading file");
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
            self.disks[num].cylinders = 80;
            self.disks[num].sectors = 18;
            self.disks[num].heads = 2;
            if self.disks[num].filesize <= 1228800 {
                self.disks[num].sectors = 15
            };
            if self.disks[num].filesize <= 737280 {
                self.disks[num].sectors = 9
            };
            if self.disks[num].filesize <= 368640 {
                self.disks[num].cylinders = 40;
                self.disks[num].sectors = 9;
            }
            if self.disks[num].filesize <= 163840 {
                self.disks[num].cylinders = 40;
                self.disks[num].sectors = 8;
                self.disks[num].heads = 1;
            }
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
