pub mod hardware;
pub mod util;

pub mod debugger;
pub mod screen;

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};
    use serde_json::Value;


    fn read_file(filename: &str) -> Value {
        let mut file = File::open(filename).expect("JSON no existe");
        let mut file_string = String::new();

        file.read_to_string(&mut file_string).expect("Error leyendo el JSON");

        serde_json::from_str(&file_string).unwrap()
    }

    #[test]
    fn test_00() {
        let file = read_file("prueba.json");

        println!("{:#?}", file)
    }
}