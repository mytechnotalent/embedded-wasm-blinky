//! SPDX-License-Identifier: MIT
//!
//! Copyright (c) 2026 Kevin Thomas
//!
//! # WASM Blinky Application
//!
//! A minimal WebAssembly module that blinks the onboard LED on an RP2350 Pico 2
//! by calling host-provided GPIO and delay functions. GPIO pins are addressed
//! by their hardware pin number (e.g., 25 for the onboard LED).

#![no_std]

use core::panic::PanicInfo;

// Host-imported functions provided by the firmware WASM runtime.
// The `unsafe extern` block is required by Rust 2024 — no C code is involved.
// Individual functions are declared `safe fn` so callers need no unsafe.
unsafe extern "C" {
    /// Sets the specified GPIO pin to a high (on) state.
    ///
    /// # Arguments
    ///
    /// * `pin` - GPIO pin number (e.g., 25 for onboard LED).
    safe fn gpio_set_high(pin: u32);
    /// Sets the specified GPIO pin to a low (off) state.
    ///
    /// # Arguments
    ///
    /// * `pin` - GPIO pin number (e.g., 25 for onboard LED).
    safe fn gpio_set_low(pin: u32);
    /// Delays execution for the specified number of milliseconds.
    ///
    /// # Arguments
    ///
    /// * `ms` - Duration of the delay in milliseconds.
    safe fn delay_ms(ms: u32);
}

/// Panic handler for the WASM environment that halts in an infinite loop.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// GPIO pin number for the onboard LED.
const LED_PIN: u32 = 25;

/// Calls the host function to set the given GPIO pin high.
///
/// # Arguments
///
/// * `pin` - GPIO pin number.
fn set_pin_high(pin: u32) {
    gpio_set_high(pin);
}

/// Calls the host function to delay execution for the given milliseconds.
///
/// # Arguments
///
/// * `ms` - Duration of the delay in milliseconds.
fn delay(ms: u32) {
    delay_ms(ms);
}

/// Calls the host function to set the given GPIO pin low.
///
/// # Arguments
///
/// * `pin` - GPIO pin number.
fn set_pin_low(pin: u32) {
    gpio_set_low(pin);
}

/// WASM entry point that blinks the onboard LED at 500ms intervals indefinitely.
///
/// Calls host-provided GPIO and delay functions in a continuous loop to toggle
/// the LED on and off with a half-second period.
#[unsafe(no_mangle)]
pub fn run() {
    loop {
        set_pin_high(LED_PIN);
        delay(500);
        set_pin_low(LED_PIN);
        delay(500);
    }
}
