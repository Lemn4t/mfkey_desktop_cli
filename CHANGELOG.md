# Changelog

## [1.0.6] - 2026-08-02

### Added

- Add one-time startup disclaimer
- Add HardNested detection stub
- Delete auto-mode logs only after a successful attack, with confirmation
- Add unit tests for core::parser (line classification, mfkey32/static_nested/static_encrypted/HardNested, load_nested_nonces)
- Auto-detect NO_COLOR and non-interactive stdout, forcing plain output even without --plain
- Add unit tests for ui::render\_\* formatting functions and ui::theme's style primitives
- Add unit tests for core::model (MfClassicKey from_slice/to_hex, save_keys_to_file), core::state's DedupVec, core::paths::display_path, and core::parser's mfkey32 line parsing

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
- Split src/ui/mod.rs into ui/attack_report.rs and ui/auto_report.rs, grouped by which caller actually uses them
- Extract src/run.rs for the single-file mode's orchestration, moving save_keys_to_file into core::model and display_path into core::paths, leaving main.rs as a thin entry point
- Extract a shared accumulate() helper in core::state, replacing four hand-rolled copies of the same insert-if-new bookkeeping in TaskState/AttackState
- Unify the Mfkey32 and StaticNested attack passes in engine::run_attack behind a shared run_pass() helper, leaving the StaticEncrypted pass untouched
- Remove dead fields and unused parameters (TaskState.current_uid, AttackState::new's ignored nonce-count argument, UiInner.total_nonces, DirEntry.size)
- Deduplicate Flipper port discovery by having find_flipper_port reuse find_all_flipper_ports instead of a second copy of the scan loop
- Remove the unused storage_exists RPC helper
- Migrate FlipperError to a thiserror-derived enum, dropping the hand-written Display/Error/From impls while keeping the exact same error messages
- Make run() return Rslt<()> and centralize its exit-code handling in main, matching the --auto arm instead of calling process::exit inline
- Decompose the ~125-line run_auto into focused helpers (resolve_logs_dir, discover_port, download_target_logs, run_attacks_over_logs, maybe_delete_remote_logs)
- Replace the duplicated dedup bookkeeping in TaskState/AttackState with a shared DedupVec<T> newtype (Vec + HashSet, derefs to a slice), dropping the accumulate() helper and the parallel *_set fields
- Split the StaticEncrypted path of engine::run_attack into group_static_encrypted_by_uid and process_uid_group helpers, leaving run_attack a linear two-pass-plus-loop
- Move dictionary-file error reporting out of the file-writing helper: split save_candidate_dict into a pure candidate_dict_path and a UI-free write_candidate_dict, leaving the run_file_attack closure to report failures
- Derive Default for Nonce and AttackType instead of a hand-written impl, and build MfClassicKey::to_hex in a single preallocated String instead of one allocation per byte
- Read a parsed nonce's uid/attack_name before pushing it, removing a nonces.last().unwrap() in load_nested_nonces
- Pass candidate dictionary paths to show_saved_dicts as &str instead of cloning them into owned Strings
- Route the --auto report's status glyphs through theme::glyph and omit them in plain mode, matching the attack report's glyph-free plain output

### Fixed

- Remove /GL flag from MSVC build
- Route all --auto/--plain output through Ui instead of raw colored calls
- Strip Windows verbatim path prefix (\\?\) from displayed paths
- Don't record a dict path when the write actually failed
- Route all remaining console output through the Ui layer instead of raw println!/eprintln! calls, including a --plain color leak in the "AUTO failed" message
- Rewrite the disclaimer flow to use Ui instead of its own duplicated color/prompt logic, fixing a nested-ANSI-color rendering bug in the "pill taken" message
- Show disclaimer and tool's title in main
- Only pass -fno-plt on Linux, avoiding a harmless but noisy "argument unused" warning on macOS builds
- Return a dedicated FlipperError::Encode on protobuf encode failure instead of mislabeling it as a generic protocol error
- Return Option from config_path and handle a missing home/config directory gracefully instead of panicking (the config simply isn't persisted)
- Correct the README build note: protobuf definitions are compiled with prost-build and a vendored protoc (protoc-bin-vendored), not the pure-Rust protox parser

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
