#![no_std]
#![no_main]

use core::str;
use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use cstr_core::cstr;
use panic_halt as _;

use semihosting_files::{File, FileOpenMode, SeekFrom};

// Needed for linking
#[allow(unused_imports)]
use stm32f2::stm32f217 as _;

#[entry]
fn main() -> ! {
    let mut read = File::open(cstr!("./examples/files/read.txt"), FileOpenMode::Read)
        .expect("Could not open read.txt");
    let mut write = File::open(
        cstr!("./examples/files/write.txt"),
        FileOpenMode::WriteTruncate,
    )
    .expect("Could not open write.txt");

    let read_length = read.len().expect("Could not get length of read.txt");
    hprintln!("File length: {}", read_length).unwrap();

    read_whole_file(&mut read);

    read.rewind().expect("Could not seek to start");
    read_file_chunks(&mut read);

    hprintln!("Seeking inside, the file, skipping first section").unwrap();
    read.seek(SeekFrom::Start(5))
        .expect("Could not seek read.txt");
    read_whole_file(&mut read);

    hprintln!("Seeking inside, the file, skipping everything but the last section").unwrap();
    read.seek(SeekFrom::End(-5))
        .expect("Could not seek read.txt");
    read_whole_file(&mut read);

    hprintln!("Writing to file").unwrap();
    write.write(b"Hello").expect("Could not write to write.txt");
    write.write(&[b' ']).expect("Could not write to write.txt");
    write
        .write(b"World!")
        .expect("Could not write to write.txt");

    hprintln!("Closing files").unwrap();
    read.close().expect("Could not close read.txt");
    write.close().expect("Could not close write.txt");

    hprintln!("Done").unwrap();

    loop {
        asm::nop();
    }
}

fn read_whole_file(file: &mut File) {
    hprintln!("Reading whole file:").unwrap();

    // Sufficiently large buffer for the file
    // only one read operation is needed
    let mut buffer = [0; 64];

    let length = file.read(&mut buffer).expect("Could not read from file");

    let string = str::from_utf8(&buffer[..length]).expect("file was not valid utf8");

    hprintln!("File contained: {}", string).unwrap();
}

fn read_file_chunks(file: &mut File) {
    hprintln!("Reading file in chunks:").unwrap();

    let mut buffer = [0; 8];

    for i in 1.. {
        let length = file
            .read(&mut buffer)
            .expect("Could not read chunk from file");

        if length == 0 {
            // reached EOF
            return;
        }

        let string = str::from_utf8(&buffer[..length]).expect("Chunk was not valid utf8");

        hprintln!("Chunk {}: {}", i, string).unwrap();
    }
}
