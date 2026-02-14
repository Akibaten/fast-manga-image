// src/main.rs
use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: fmi <command> [args]");
        std::process::exit(1);
    }
    
    let command = &args[1];
    let remaining_args = &args[2..];
    
    let binary_name = match command.as_str() {
        "encode" => "encode",
        "convert" => "encode",
        "view" => "decode",
        "decode" => "decode",
        "video" => "decode_frames",
        "decode_frames" => "decode_frames",
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    };
    
    let status = Command::new(binary_name)
        .args(remaining_args)
        .status()
        .expect("Failed to execute command");
    
    std::process::exit(status.code().unwrap_or(1));
}
