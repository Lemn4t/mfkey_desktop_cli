## 📋 Description

<!-- Provide a clear and concise description of what this PR does and why. -->

## 🔗 Related issue(s)

<!-- Link the issue this PR addresses, e.g. "Closes #123". -->

Closes #

## 🧩 Type of change

<!-- Mark all that apply with an "x". -->

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ⚡ Enhancement (improvement to existing functionality)
- [ ] 🎯 New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that changes existing behavior)
- [ ] 📄 Documentation update
- [ ] 🧹 Refactor / code style / build (no functional changes)

## 🛠️ Affected components

- [ ] Crypto-1 core (C)
- [ ] Attack orchestration / parsing (Rust)
- [ ] CLI interface
- [ ] Automatic mode (`--auto` / Flipper RPC over USB)
- [ ] Dictionary handling (`mf_classic_dict_user.nfc`)
- [ ] FFI bridge (Rust ↔ C)
- [ ] Build system (`Cargo.toml` / `build.rs` / `.clang-format`)
- [ ] Documentation

## ✅ How has this been tested?

<!-- Describe the tests you ran. Include OS, attack type, and whether you tested offline or with a live device. -->

- **OS tested on:** <!-- Windows / Linux / macOS -->
- **Attack type(s):** <!-- mfkey32 / static_nested / static_encrypted -->
- **Mode:** <!-- single file (offline) / --auto (live device) -->

Test details:

```
<!-- paste relevant commands / output here -->
```

## 📸 Screenshots / output (if applicable)

<!-- Add terminal output or screenshots demonstrating the change. -->

## ☑️ Checklist

- [ ] My code follows the project's style (`cargo fmt`, `.clang-format` for C code).
- [ ] `cargo build --release` succeeds without new warnings.
- [ ] `cargo clippy` passes (if applicable).
- [ ] I have tested my changes on at least one platform.
- [ ] I have updated the documentation (README) where necessary.
- [ ] My changes do not introduce new compiler/linker warnings.
- [ ] I have searched for and linked any related issues/PRs.

## 📝 Additional notes

<!-- Anything else the reviewer should know (trade-offs, follow-ups, etc.). -->
