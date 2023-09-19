// SW1 CONF
pub const DD_ENABLE: u8 = 0b00000001;
pub const DD_DISABLE: u8 = 0;
pub const RESERVED: u8 = 0;
pub const MEM_16K: u8 = 0b00000000;
pub const MEM_32K: u8 = 0b00000100;
pub const MEM_48K: u8 = 0b00001000;
pub const MEM_64K: u8 = 0b00001100;
pub const DISPLAY_RESERVED: u8 = 0;
pub const DISPLAY_CGA_40_25: u8 = 0b00010000;
pub const DISPLAY_CGA_80_25: u8 = 0b00100000;
pub const DISPLAY_MDA_80_25: u8 = 0b00110000;
pub const DRIVES_1: u8 = 0b00000000;
pub const DRIVES_2: u8 = 0b01000000;
pub const DRIVES_3: u8 = 0b10000000;
pub const DRIVES_4: u8 = 0b11000000;

// SW2 CONF
pub const HIGH_NIBBLE: u8 = 0b11110000;
pub const PLUS_0: u8 = 0;
pub const PLUS_32: u8 = 0b00000001;
pub const PLUS_64: u8 = 0b00000010;
pub const PLUS_96: u8 = 0b00000011;
pub const PLUS_128: u8 = 0b00000100;
pub const PLUS_160: u8 = 0b00000101;
pub const PLUS_192: u8 = 0b00000110;
