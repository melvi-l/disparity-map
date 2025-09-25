use std::io::Cursor;

use png::{BitDepth, Decoder};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {name} from Rust+WASM!")
}

pub fn decode_png(data: &[u8]) -> (Vec<u16>, u32, u32) {
    let cursor = Cursor::new(data);
    let decoder = Decoder::new(cursor);
    let mut reader = decoder.read_info().expect("Unable to PNG reader");
    let mut buf = vec![
        0;
        reader
            .output_buffer_size()
            .expect("Unable to get PNG buffer size")
    ];

    let info = reader.next_frame(&mut buf).expect("Unable to decode PNG");

    println!(
        "Disparity map dimension => {} x {}",
        info.width, info.height
    );
    println!("Bit depth => {:?}", info.bit_depth);
    println!("Color type => {:?}", info.color_type);

    if info.bit_depth != BitDepth::Sixteen {
        panic!("Incorrect disparity map: PNG should have a 8bit depth");
    }

    let decoded_data: Vec<u16> = buf[..info.buffer_size()]
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();

    println!("Decoded u16 buffer length => {}", decoded_data.len());

    return (decoded_data, info.width, info.height);
}
