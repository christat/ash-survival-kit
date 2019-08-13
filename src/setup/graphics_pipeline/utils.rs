use std::{
    fs::File,
    io::Read,
    path::Path
};

extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

pub fn read_shader(file_path: &Path) -> Vec<u32> {
    let shader_file = File::open(file_path).expect(&format!("Failed to read shader: {}", file_path.display()));
    let shader_bytes = shader_file.bytes().filter_map(|byte| byte.ok()).collect::<Vec<u8>>();
    let shader_raw: Vec<u32> = (0..shader_bytes.len()).step_by(4).fold(vec![], |mut acc, i| {
        acc.push(LittleEndian::read_u32(&shader_bytes[i..]));
        acc
    });
    shader_raw
}