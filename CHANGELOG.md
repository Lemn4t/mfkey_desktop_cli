# Changelog

## [1.0.6] - 2026-07-18

### Added

- Add one-time startup disclaimer

### Updated

### Fixed

- Remove /GL flag from MSVC build

## [1.0.5] - 2026-07-18

### Added

- Migrated CLI parsing to `clap`; added `params` module for argument handling.

### Updated

- Reorganized the codebase into domain-based modules (`core/`, `auto/`, `ui/`, `params/`, `flipper/`) for better maintainability; no functional changes.

### Fixed

- Factor out SaveDictFn type alias to fix type_complexity in run_attack
- Collapse nested if-let chains in find_flipper_port, find_all_flipper_ports, storage_read, show_saved_files, and main()
- Take self by value in MfClassicKey::to_hex to fix wrong_self_convention

## [1.0.4] - 2026-06-27

### Added

- Candidate dictionaries are now uploaded to the device.
- Multithreaded key recovery using `rayon`: nonces are now processed in parallel across all CPU cores for a major speedup on multi-core systems.
- Optional `MFKEY_NATIVE=1` build flag to enable CPU-native optimizations (`-march=native` / `/arch:AVX2`) for local builds.

### Updated

- Found keys are now printed in real time as they are discovered during the attack, instead of being shown in a batch at the end.
- Reworked progress display to use a single stable progress bar (processed/total nonces), removing flickering under multithreading.
- Enabled additional portable C compiler optimizations and Rust LTO for release builds.

### Fixed

- Fixed builds of the application and proto files
- Fixed `Found key:` lines occasionally not being displayed during attacks.
- Prevented potential data races by isolating per-task state and merging results in a single thread.
