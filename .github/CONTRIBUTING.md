# Contributing to MFKey Desktop CLI

First off — thank you for taking the time to contribute! 🙏

`mfkey_desktop_cli` is a cross-platform MIFARE Classic key-recovery tool for
Flipper Zero, built with a **high-performance Crypto-1 core in C** and a
**Rust** layer for parsing, attack orchestration, the CLI and the Flipper RPC
(USB) transport. Contributions of all kinds are welcome: bug fixes,
performance improvements, new attacks, documentation, and more.

> By contributing to this project, you agree that your contributions will be
> licensed under the project's **GPL-3.0** license.

---

## 🤝 Code of Conduct

Be respectful, constructive and patient. We're all here to build something
useful together. Harassment or hostile behavior of any kind will not be
tolerated.

---

## ⚖️ Legal & Ethical Use

This tool is intended **exclusively** for security research, training, and
testing **your own** cards or cards you have **explicit permission** to analyze.

Contributions that are designed primarily to facilitate illegal access,
fraud, or any unlawful activity will not be accepted. By contributing, you
confirm that your contribution complies with this principle and with the laws
of your country.

---

## 🙋 How Can I Contribute?

### Reporting bugs

- Search [existing issues](https://github.com/phntm-lab/mfkey_desktop_cli/issues)
  first to avoid duplicates.
- Open a new issue using the **🐛 Bug Report** template and fill it out as
  completely as possible (OS, version, attack type, `--auto` vs single file,
  Flipper firmware, logs).

### Suggesting changes

- For improvements to existing behavior, use the **⚡ Enhancement** template.
- For brand-new functionality, use the **🎯 Feature Request** template.

### Contributing code

- For **small fixes** (typos, obvious bugs), feel free to open a PR directly.
- For **larger changes** (new attacks, RPC changes, architectural changes),
  please open an issue first to discuss the approach before investing time.

---

## 🛠️ Development Setup

You will need the [Rust toolchain](https://rustup.rs/) and a C compiler
(MSVC / GCC / Clang).

```bash
git clone https://github.com/phntm-lab/mfkey_desktop_cli.git
cd mfkey_desktop_cli
cargo build --release
```

The compiled binary appears in `target/release/`.

> **Note:** Protobuf definitions for the Flipper RPC protocol are compiled at
> build time with the pure-Rust [`protox`](https://crates.io/crates/protox)
> parser, so **`protoc` is not required**.

### Project layout

| Path       | Description                                                       |
| ---------- | ----------------------------------------------------------------- |
| `csrc/`    | Crypto-1 recovery core (C) — LFSR state restoration, MSB tables   |
| `src/`     | Rust code — parsing, attack orchestration, CLI, Flipper RPC (USB) |
| `proto/`   | Protobuf definitions for the Flipper Zero RPC protocol            |
| `build.rs` | Build script (compiles C sources and protobufs)                   |
| `.github/` | Issue/PR templates and community health files                     |

### Platform-specific notes for testing `--auto`

- **Windows:** find the Flipper COM port in _Device Manager → Ports (COM & LPT)_.
- **Linux:** you may need to join the `dialout` group:
  `sudo usermod -aG dialout $USER` (re-login afterwards).
- **macOS:** Gatekeeper may need approval in
  _System Settings → Privacy & Security_.
- ⚠️ Close **qFlipper**, the **Web Updater**, and any **serial terminals**
  before testing `--auto` — they hold the serial port exclusively.

---

## 🌿 Branching & Workflow

- The default development branch is **`dev`**. Please **base your work on
  `dev`** and target your pull requests at `dev`.
- Create a topic branch with a descriptive name, e.g.:
  - `fix/nested-parser-overflow`
  - `feat/static-encrypted-progress`
  - `docs/readme-macos-notes`
- Keep your branch up to date with `dev` (rebase preferred over merge commits).

---

## ✅ Code Quality Standards

Before submitting a pull request, please make sure:

### Rust

```bash
cargo fmt --all              # format the code
cargo clippy --all-targets   # lint (no new warnings)
cargo build --release        # builds cleanly
```

### C

- Format C sources using the project's `.clang-format`:
  ```bash
  clang-format -i csrc/*.c csrc/*.h
  ```
- Avoid introducing new compiler warnings.
- Be mindful of memory safety at the FFI boundary (Rust ↔ C): document
  ownership, lifetimes and any `unsafe` invariants.

### General

- Keep changes focused — one logical change per PR.
- Add or update documentation (README) when behavior changes.
- Prefer cross-platform code; if something is platform-specific, gate and
  document it.

---

## 📝 Commit Messages

Write clear, descriptive commit messages. We recommend a
[Conventional Commits](https://www.conventionalcommits.org/)-style prefix:

```
feat: add progress reporting to static_nested attack
fix: handle malformed cuid in mfkey32 parser
perf: reduce allocations in Crypto-1 candidate filtering
docs: clarify --port usage on Windows
refactor: simplify RPC framing in storage module
```

Reference related issues where applicable (e.g. `Closes #42`).

---

## 📬 Submitting a Pull Request

1. Fork the repository and create your branch from `dev`.
2. Make your changes following the standards above.
3. Test on at least one platform; if possible, note which attack types and
   whether you tested offline (single file) and/or `--auto` (live device).
4. Fill out the **pull request template** completely.
5. Run local checks yourself before opening the PR (`fmt`, `clippy`, `build`) — there is currently no automated CI on pull requests, only on pushes to `dev`/`release`.
6. Open the PR against `dev` and link any related issues.

A maintainer will review your PR. Please be responsive to feedback — small
follow-up commits are perfectly fine.

---

## ❓ Questions

For general questions or discussion you can reach out via
[Telegram](https://t.me/Lemn4t). For bugs and feature ideas,
please use the issue templates.

Thanks again for contributing! 🎉
