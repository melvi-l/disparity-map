mod block;
mod ctx;
mod display;

use disparity_map::{decode_png, greet};
use std::fs;

use crate::{block::block_on, display::run};

fn main() {
    println!("{}", greet("Melvil"));
    let mut data = open_png("./front/public/map.png");
    println!(
        "header: [{}]",
        &data[..8]
            .iter()
            .map(|b| format!("0x{:02X}", b))
            .collect::<Vec<String>>()
            .join(", ")
    );
    let decoded = decode_png(&data);
    block_on(run())
}

fn open_png(filename: &str) -> Vec<u8> {
    let data: Vec<u8> = fs::read(&filename).expect(&format!("Unable to open {}", &filename));
    data
}
