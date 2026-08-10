# Changelog

## [1.0.7] - 2026-08-10

### Added

- HardNested attack support: recover MIFARE Classic keys from HardNested nonces
  in Flipper Zero `.nested.log` files. HardNested logs now run the attack
  (in single-file and `--auto` modes) instead of being reported as unsupported.
- Bluetooth LE live mode (`--ble`, with optional `--device <ID>`): run the same
  hands-free download → attack → upload flow over BLE instead of USB. The Flipper
  must be paired in the operating system's Bluetooth settings first.
- Interactive device selection when multiple Flippers are connected: a picker in
  the fancy UI and a numbered prompt in `--plain` (`--port` / `--device` skip it).

### Changed

### Fixed
