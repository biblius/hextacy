use std::env;

use rand::{thread_rng, Rng};

#[allow(dead_code)]
fn main() {
    let args = env::args().collect::<Vec<String>>();

    let length = &args[1].parse::<u8>().unwrap();

    println!("{length}");

    let mut pw = String::new();

    let mut rng = thread_rng();

    for _ in 0..*length {
        pw.push(char::from_u32(rng.gen_range(33..127) as u32).unwrap());
    }

    std::fs::write(std::path::Path::new("./password"), pw).unwrap();
}
