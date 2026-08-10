# Changelog

## [1.0.8] - 2026-08-10

### Added

- Weak-nested logs are now recovered. `.nested.log` lines with a non-zero `dist`
  (weak-PRNG nested collections) were previously ignored; they now run the nested
  attack just like the static case. The plaintext nonce is resolved by the Flipper
  firmware, so `dist` is not needed for recovery on the desktop side.

### Changed

- Releases now include a `SHA256SUMS` file, and the README documents how to
  verify downloads and why antivirus engines may flag the tool as a false
  positive.
- HardNested progress is now shown as a single, in-place status line with a
  per-target counter ("HardNested [k/N] …") in the interactive UI, instead of
  scrolling the raw engine table.
- Unrecognized log lines are now reported ("N unrecognized line(s) were
  skipped") instead of being dropped silently.

### Fixed

- HardNested nonces are now grouped per (UID, sector, key type) target rather
  than per (UID, key type), so logs covering several sectors recover each key
  correctly instead of mixing their nonces.
