# esp32-rust-wifi-test
ESP32C3 application to connect to your wifi and run a simple HTTP GET request

This project use the cfg-toml crate to remove hardcoded wifi credentials from your code.
In order to connect to your wifi, rename the `cfg.toml.example` to `cfg.toml` and enter your wifi credentials there.

## Usage
Flash and monitor your ESP32C3 with:
`cargo espflash /dev/ttyUSB0 -s 4MB --speed 921600 --partition-table partitions.csv --release --monitor`
