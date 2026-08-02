# Changelog

## [1.0.6] - 2026-08-02

### Added

- A one-time disclaimer is shown on first launch, and your choice is remembered. Pass `--accept-disclaimer` to accept it automatically (for example, in scripts).
- HardNested capture logs are now recognized and clearly reported as not yet supported, instead of quietly finding no keys.
- Output automatically switches to plain, color-free text when it is sent to a file or pipe, or when the `NO_COLOR` environment variable is set — you no longer need to pass `--plain`.

### Changed

- Faster and more efficient key recovery, especially on large captures.
- Clearer, steadier progress and status display; the progress bar can no longer be scrambled by other messages.
- In auto mode, the original log files on the Flipper are now deleted only after a successful recovery, and only after you confirm.
- Large internal cleanup and refactoring, plus a much wider automated test suite (75 tests). This does not change how the tool is used.

### Fixed

- Windows: saved file paths are shown without the confusing `\\?\` prefix.
- Colors no longer leak into plain-text output, including error messages.
- The tool no longer stops with an error when a home or configuration folder cannot be found; it simply skips saving your disclaimer choice.
- Corrected build settings and documentation, including the note describing how the project is built.

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
