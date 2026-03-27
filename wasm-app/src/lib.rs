//! SPDX-License-Identifier: MIT
//!
//! Copyright (c) 2026 Kevin Thomas
//!
//! # WASM Blinky Component
//!
//! A minimal WebAssembly component that blinks the onboard LED on an RP2350
//! Pico 2 by calling host-provided GPIO and delay functions through typed
//! WIT interfaces. GPIO pins are addressed by their hardware pin number
//! (e.g., 25 for the onboard LED).

#![no_std]

extern crate alloc;

use core::panic::PanicInfo;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

use embedded::platform::{gpio, timing};

wit_bindgen::generate!({
    world: "blinky",
    path: "../wit",
});

struct BlinkyApp;

export!(BlinkyApp);

impl Guest for BlinkyApp {
    fn run() {
        const LED_PIN: u32 = 25;
        loop {
            gpio::set_high(LED_PIN);
            timing::delay_ms(500);
            gpio::set_low(LED_PIN);
            timing::delay_ms(500);
        }
    }
}

/// Panic handler for the WASM environment that halts in an infinite loop.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
