# Embedded WASM Blinky
## WebAssembly Blinky on RP2350 Pico 2

A pure Embedded Rust project that runs a **WebAssembly runtime** (wasmtime + Pulley interpreter) directly on the RP2350 (Raspberry Pi Pico 2) bare-metal. A WASM module is AOT-compiled to Pulley bytecode on the host and executed on the device to control the onboard LED — no operating system and no standard library.

## Table of Contents

- [Embedded WASM Blinky](#embedded-wasm-blinky)
  - [WebAssembly Blinky on RP2350 Pico 2](#webassembly-blinky-on-rp2350-pico-2)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Architecture](#architecture)
  - [Project Structure](#project-structure)
  - [Prerequisites](#prerequisites)
    - [Toolchain](#toolchain)
    - [Flashing Tool](#flashing-tool)
    - [Optional (Debugging)](#optional-debugging)
  - [Building](#building)
  - [Flashing](#flashing)
  - [Testing](#testing)
  - [How It Works](#how-it-works)
    - [1. The WASM Application](#1-the-wasm-application)
    - [2. The Firmware Runtime](#2-the-firmware-runtime)
    - [3. The Build Pipeline](#3-the-build-pipeline)
  - [Host Function Interface](#host-function-interface)
  - [Memory Layout](#memory-layout)
  - [Extending the Project](#extending-the-project)
    - [Adding New Host Functions](#adding-new-host-functions)
    - [Changing Blink Speed](#changing-blink-speed)
  - [Troubleshooting](#troubleshooting)
  - [License](#license)

## Overview

This project demonstrates that WebAssembly is not just for browsers — it can run on a microcontroller with 512 KB of RAM. The firmware uses [wasmtime](https://github.com/bytecodealliance/wasmtime) with the **Pulley interpreter** (a portable, `no_std`-compatible WebAssembly runtime) and executes a precompiled WASM module that blinks GPIO25 at 500ms intervals.

**Key properties:**

- **Pure Rust** — zero C code, zero C bindings, zero FFI
- **Minimal unsafe** — only four unavoidable sites (heap init, boot metadata, module deserialize, panic handler GPIO)
- **Tiny WASM binary** — 191 bytes for the blinky module
- **AOT compilation** — WASM is compiled to Pulley bytecode on the host, no compilation on device
- **Industry-standard runtime** — wasmtime is the reference WebAssembly implementation

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 RP2350 (Pico 2)                 │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │            Firmware (src/main.rs)         │  │
│  │                                           │  │
│  │  ┌─────────┐  ┌────────┐  ┌───────────┐   │  │
│  │  │  Heap   │  │wasmtime│  │ Host Fns  │   │  │
│  │  │ 256 KiB │  │ Pulley │  │GPIO/Timer │   │  │
│  │  └─────────┘  └───┬────┘  └─────┬─────┘   │  │
│  │                   │             │         │  │
│  │              ┌────┴─────────────┴──────┐  │  │
│  │              │ Pulley Bytecode(.cwasm) │  │  │
│  │              │                         │  │  │
│  │              │  imports:               │  │  │
│  │              │    env.gpio_set_high()  │  │  │
│  │              │    env.gpio_set_low()   │  │  │
│  │              │    env.delay_ms(u32)    │  │  │
│  │              │                         │  │  │
│  │              │  exports:               │  │  │
│  │              │    run()                │  │  │
│  │              └─────────────────────────┘  │  │
│  └───────────────────────────────────────────┘  │
│                                                 │
│  GPIO25 (Onboard LED) ◄── set_high / set_low    │
└─────────────────────────────────────────────────┘
```

## Project Structure

```
embedded-wasm-blinky/
├── .cargo/
│   └── config.toml           # ARM Cortex-M33 target, linker flags, picotool runner
├── .vscode/
│   ├── extensions.json       # Recommended VS Code extensions
│   └── settings.json         # Rust-analyzer target configuration
├── wasm-app/                 # WASM blinky module (compiled to .wasm)
│   ├── .cargo/
│   │   └── config.toml       # WASM linker flags (minimal memory)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # Blinky logic: imports host GPIO/delay, exports run()
├── src/
│   ├── main.rs               # Firmware: hardware init, wasmtime runtime, host functions
│   └── platform.rs           # Platform TLS glue for wasmtime no_std
├── build.rs                  # Compiles WASM app, AOT-compiles to Pulley bytecode
├── Cargo.toml                # Firmware dependencies
├── rp2350.x                  # RP2350 memory layout linker script
├── SKILLS.md                 # Project conventions and lessons learned
└── README.md                 # This file
```

## Prerequisites

### Toolchain

```bash
# Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Required compilation targets
rustup target add thumbv8m.main-none-eabihf   # RP2350 ARM Cortex-M33
rustup target add wasm32-unknown-unknown      # WebAssembly
```

### Flashing Tool

```bash
# macOS
brew install picotool

# Linux (build from source)
# See https://github.com/raspberrypi/picotool
```

### Optional (Debugging)

```bash
cargo install probe-rs-tools
```

## Building

```bash
cargo build --release
```

This single command does everything:

1. `build.rs` compiles `wasm-app/` to `wasm32-unknown-unknown` → produces `wasm_app.wasm` (191 bytes)
2. `build.rs` AOT-compiles the WASM binary to Pulley bytecode via Cranelift → produces `blinky.cwasm`
3. The firmware compiles for `thumbv8m.main-none-eabihf`, embedding the Pulley bytecode via `include_bytes!`
4. The result is an ELF at `target/thumbv8m.main-none-eabihf/release/t-wasm`

## Flashing

```bash
cargo run --release
```

This builds the firmware and flashes it to the Pico 2 via `picotool` (configured as the cargo runner in `.cargo/config.toml`).

> **Note:** Hold the **BOOTSEL** button on the Pico 2 while plugging in the USB cable to enter bootloader mode. Release once connected.

After flashing, the LED on GPIO25 will begin blinking at 500ms intervals.

## Testing

```bash
cd wasm-tests && cargo test
```

Runs all 9 integration tests validating module loading, import/export contracts, blink sequencing, timing, and fuel-based execution limits.

## How It Works

### 1. The WASM Application

**File:** `wasm-app/src/lib.rs`

The WASM module is a `#![no_std]` Rust library compiled to `wasm32-unknown-unknown`. It declares three host imports and one export:

```rust
// Host-imported functions — these are provided by the firmware at runtime.
unsafe extern {
    safe fn gpio_set_high();
    safe fn gpio_set_low();
    safe fn delay_ms(ms: u32);
}

// Exported entry point called by the firmware.
#[unsafe(no_mangle)]
pub fn run() {
    loop {
        set_led_high();
        delay(500);
        set_led_low();
        delay(500);
    }
}
```

The `safe fn` declarations inside `unsafe extern` mean that calling these functions from Rust requires no `unsafe` block — the safety invariant is upheld by the firmware implementation.

The compiled WASM binary is only **191 bytes** because:
- No standard library (`#![no_std]`)
- Stack limited to 4 KB via linker flags
- Linear memory limited to 1 page (64 KB)
- LTO + size optimization (`opt-level = "s"`)

### 2. The Firmware Runtime

**File:** `src/main.rs`

The firmware performs these steps at boot:

1. **Initialize heap** — 256 KiB of the RP2350's 512 KiB RAM is allocated as a heap for the wasmtime runtime using `embedded-alloc`'s linked-list first-fit allocator.

2. **Initialize hardware** — Configures the external 12 MHz crystal oscillator, system clocks/PLLs, watchdog, SIO, GPIO25 (push-pull output), and Timer0.

3. **Create host state** — Wraps the LED pin and timer in boxed closures (`Box<dyn FnMut>`) so the WASM runtime doesn't need to know concrete HAL types.

4. **Boot the WASM runtime:**
   ```
   Config::target("pulley32")   → Explicit Pulley target (must match build.rs)
   Engine::new(&config)          → Create the wasmtime Pulley engine
   Module::deserialize(cwasm)   → Deserialize precompiled Pulley bytecode
   Store::new(host_state)       → Create a store holding our GPIO/delay closures
   Linker::new()                → Register host functions:
                                    env.gpio_set_high → (set_led)(true)
                                    env.gpio_set_low  → (set_led)(false)
                                    env.delay_ms      → cortex_m::asm::delay(ms * 150_000)
   linker.instantiate()         → Link imports, create WASM instance
   instance.get("run")          → Look up the exported run() function
   run.call()                   → Execute — blinks forever
   ```

### 3. The Build Pipeline

**File:** `build.rs`

The build script orchestrates three compilations in sequence:

```
cargo build --release
       │
       ▼
   build.rs runs:
       │
       ├── 1. Write rp2350.x → OUT_DIR/memory.x (linker script)
       │
       ├── 2. Spawn: cargo build --release --target wasm32-unknown-unknown
       │         └── wasm-app/ compiles → wasm_app.wasm (191 B)
       │
       ├── 3. AOT-compile to Pulley bytecode via Cranelift:
       │         └── Config::target("pulley32") → Engine → precompile_module
       │         └── engine.precompile_module(&wasm_bytes) → OUT_DIR/blinky.cwasm
       │
       └── 4. Main firmware compiles:
               └── include_bytes!("blinky.cwasm") embeds the Pulley bytecode
               └── Links against memory.x for RP2350 memory layout
               └── Produces ELF binary
```

A critical detail: the parent build's `CARGO_ENCODED_RUSTFLAGS` (containing ARM-specific flags like `--nmagic` and `-Tlink.x`) must be stripped from the child WASM build via `.env_remove("CARGO_ENCODED_RUSTFLAGS")`, otherwise the WASM linker will fail on unrecognized arguments.

## Host Function Interface

The WASM module communicates with hardware through three host functions registered under the `"env"` namespace:

| Import Name         | Signature    | Description                                                  |
| ------------------- | ------------ | ------------------------------------------------------------ |
| `env.gpio_set_high` | `() → ()`    | Sets GPIO25 (onboard LED) to high (on)                       |
| `env.gpio_set_low`  | `() → ()`    | Sets GPIO25 (onboard LED) to low (off)                       |
| `env.delay_ms`      | `(i32) → ()` | Blocks execution for N milliseconds (via CPU cycle counting) |

These are registered with the wasmtime `Linker` via `func_wrap()`, which wraps Rust closures as WASM-callable functions.

## Memory Layout

| Region             | Address      | Size            | Usage                                              |
| ------------------ | ------------ | --------------- | -------------------------------------------------- |
| Flash              | `0x10000000` | 2 MiB           | Firmware code + embedded WASM binary               |
| RAM (striped)      | `0x20000000` | 512 KiB         | Stack + heap + data                                |
| Heap (allocated)   | —            | 256 KiB         | wasmtime engine, store, module, WASM linear memory |
| WASM linear memory | —            | 64 KiB (1 page) | WASM module's addressable memory                   |
| WASM stack         | —            | 4 KiB           | WASM call stack                                    |

> **Important:** The default WASM linker allocates 1 MB of linear memory (16 pages). This exceeds the RP2350's total RAM. The `wasm-app/.cargo/config.toml` explicitly sets `--initial-memory=65536` (1 page) and `stack-size=4096`.

## Extending the Project

### Adding New Host Functions

1. Add the import declaration in `wasm-app/src/lib.rs`:
   ```rust
   unsafe extern {
       safe fn my_new_function(arg: u32);
   }
   ```

2. Register the host function in `src/main.rs`:
   ```rust
   linker.func_wrap("env", "my_new_function", |mut caller: Caller<'_, HostState>, arg: i32| {
       // Your implementation here
   }).expect("register my_new_function");
   ```

3. Add the corresponding field/closure to `HostState` if hardware access is needed.

### Changing Blink Speed

Edit the delay values in `wasm-app/src/lib.rs`:

```rust
pub fn run() {
    loop {
        set_led_high();
        delay(100);     // 100ms on
        set_led_low();
        delay(900);     // 900ms off
    }
}
```

Rebuild and reflash — only the 191-byte WASM binary changes.

## Troubleshooting

| Symptom                                         | Cause                                  | Fix                                                                                    |
| ----------------------------------------------- | -------------------------------------- | -------------------------------------------------------------------------------------- |
| LED not blinking after flash                    | WASM linear memory too large for heap  | Ensure `wasm-app/.cargo/config.toml` has `--initial-memory=65536`                      |
| LED stays solid after flash                     | Panic handler blinking too fast to see | Use large `cortex_m::asm::delay` values (37.5M+ cycles at 150MHz)                      |
| `Module::deserialize` panics                    | Config mismatch build vs device        | Both engines must have identical `Config` settings                                     |
| `Module::deserialize` panics                    | `default-features` mismatch            | Both `[dependencies]` and `[build-dependencies]` need `default-features = false`       |
| Build fails with `unknown argument: --nmagic`   | Parent rustflags leaking to WASM build | Ensure `build.rs` has `.env_remove("CARGO_ENCODED_RUSTFLAGS")`                         |
| Build fails with `extern blocks must be unsafe` | Rust 2024 edition                      | Use `unsafe extern { ... }` with `safe fn` declarations                                |
| `picotool` can't find device                    | Not in bootloader mode                 | Hold BOOTSEL while plugging in USB                                                     |
| `cargo build` doesn't pick up WASM changes      | Cached build artifacts                 | Run `cargo clean && cargo build --release`                                             |
| SIO register writes have no visible effect      | Using RP2040 offsets on RP2350         | RP2350 offsets differ: `GPIO_OUT_SET=0x018`, `GPIO_OUT_CLR=0x020`, `GPIO_OE_SET=0x038` |

## License

- [MIT License](LICENSE)
