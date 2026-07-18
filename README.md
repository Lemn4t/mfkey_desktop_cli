<div align="center">

# 🔑 Flipper Zero :: MFKey Desktop CLI

**A cross-platform CLI tool for recovering MIFARE Classic keys from Flipper Zero nonce logs**

[![Latest Release](https://img.shields.io/github/v/release/phntm-lab/mfkey_desktop_cli?style=for-the-badge&logo=github&color=blue)](https://github.com/phntm-lab/mfkey_desktop_cli/releases/latest)
[![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads/phntm-lab/mfkey_desktop_cli/latest/total?style=for-the-badge&logo=github&color=success)](https://github.com/phntm-lab/mfkey_desktop_cli/releases/latest)

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![C](https://img.shields.io/badge/C-00599C?style=for-the-badge&logo=c&logoColor=white)](<https://en.wikipedia.org/wiki/C_(programming_language)>)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey?style=for-the-badge)](https://github.com/phntm-lab/mfkey_desktop_cli/releases)

[![GitHub License](https://img.shields.io/github/license/phntm-lab/mfkey_desktop_cli?style=for-the-badge&color=orange)](LICENSE)

</div>

---

## 📥 Download

Ready-made builds for Windows, Linux and macOS are available on the releases page — no dependencies needed, just download and run:

### ➡️ **[github.com/phntm-lab/mfkey_desktop_cli/releases](https://github.com/phntm-lab/mfkey_desktop_cli/releases)**

---

## ✨ Features

- ⚡ **High-performance Crypto-1 core in C**, based on [crapto1](https://github.com/RfidResearchGroup/proxmark3) — the open-source Crypto-1 cipher implementation used across the Proxmark3/libnfc ecosystem
- 🦀 **Thin, safe Rust layer** on top — parsing, attack orchestration, progress, and CLI
- 🎯 Support for **three attack types**, auto-detected from the input file:
  - `mfkey32` — key recovery from two intercepted authentications (Mfkey32 / Moebius)
  - `static_nested` — attacking cards with a predictable (static) nested PRNG
  - `static_encrypted` — attacking cards where only encrypted nonces were collected
- 🤖 **`--auto` mode** — talks to a connected Flipper Zero directly over USB: detects the port, pulls `.mfkey32.log` / `.nested.log` files, runs the attack, and uploads recovered keys and candidate dictionaries back to the device — fully hands-free
- 🛑 Graceful `Ctrl+C` interruption at any point
- 💾 Saves both confirmed keys and candidate key dictionaries to disk

---

## 🚀 Usage

### Single file (offline)

```bash
mfkey_desktop_cli <input_file> [output_keys.nfc] [dict_output_dir]
```

Example:

```bash
mfkey_desktop_cli .nested.log
```

The attack type is detected automatically from the file contents — no need to specify it. Recovered keys are written to `mf_classic_dict_user.nfc` by default.

### Automatic mode (live device)

```bash
mfkey_desktop_cli --auto
mfkey_desktop_cli --auto --port COM3
```

See [Automatic mode](#-automatic-mode---auto) below for details.

### All options

```
Usage: mfkey_desktop_cli [OPTIONS] <.nested.log/.mfkey32.log> [output_keys.nfc] [dict_output_dir]

OPTIONS:
  -h, --help        Show the help message and exit
  --no-ui           Disable the interactive UI and use plain text output
  --version         Show version information

AUTO MODE (Flipper Zero over USB):
  --auto            Find a connected Flipper, pull *.mfkey32.log / *.nested.log
                     from /ext/nfc, run the attack, and upload recovered keys
                     to /ext/nfc/assets/mf_classic_dict_user.nfc
  --port <PORT>     (optional) Serial port of the Flipper (skips auto-detect)
  --out <DIR>       (optional) Directory for local copies of logs/keys
```

---

## 🤖 Automatic mode (`--auto`)

`--auto` turns the tool into a one-click solution that talks **directly to a connected Flipper Zero** over USB — no manual file copying required.

```bash
mfkey_desktop_cli --auto
```

If the port isn't detected automatically, specify it manually:

```bash
mfkey_desktop_cli --auto --port COM3                       # Windows
mfkey_desktop_cli --auto --port /dev/ttyACM0                # Linux
mfkey_desktop_cli --auto --port /dev/cu.usbmodemflip_XXXX1  # macOS
```

> [!TIP]
> On Windows, find the Flipper's COM port in **Device Manager → Ports (COM & LPT)**. A Flipper can expose more than one COM port — if the first doesn't respond, try the next.

> [!NOTE]
> Before running `--auto`, close qFlipper, the Web Updater, and any serial terminals — they hold the port exclusively and will block the connection.

### What it does, step by step

1. **🔌 Detects** the Flipper Zero's serial port (or uses `--port`).
2. **🤝 Opens an RPC session** over USB-CDC.
3. **📂 Lists** `/ext/nfc` and finds every `.mfkey32.log` and `.nested.log` file present.
4. **⬇️ Downloads** each log and runs the matching attack on it; recovered keys from all files are merged together.
5. **🔀 Merges dictionaries smartly:** if `mf_classic_dict_user.nfc` already exists on the device, only genuinely new keys are appended to it; otherwise a fresh dictionary is created.
6. **⬆️ Uploads** the merged key dictionary back to `/ext/nfc/assets/mf_classic_dict_user.nfc` (skipped entirely if nothing new was found).
7. **🗂️ Uploads candidate dictionaries** (`mf_classic_dict_<uid>.nfc`) to `/ext/nfc/assets/` as well, always overwriting any existing file with the same name.

> [!NOTE]
> Uploading candidate dictionaries can take a while over USB-CDC if there are many of them or they're large — this is expected.

---

## 📄 Input file format

**Nested** (Flipper Zero format):

```
Sec 0 key B cuid da7d3c2e nt0 b07cef37 ks0 54a0efed par0 1001 nt1 224737c4 ks1 ce956841 par1 1000 dist 0
```

**Mfkey32** (Flipper Zero format):

```
Sec 0 key A cuid 801aa11c nt0 e58455e4 nr0 761ff4ec ar0 162122ec nt1 20782e85 nr1 ecb6f04f ar1 bf19891b
```

The tool detects which attack applies to each line automatically — you don't need to sort or split the log yourself.

---

## 🛠️ Building from source

Requires the [Rust toolchain](https://rustup.rs/) and a C compiler (MSVC on Windows, GCC/Clang elsewhere).

```bash
git clone https://github.com/phntm-lab/mfkey_desktop_cli.git
cd mfkey_desktop_cli
cargo build --release
```

The binary will be in `target/release/`.

> [!NOTE]
> Flipper RPC protobuf definitions are compiled at build time using the pure-Rust [`protox`](https://crates.io/crates/protox) parser — a system-wide `protoc` install is not required.

---

## 🧩 How it works

| Layer                 | Language     | Responsibility                                              |
| --------------------- | ------------ | ------------------------------------------------------------ |
| Crypto-1 core          | **C**        | LFSR state recovery, MSB-table search (crapto1-based)         |
| Attack engine & parser | **Rust**     | Nonce parsing, attack orchestration, progress reporting        |
| FFI bridge             | **Rust ↔ C** | Passing structures and callbacks between the two layers        |
| Flipper RPC (USB)      | **Rust**     | Serial transport, protobuf framing, Storage read/write/delete   |
| CLI & UI               | **Rust**     | Argument parsing (clap), console output, progress bar          |

The attacks exploit known cryptographic weaknesses in the Crypto-1 cipher used by MIFARE Classic cards. In `--auto` mode, the tool speaks the Flipper Zero's **Protobuf RPC** protocol over USB-CDC — it starts an RPC session and uses `Storage*` commands to list, read, write, and delete files on the device.

---

## ⚖️ Disclaimer

> [!WARNING]
> This tool is intended **exclusively** for security research, education, and testing on cards you own or have explicit permission to analyze.
>
> The author is not responsible for any misuse. Use at your own risk and in accordance with the laws of your jurisdiction.

---

## 🙏 Credits

- [Proxmark3 / RfidResearchGroup](https://github.com/RfidResearchGroup/proxmark3) — Crypto-1 / crapto1 recovery algorithm
- [mfkey / noproto](https://github.com/noproto/xero-firmware/tree/dev/applications/system/mfkey) — Flipper Zero MFKey app
- [Flipper Zero Protobuf](https://github.com/flipperdevices/flipperzero-protobuf) — RPC protocol definitions

---

## 📄 License

Distributed under the **GPL-3.0** license. See [LICENSE](LICENSE) for details.
