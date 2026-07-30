# Changelog

## [1.0.6] - 2026-07-30

### Added

- Add one-time startup disclaimer
- Add HardNested detection stub
- Delete auto-mode logs only after a successful attack, with confirmation
- Add unit tests for core::parser (line classification, mfkey32/static_nested/static_encrypted/HardNested, load_nested_nonces)
- Auto-detect NO_COLOR and non-interactive stdout, forcing plain output even without --plain
- Add unit tests for ui::render\_\* formatting functions and ui::theme's style primitives

### Updated

- Combine no_ui and use_colors flags into plain_ui
- Group StaticEncrypted nonces by UID in one pass instead of O(n²)
- Move generated protobuf module declarations out of main.rs into src/generated_pb.rs
- Unify error handling around a shared Rslt<T> (Box<dyn Error>) with context, instead of ad hoc Strings and a typed error that got stringified at the auto/ boundary
- Throttle the should_stop poll in the crypto1 MSB search loop from every iteration to every 4096
- Replace crypto1_recover's per-nonce malloc/free scratch buffers with reusable thread-local storage
- Stop deriving serde::Serialize/Deserialize on every generated protobuf type; nothing used it outside core::config::Config
- Introduce a MessageKind enum so status colors are mapped in one place instead of scattered Color:: literals in auto/mod.rs and auto/upload.rs
- Extract src/ui/theme.rs for separators, glyphs, and style roles, removing dead colored()-branches left over from earlier Ui refactors
- Replace Ui's generic status/detail calls with named domain methods, so auto/mod.rs and auto/upload.rs no longer choose their own icons or colors
- Route every Ui line through a shared write_line/write_err_line so an active progress bar can no longer be corrupted by interleaved output
- Unify --plain and colored progress reporting on a single indicatif ProgressBar instead of a hand-rolled \r print in --plain mode
- Replace UiOptions' bool plain_ui with an OutputMode enum (Fancy/Plain), leaving room for a future Json mode
- Extract Ui's formatting logic into pure render\_\* functions, separate from the methods that print them

### Fixed

- Remove /GL flag from MSVC build
- Route all --auto/--plain output through Ui instead of raw colored calls
- Strip Windows verbatim path prefix (\\?\) from displayed paths
- Don't record a dict path when the write actually failed
- Route all remaining console output through the Ui layer instead of raw println!/eprintln! calls, including a --plain color leak in the "AUTO failed" message
- Rewrite the disclaimer flow to use Ui instead of its own duplicated color/prompt logic, fixing a nested-ANSI-color rendering bug in the "pill taken" message
- Show disclaimer and tool's title in main
- Only pass -fno-plt on Linux, avoiding a harmless but noisy "argument unused" warning on macOS builds

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
