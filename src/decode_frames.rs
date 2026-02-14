use std::fs::{read,read_dir};
use std::path::{PathBuf,Path};
use std::env;
use std::time::Instant;
use minifb::{Key, Window, WindowOptions};
use colored::Colorize;

fn pos_hash(brightness: u8) -> usize {
    let hash_index = brightness >> 2;
    hash_index as usize
}

// Return buffer, width, height, and file size
fn decode(path: &Path) -> Option<(Vec<u32>, u32, u32, usize)> {
    // Raw image binary
    let raw_binary: Vec<u8> = read(path).ok()?;
    let file_size = raw_binary.len();

    // Reading header
    let identifiers: String = raw_binary[0..4].iter().map(|x| *x as char).collect();
    
    if identifiers != "fmif" {
        println!("This is not a valid FMI image");
        return None;
    }

    let mut width: u32 = 0;
    for byte in 4..8 {
        width = (width << 8) | (raw_binary[byte] as u32);
    }
    
    let mut height: u32 = 0;
    for byte in 8..12 {
        height = (height << 8) | (raw_binary[byte] as u32);
    }
     
    let mut buffer: Vec<u32> = Vec::new();

    let start = Instant::now();
    // Decode starting at the end of the header
    let mut i = 12;
    let mut tag: u8;
    let mut brightness: u32;
    let mut prev_brightness: u8 = 0;

    // We have to rebuild the gray index of hashes as we decode
    let mut gray_index: [Option<u8>; 64] = [None; 64];

    // While loop goes until the last 8 bytes which it ignores as they are the footer
    while i < (raw_binary.len() - 8) {
        tag = raw_binary[i] >> 6;

        match tag {
            // Run byte decoding
            0b10 => {
                brightness = prev_brightness as u32;
                let brightness_rgb: u32 = brightness << 16 | brightness << 8 | brightness;
                let length: u16 = ((0x3f & raw_binary[i]) as u16) << 8 | (raw_binary[i + 1] as u16);            
                for _ in 0..length {
                    buffer.push(brightness_rgb);    
                }
                prev_brightness = brightness as u8;
                i += 2;
            }

            // Full grayscale tag decoding
            0b11 => {
                brightness = raw_binary[i + 1] as u32;
                let brightness_rgb: u32 = brightness << 16 | brightness << 8 | brightness;
                buffer.push(brightness_rgb);
                prev_brightness = brightness as u8;

                let hash_index = pos_hash(brightness as u8);
                gray_index[hash_index] = Some(brightness as u8);

                i += 2;                
            }

            // Hash decoding
            0b00 => {
                let hash_index = raw_binary[i] as usize;
                brightness = gray_index[hash_index].unwrap() as u32;
                let brightness_rgb: u32 = brightness << 16 | brightness << 8 | brightness;
                buffer.push(brightness_rgb);
                prev_brightness = brightness as u8;
                i += 1;
            }

            // Diff decoding
            0b01 => {
                let difference: u8 = 0b00111111 & raw_binary[i];
                let signed_difference = (difference as i8) - 32;
                let brightness = ((prev_brightness as i8).wrapping_add(signed_difference) as u8) as u32;
                let brightness_rgb: u32 = brightness << 16 | brightness << 8 | brightness;
                prev_brightness = brightness as u8;
                buffer.push(brightness_rgb);
                i += 1;
            }

            // Catch all for unused tags
            _ => {}
        }
    }

    println!("decoded in {:?}...", start.elapsed());
    Some((buffer, width, height, file_size))
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let directory_path:String;

    if args.len() == 1 {
        directory_path = ".".to_string()
    }else{
        directory_path = args[1].clone();
    }

    let directory = read_dir(directory_path).unwrap();
    let mut paths: Vec<PathBuf> = Vec::new();

    for entry in directory {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "fmi" {
                paths.push(path);
            }
        }
    }


    // Decode the image
    let (buffer, width, height, file_size) = match decode(&paths[0]) {
        Some(data) => data,
        None => {
            println!("Failed to decode image");
            return;
        }
    };

    let monitor_height = 1000;
    let image_scale = (monitor_height as f32) / (height as f32);

    let mut window = Window::new(
        "FMI Viewer",
        ((width as f32) * image_scale) as usize,
        ((height as f32) * image_scale) as usize,
        WindowOptions {
            ..WindowOptions::default()
        }
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    paths.sort();

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);
 
    let mut current_frame = 0;

    while window.is_open() && !window.is_key_down(Key::Escape){
        let (buffer, width, height, file_size) = match decode(&paths[current_frame]) {
            Some(data) => data,
            None => {
                println!("Failed to decode image");
                return;
            }
        };

        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .unwrap();
        if current_frame == paths.len()-1{
            current_frame = 0;
        }else{
            current_frame += 1;
        }
    }
}
