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

### Fixed
