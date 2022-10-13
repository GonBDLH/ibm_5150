use super::ibm_mda::IbmMDA;

// TODO Generalizar
fn print_char(display_adapter: IbmMDA, char_index: usize) {
    for i in 0..14 {
        let char = if i < 8 {
            display_adapter.font[i + char_index * 8]
        } else {
            display_adapter.font[0x800 + (i - 8) + char_index * 8]
        };

        for j in 0..9 {
            let pixel = if j < 8 {
                char & (1 << (7 - j))
            } else {
                if char_index >= 0xC0 && char_index <= 0xDF {
                    char & 1
                } else {
                    0
                }
            };

            // TODO Cambiar prints por correspondiente codigo para poner pixeles en la imagen
            if pixel > 0 {
                print!("{}", std::char::from_u32(0x2588).unwrap());
            } else {
                print!(" ");
            }
        }
        println!();
    }
}
