use std::fs::File;
use std::io::{Read, Stdin};

use clap::{App, Arg};
use esp_exception_decoder_rs::decode;
use colored::*;

enum DumpSource<'a> {
    FilePath(&'a str),
    Stdin(Stdin),
}

fn main() {
    let matches = App::new("hardliner")
        .version("0.1")
        .about("ESP8266 exception decoder")
        .arg(
            Arg::with_name("elf")
                .value_name("binary_file")
                .help(
                    "Specify the name of the .elf executable.",
                )
                .required(true),
        )
        .arg(
            Arg::with_name("dump")
                .value_name("stack_trace_file")
                .help("Stack trace file."),
        )
        .get_matches();

    let binary_file = matches.value_of("elf").unwrap();
    let mut binary_file = File::open(binary_file).unwrap();
    let mut binary_buf = Vec::<u8>::new(); 
    let _ = binary_file.read_to_end(&mut binary_buf);

    let stdin = std::io::stdin();
    let stack_trace_source = matches
        .value_of("dump")
        .map(DumpSource::FilePath)
        .unwrap_or_else(|| DumpSource::Stdin(stdin));

    let stack_trace = match stack_trace_source {
        DumpSource::FilePath(file) => {
            let mut file = File::open(file).unwrap();
            let mut stack_trace = String::new();
            let _ = file.read_to_string(&mut stack_trace).unwrap();
            stack_trace
        },
        DumpSource::Stdin(stdin) => {
            let mut stdin_buffer = Vec::<u8>::new();
            let _ = stdin.lock().read_to_end(&mut stdin_buffer).unwrap();
            let stack_trace = String::from_utf8(stdin_buffer).unwrap();
            stack_trace
        }
    };

    let decoded_addresses = decode(&binary_buf, &stack_trace);
    for address in decoded_addresses {
        println!("0x{:04x}: {} at {}", 
            address.address, 
            address.function_name.bold(),
            address.location.blue()
        );
    }
}
