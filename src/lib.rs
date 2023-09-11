pub mod hardware;
pub mod util;

pub mod debugger;
pub mod screen;

use std::io::Read;

use assert_hex::assert_eq_hex;
use egui::epaint::ahash::HashMap;
use flate2::read::GzDecoder;
#[allow(unused_imports)]
use hardware::{
    cpu_8088::instr_utils::{Opcode, RepetitionPrefix},
    sys::System,
};
#[allow(unused_imports)]
use ntest::timeout;
use serde::Deserialize;

use crate::hardware::cpu_8088::cpu_utils::to_2u8;

#[allow(dead_code)]
fn open_json(path: &str) -> Vec<Instr> {
    let buff = std::fs::read(path).unwrap();
    let mut decoder = GzDecoder::new(&buff[..]);

    let mut str_file_gz = String::new();

    decoder
        .read_to_string(&mut str_file_gz)
        .expect("Error descomprimiendo");

    serde_json::from_str(&str_file_gz).unwrap()
}

#[allow(dead_code)]
fn open_metadata() -> HashMap<String, Entry> {
    let buff = std::fs::read("roms/tests/8088/v1/8088.json").unwrap();

    serde_json::from_slice(&buff).unwrap()
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Entry {
    Normal {
        status: String,
        flags: Option<String>,
        #[serde(rename = "flags-mask")]
        flags_mask: Option<u16>
    },
    Nested {
        reg: HashMap<String, Entry>
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct State {
    regs: HashMap<String, u16>,
    ram: Vec<(usize, u8)>,
    queue: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Instr {
    name: String,
    bytes: Vec<u8>,
    #[serde(rename = "initial")]
    initial_state: State,
    #[serde(rename = "final")]
    final_state: State,
}

#[allow(dead_code)]
fn set_state(instr: &Instr, sys: &mut System) {
    sys.cpu
        .ax
        .set_x(*instr.initial_state.regs.get("ax").unwrap());
    sys.cpu
        .bx
        .set_x(*instr.initial_state.regs.get("bx").unwrap());
    sys.cpu
        .cx
        .set_x(*instr.initial_state.regs.get("cx").unwrap());
    sys.cpu
        .dx
        .set_x(*instr.initial_state.regs.get("dx").unwrap());
    sys.cpu.cs = *instr.initial_state.regs.get("cs").unwrap();
    sys.cpu.ss = *instr.initial_state.regs.get("ss").unwrap();
    sys.cpu.ds = *instr.initial_state.regs.get("ds").unwrap();
    sys.cpu.es = *instr.initial_state.regs.get("es").unwrap();
    sys.cpu.sp = *instr.initial_state.regs.get("sp").unwrap();
    sys.cpu.bp = *instr.initial_state.regs.get("bp").unwrap();
    sys.cpu.si = *instr.initial_state.regs.get("si").unwrap();
    sys.cpu.di = *instr.initial_state.regs.get("di").unwrap();
    sys.cpu.ip = *instr.initial_state.regs.get("ip").unwrap();
    sys.cpu
        .flags
        .set_flags(*instr.initial_state.regs.get("flags").unwrap());

    for i in &instr.initial_state.ram {
        sys.bus.memory[i.0] = i.1
    }
}

#[allow(dead_code)]
fn check_state(instr: &Instr, sys: &mut System, metadata: &HashMap<String, Entry>, file_name: &str) {
    assert_eq_hex!(
        sys.cpu.ax.get_x(),
        *instr.final_state.regs.get("ax").unwrap(),
        "ax {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.bx.get_x(),
        *instr.final_state.regs.get("bx").unwrap(),
        "bx {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.cx.get_x(),
        *instr.final_state.regs.get("cx").unwrap(),
        "cx {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.dx.get_x(),
        *instr.final_state.regs.get("dx").unwrap(),
        "dx {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.cs,
        *instr.final_state.regs.get("cs").unwrap(),
        "cs {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.ss,
        *instr.final_state.regs.get("ss").unwrap(),
        "ss {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.ds,
        *instr.final_state.regs.get("ds").unwrap(),
        "ds {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.es,
        *instr.final_state.regs.get("es").unwrap(),
        "es {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.sp,
        *instr.final_state.regs.get("sp").unwrap(),
        "sp {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.bp,
        *instr.final_state.regs.get("bp").unwrap(),
        "bp {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.si,
        *instr.final_state.regs.get("si").unwrap(),
        "si {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.di,
        *instr.final_state.regs.get("di").unwrap(),
        "di {} {:#?}",
        instr.name,
        instr
    );
    assert_eq_hex!(
        sys.cpu.ip,
        *instr.final_state.regs.get("ip").unwrap(),
        "ip {} {:#?}",
        instr.name,
        instr
    );

    let mut parts = Vec::new();
    for part in file_name.split(".") {
        parts.push(part);
    }

    let flags_mask = if let Some(e) = metadata.get(parts[0]) {
        match e {
            Entry::Nested { reg } => {
                let option = *parts.get(1).unwrap_or(&"0");
                if let Some(m) = reg.get(option) {
                    match m {
                        Entry::Nested { reg: _ } => None,
                        Entry::Normal { status: _, flags: _, flags_mask } => flags_mask.clone()
                    }
                } else {
                    None
                }
            },
            Entry::Normal { status: _, flags: _, flags_mask } => {
                flags_mask.clone()
            }
        }
    } else {
        None
    };

    let flags_mask = flags_mask.unwrap_or(0xFFFF);

    assert_eq!(
        sys.cpu.flags.get_flags() & flags_mask,
        *instr.final_state.regs.get("flags").unwrap() & flags_mask,
        "flags {} {:#?}",
        instr.name,
        instr
    );

    for i in &instr.final_state.ram {
        // println!("{} {}", sys.bus.memory[i.0], i.1);
        let (mask_low, mask_high) = to_2u8(flags_mask);
        let stack = (sys.cpu.ss as usize * 0x10 + sys.cpu.sp as usize) & 0xFFFFF;

        if i.0 == stack + 5 {
            assert_eq_hex!(
                sys.bus.memory[i.0] & mask_high,
                i.1 & mask_high,
                "stack flags high {:05X} {} {:#?}",
                i.0,
                instr.name,
                instr
            )
        } else if i.0 == stack + 4 {
            assert_eq_hex!(
                sys.bus.memory[i.0] & mask_low,
                i.1 & mask_low,
                "stack flags low {:05X} {} {:#?}",
                i.0,
                instr.name,
                instr
            )
        } else {
            assert_eq_hex!(
                sys.bus.memory[i.0],
                i.1,
                "mem {:05X} {} {:#?}",
                i.0,
                instr.name,
                instr
            )
        }
    }
}

macro_rules! create_test {
    ( $name: ident, $file_name: literal ) => {
        #[test]
        #[timeout(150000)]
        fn $name() {
            let v = open_json(&(String::from("roms/tests/8088/v1/") + $file_name + ".json.gz"));
            let metadata = open_metadata();

            for i in v {
                let mut sys = System::new();
                set_state(&i, &mut sys);

                // if i.name == String::from("add byte cs:[bp+si+E7h], bh") {
                //     println!("ENTRO")
                // }

                loop {
                    sys.step(&mut 0);
                    if sys.cpu.instr.repetition_prefix != RepetitionPrefix::None
                        && !sys.cpu.to_decode
                    {
                        continue;
                    }

                    break;
                }

                check_state(&i, &mut sys, &metadata, $file_name);
            }
        }
    };
}

create_test!(test_00, "00");
create_test!(test_01, "01");
create_test!(test_02, "02");
create_test!(test_03, "03");
create_test!(test_04, "04");
create_test!(test_05, "05");
create_test!(test_06, "06");
create_test!(test_07, "07");
create_test!(test_08, "08");
create_test!(test_09, "09");
create_test!(test_0a, "0A");
create_test!(test_0b, "0B");
create_test!(test_0c, "0C");
create_test!(test_0d, "0D");
create_test!(test_0e, "0E");
create_test!(test_0f, "0F");
create_test!(test_10, "10");
create_test!(test_11, "11");
create_test!(test_12, "12");
create_test!(test_13, "13");
create_test!(test_14, "14");
create_test!(test_15, "15");
create_test!(test_16, "16");
create_test!(test_17, "17");
create_test!(test_18, "18");
create_test!(test_19, "19");
create_test!(test_1a, "1A");
create_test!(test_1b, "1B");
create_test!(test_1c, "1C");
create_test!(test_1d, "1D");
create_test!(test_1e, "1E");
create_test!(test_1f, "1F");
create_test!(test_20, "20");
create_test!(test_21, "21");
create_test!(test_22, "22");
create_test!(test_23, "23");
create_test!(test_24, "24");
create_test!(test_25, "25");
create_test!(test_27, "27");
create_test!(test_28, "28");
create_test!(test_29, "29");
create_test!(test_2a, "2A");
create_test!(test_2b, "2B");
create_test!(test_2c, "2C");
create_test!(test_2d, "2D");
create_test!(test_2f, "2F");
create_test!(test_30, "30");
create_test!(test_31, "31");
create_test!(test_32, "32");
create_test!(test_33, "33");
create_test!(test_34, "34");
create_test!(test_35, "35");
create_test!(test_37, "37");
create_test!(test_38, "38");
create_test!(test_39, "39");
create_test!(test_3a, "3A");
create_test!(test_3b, "3B");
create_test!(test_3c, "3C");
create_test!(test_3d, "3D");
create_test!(test_3f, "3F");
create_test!(test_40, "40");
create_test!(test_41, "41");
create_test!(test_42, "42");
create_test!(test_43, "43");
create_test!(test_44, "44");
create_test!(test_45, "45");
create_test!(test_46, "46");
create_test!(test_47, "47");
create_test!(test_48, "48");
create_test!(test_49, "49");
create_test!(test_4a, "4A");
create_test!(test_4b, "4B");
create_test!(test_4c, "4C");
create_test!(test_4d, "4D");
create_test!(test_4e, "4E");
create_test!(test_4f, "4F");
create_test!(test_50, "50");
create_test!(test_51, "51");
create_test!(test_52, "52");
create_test!(test_53, "53");
create_test!(test_54, "54");
create_test!(test_55, "55");
create_test!(test_56, "56");
create_test!(test_57, "57");
create_test!(test_58, "58");
create_test!(test_59, "59");
create_test!(test_5a, "5A");
create_test!(test_5b, "5B");
create_test!(test_5c, "5C");
create_test!(test_5d, "5D");
create_test!(test_5e, "5E");
create_test!(test_5f, "5F");
create_test!(test_60, "60");
create_test!(test_61, "61");
create_test!(test_62, "62");
create_test!(test_63, "63");
create_test!(test_64, "64");
create_test!(test_65, "65");
create_test!(test_66, "66");
create_test!(test_67, "67");
create_test!(test_68, "68");
create_test!(test_69, "69");
create_test!(test_6a, "6A");
create_test!(test_6b, "6B");
create_test!(test_6c, "6C");
create_test!(test_6d, "6D");
create_test!(test_6e, "6E");
create_test!(test_6f, "6F");
create_test!(test_70, "70");
create_test!(test_71, "71");
create_test!(test_72, "72");
create_test!(test_73, "73");
create_test!(test_74, "74");
create_test!(test_75, "75");
create_test!(test_76, "76");
create_test!(test_77, "77");
create_test!(test_78, "78");
create_test!(test_79, "79");
create_test!(test_7a, "7A");
create_test!(test_7b, "7B");
create_test!(test_7c, "7C");
create_test!(test_7d, "7D");
create_test!(test_7e, "7E");
create_test!(test_7f, "7F");
create_test!(test_80_0, "80.0");
create_test!(test_80_1, "80.1");
create_test!(test_80_2, "80.2");
create_test!(test_80_3, "80.3");
create_test!(test_80_4, "80.4");
create_test!(test_80_5, "80.5");
create_test!(test_80_6, "80.6");
create_test!(test_80_7, "80.7");
create_test!(test_81_0, "81.0");
create_test!(test_81_1, "81.1");
create_test!(test_81_2, "81.2");
create_test!(test_81_3, "81.3");
create_test!(test_81_4, "81.4");
create_test!(test_81_5, "81.5");
create_test!(test_81_6, "81.6");
create_test!(test_81_7, "81.7");
create_test!(test_82_0, "82.0");
create_test!(test_82_1, "82.1");
create_test!(test_82_2, "82.2");
create_test!(test_82_3, "82.3");
create_test!(test_82_4, "82.4");
create_test!(test_82_5, "82.5");
create_test!(test_82_6, "82.6");
create_test!(test_82_7, "82.7");
create_test!(test_83_0, "83.0");
create_test!(test_83_1, "83.1");
create_test!(test_83_2, "83.2");
create_test!(test_83_3, "83.3");
create_test!(test_83_4, "83.4");
create_test!(test_83_5, "83.5");
create_test!(test_83_6, "83.6");
create_test!(test_83_7, "83.7");
create_test!(test_84, "84");
create_test!(test_85, "85");
create_test!(test_86, "86");
create_test!(test_87, "87");
create_test!(test_88, "88");
create_test!(test_89, "89");
create_test!(test_8a, "8A");
create_test!(test_8b, "8B");
create_test!(test_8c, "8C");
create_test!(test_8d, "8D");
create_test!(test_8e, "8E");
create_test!(test_8f, "8F");
create_test!(test_90, "90");
create_test!(test_91, "91");
create_test!(test_92, "92");
create_test!(test_93, "93");
create_test!(test_94, "94");
create_test!(test_95, "95");
create_test!(test_96, "96");
create_test!(test_97, "97");
create_test!(test_98, "98");
create_test!(test_99, "99");
create_test!(test_9a, "9A");
create_test!(test_9c, "9C");
create_test!(test_9d, "9D");
create_test!(test_9e, "9E");
create_test!(test_9f, "9F");
create_test!(test_a0, "A0");
create_test!(test_a1, "A1");
create_test!(test_a2, "A2");
create_test!(test_a3, "A3");
create_test!(test_a4, "A4");
create_test!(test_a5, "A5");
create_test!(test_a6, "A6");
create_test!(test_a7, "A7");
create_test!(test_a8, "A8");
create_test!(test_a9, "A9");
create_test!(test_aa, "AA");
create_test!(test_ab, "AB");
create_test!(test_ac, "AC");
create_test!(test_ad, "AD");
create_test!(test_ae, "AE");
create_test!(test_af, "AF");
create_test!(test_b0, "B0");
create_test!(test_b1, "B1");
create_test!(test_b2, "B2");
create_test!(test_b3, "B3");
create_test!(test_b4, "B4");
create_test!(test_b5, "B5");
create_test!(test_b6, "B6");
create_test!(test_b7, "B7");
create_test!(test_b8, "B8");
create_test!(test_b9, "B9");
create_test!(test_ba, "BA");
create_test!(test_bb, "BB");
create_test!(test_bc, "BC");
create_test!(test_bd, "BD");
create_test!(test_be, "BE");
create_test!(test_bf, "BF");
create_test!(test_c0, "C0");
create_test!(test_c1, "C1");
create_test!(test_c2, "C2");
create_test!(test_c3, "C3");
create_test!(test_c4, "C4");
create_test!(test_c5, "C5");
create_test!(test_c6, "C6");
create_test!(test_c7, "C7");
create_test!(test_c8, "C8");
create_test!(test_c9, "C9");
create_test!(test_ca, "CA");
create_test!(test_cb, "CB");
create_test!(test_cc, "CC");
create_test!(test_cd, "CD");
create_test!(test_ce, "CE");
create_test!(test_cf, "CF");
create_test!(test_d0_0, "D0.0");
create_test!(test_d0_1, "D0.1");
create_test!(test_d0_2, "D0.2");
create_test!(test_d0_3, "D0.3");
create_test!(test_d0_4, "D0.4");
create_test!(test_d0_5, "D0.5");
create_test!(test_d0_6, "D0.6");
create_test!(test_d0_7, "D0.7");
create_test!(test_d1_0, "D1.0");
create_test!(test_d1_1, "D1.1");
create_test!(test_d1_2, "D1.2");
create_test!(test_d1_3, "D1.3");
create_test!(test_d1_4, "D1.4");
create_test!(test_d1_5, "D1.5");
create_test!(test_d1_6, "D1.6");
create_test!(test_d1_7, "D1.7");
create_test!(test_d2_0, "D2.0");
create_test!(test_d2_1, "D2.1");
create_test!(test_d2_2, "D2.2");
create_test!(test_d2_3, "D2.3");
create_test!(test_d2_4, "D2.4");
create_test!(test_d2_5, "D2.5");
create_test!(test_d2_6, "D2.6");
create_test!(test_d2_7, "D2.7");
create_test!(test_d3_0, "D3.0");
create_test!(test_d3_1, "D3.1");
create_test!(test_d3_2, "D3.2");
create_test!(test_d3_3, "D3.3");
create_test!(test_d3_4, "D3.4");
create_test!(test_d3_5, "D3.5");
create_test!(test_d3_6, "D3.6");
create_test!(test_d3_7, "D3.7");
create_test!(test_d4, "D4");
create_test!(test_d5, "D5");
create_test!(test_d6, "D6");
create_test!(test_d7, "D7");
create_test!(test_d8, "D8");
create_test!(test_d9, "D9");
create_test!(test_da, "DA");
create_test!(test_db, "DB");
create_test!(test_dc, "DC");
create_test!(test_dd, "DD");
create_test!(test_de, "DE");
create_test!(test_df, "DF");
create_test!(test_e0, "E0");
create_test!(test_e1, "E1");
create_test!(test_e2, "E2");
create_test!(test_e3, "E3");
create_test!(test_e4, "E4");
create_test!(test_e5, "E5");
create_test!(test_e6, "E6");
create_test!(test_e7, "E7");
create_test!(test_e8, "E8");
create_test!(test_e9, "E9");
create_test!(test_ea, "EA");
create_test!(test_eb, "EB");
create_test!(test_ec, "EC");
create_test!(test_ed, "ED");
create_test!(test_ee, "EE");
create_test!(test_ef, "EF");
create_test!(test_f5, "F5");
create_test!(test_f6_0, "F6.0");
create_test!(test_f6_1, "F6.1");
create_test!(test_f6_2, "F6.2");
create_test!(test_f6_3, "F6.3");
create_test!(test_f6_4, "F6.4");
create_test!(test_f6_5, "F6.5");
create_test!(test_f6_6, "F6.6");
create_test!(test_f6_7, "F6.7");
create_test!(test_f7_0, "F7.0");
create_test!(test_f7_1, "F7.1");
create_test!(test_f7_2, "F7.2");
create_test!(test_f7_3, "F7.3");
create_test!(test_f7_4, "F7.4");
create_test!(test_f7_5, "F7.5");
create_test!(test_f7_6, "F7.6");
create_test!(test_f7_7, "F7.7");
create_test!(test_f8, "F8");
create_test!(test_f9, "F9");
create_test!(test_fa, "FA");
create_test!(test_fb, "FB");
create_test!(test_fc, "FC");
create_test!(test_fd, "FD");
create_test!(test_fe_0, "FE.0");
create_test!(test_fe_1, "FE.1");
create_test!(test_ff_0, "FF.0");
create_test!(test_ff_1, "FF.1");
create_test!(test_ff_2, "FF.2");
create_test!(test_ff_3, "FF.3");
create_test!(test_ff_4, "FF.4");
create_test!(test_ff_5, "FF.5");
create_test!(test_ff_6, "FF.6");
create_test!(test_ff_7, "FF.7");
