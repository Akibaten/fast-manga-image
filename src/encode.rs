use std::fs::write;
use std::path::Path;
use std::env;
use std::time::Instant;
use image::ImageReader;
use colored::Colorize;

fn grayscale(red:u8,green:u8,blue:u8) -> u8 {
    let r = red as u32;
    let g = green as u32;
    let b = blue as u32;
    let brightness = (r*30 + g*59 + b*11) / 100;
    brightness as u8
}

fn pos_hash(brightness: u8) -> usize{
    // let hash_index = brightness % 64;
    let hash_index = brightness >> 2;
    hash_index as usize
}

fn fmi_op_index(data_stream: &mut Vec<u8>, index:usize) {
    data_stream.push(index as u8 & 0x3f); 
}

fn fmi_op_difference(data_stream: &mut Vec<u8>, diff_byte:u8) {
    data_stream.push(diff_byte);
}

fn fmi_op_gray(data_stream: &mut Vec<u8>, brightness:u8){
    data_stream.push(0xFF);
    data_stream.push(brightness);
}

fn fmi_op_run(data_stream: &mut Vec<u8>, run:u16){
    let left_byte = ((run >> 8) as u8)| 0b10000000;
    let right_byte = run as u8;
    data_stream.push(left_byte);
    data_stream.push(right_byte);
}

fn write_fmi_header(data_stream: &mut Vec<u8>, width:u32, height:u32) -> (){
    data_stream.extend_from_slice(b"fmif");
    data_stream.extend_from_slice(&width.to_be_bytes());
    data_stream.extend_from_slice(&height.to_be_bytes());
}

fn main() {
    let start = Instant::now();
    let args: Vec<String> = env::args().collect();
    let mut data_stream = Vec::<u8>::new();
    let input_path: &Path;
    let output_path: &Path;

    //error handling for image input and output
    if args.len() == 1{
        println!("No valid image file provided \nPlease provide a valid image path");
        return;
    }else{
        input_path = Path::new(&args[1]);
    }

    //error handling for image input
    if args.len() == 2{
        output_path = Path::new(&args[1]);
    }else{
        output_path = Path::new(&args[2]);
    }

    //image input to be encoded
    let img = ImageReader::open(input_path)
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8();

    let pixels: Vec<u8> = img.as_raw().to_vec();
    let width = img.width() as usize;
    let height = img.height() as usize;
 
    let mut i = 0;
    let mut prev_brightness: u8 = 0;
    let mut gray_index: [Option<u8>;64] = [None; 64];

    //keeps track of total number of each type of byte
    let mut index_bytes = 0;
    let mut diff_bytes = 0;
    let mut gray_chunks = 0;
    let mut run_chunks = 0;
    
    //encode

    //first makes a header for the file
    write_fmi_header(&mut data_stream, width as u32, height as u32);

    while i < (pixels.len()/3){
        let brightness = grayscale(
            pixels[i*3],
            pixels[i*3+1],
            pixels[i*3+2]);
        let hash_index = pos_hash(brightness);
        let mut difference = (brightness as i8).wrapping_sub(prev_brightness as i8);
        
        //run chunk encoding
        let mut run = 0;
        if brightness == prev_brightness{
            while i < pixels.len()/3
                && grayscale(pixels[i*3],pixels[i*3+1],pixels[i*3+2]) == brightness
                && run <= 0x03ff{
                run += 1;
                i+= 1
            }
        }

        if run > 1{
            fmi_op_run(&mut data_stream, run);
            run_chunks += 1;
            prev_brightness = brightness;
            continue;
        }else if run == 1{
            i -= 1; 
        }
        
        //hash byte encoding
        if gray_index[hash_index].is_none(){
            gray_index[hash_index] = Some(brightness);
            fmi_op_gray(&mut data_stream, brightness);
            gray_chunks += 1;
            i += 1;
            prev_brightness = brightness;
            continue;
        }else if gray_index[hash_index] == Some(brightness) {
            fmi_op_index(&mut data_stream, hash_index);
            index_bytes += 1;
            i += 1;
            prev_brightness = brightness;
            continue;
        }

        //difference byte encoding
        if difference >= -32 && difference < 32{
            difference = difference.wrapping_add(32);
            difference = 0b01000000 | difference;
            fmi_op_difference(&mut data_stream, difference as u8);
            diff_bytes += 1;
            i += 1;
            prev_brightness = brightness;
            continue;
        }
        
        //last resort grayscale chunk encoding
        gray_index[hash_index] = Some(brightness);
        fmi_op_gray(&mut data_stream, brightness);
        gray_chunks += 1;
        i += 1;
        prev_brightness = brightness;
    }
    
    //end data_stream
    for _ in 0..7{
        data_stream.push(0x00);
    }
    data_stream.push(0x01);

    println!("Chunk Totals:");
    println!("Index bytes/chunks: {}",index_bytes);
    println!("Difference bytes/chunks: {}",diff_bytes);
    println!("Run chunks: {}", run_chunks);
    println!("Gray chunks: {}",gray_chunks);
    println!("this file is {} kilobytes long", (((data_stream.len() as f32)/(1000 as f32)).to_string()).red());

    //write to file
    write(output_path.with_extension("fmi"), &data_stream).unwrap();
    println!("encoded in {:?}...", start.elapsed());
}
