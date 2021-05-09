# ESP AT Embassy Driver
[![dependency status](https://deps.rs/repo/github/rukai/esp-at-driver/status.svg)](https://deps.rs/repo/github/rukai/esp-at-driver)
[![Crates.io](https://img.shields.io/crates/v/esp-at-driver.svg)](https://crates.io/crates/esp-at-driver)
[![Released API docs](https://docs.rs/esp-at-driver/badge.svg)](https://docs.rs/esp-at-driver)

Rust driver that runs on the main processor and communicates to an ESP coprocessor running the [AT firmware](https://github.com/espressif/esp-at).

My board hasn't actually arrived yet so this is all powered on wishful thinking and rust's type safety.

## Why Embassy/Async

Implementing this crate with async means it is unable to run in some environments.
However the expected use case of this library is on a main processor talking to an ESP coprocessor.
In such an environment it is expected for the main processor to have enough storage for async binary sizes, likely an stm32 or nrf chip.

## Implementation

The driver uses the AT commands documented [here](https://docs.espressif.com/projects/esp-at/en/latest/AT_Command_Set/index.html).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
