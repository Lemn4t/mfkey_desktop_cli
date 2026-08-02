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
