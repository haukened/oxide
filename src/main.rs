#![no_std]
#![no_main]

use uefi::prelude::*;

#[entry]
fn efi_main() -> Status {
    // init allocator, logger, panic handler, and console
    uefi::helpers::init().unwrap();

    // clear the screen
    uefi::system::with_stdout(|stdout| {
        stdout.clear().unwrap();
    });

    // print messages to the UEFI console
    uefi::println!("Hello, UEFI World!");
    uefi::println!("control transferred from firmware to UEFI application");

    loop {
        core::hint::spin_loop();
    }
}