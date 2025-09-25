mod block;
mod ctx;
mod display;

use disparity_map::{decode_png, greet};
use png::{BitDepth, ColorType, Encoder};
use std::{
    fs::{self, File},
    io::BufWriter,
};

use crate::{block::block_on, ctx::decoded_png_u16_to_grayscale_u8, display::run};

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
    let (decoded, width, height) = decode_png(&data);
    // save_png_u8_gray(
    //     "typeshit.png",
    //     &decoded_png_u16_to_grayscale_u8(&decoded),
    //     width,
    //     height,
    // );
    block_on(run(&decoded, width, height))
}

fn open_png(filename: &str) -> Vec<u8> {
    let data: Vec<u8> = fs::read(&filename).expect(&format!("Unable to open {}", &filename));
    data
}

pub fn save_png_u8_gray(path: &str, buffer: &[u8], width: u32, height: u32) {
    let file = File::create(path).expect("Unable to create file");
    let ref mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, width, height);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);

    let mut writer = encoder.write_header().expect("Png header error");
    writer
        .write_image_data(buffer)
        .expect("Unable to write PNG");
}
