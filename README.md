<div align="center">

# 🔑 Flipper Zero :: MFKey Desktop CLI

**MIFARE Classic Cross-platform CLI Key Recovery Tool for Flipper Zero**

[![Latest Release](https://img.shields.io/github/v/release/Lemn4t/mfkey_desktop_cli?style=for-the-badge&logo=github&color=blue)](https://github.com/Lemn4t/mfkey_desktop_cli/releases/latest)[![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads/Lemn4t/mfkey_desktop_cli/latest/total?style=for-the-badge&logo=github&color=success)](https://github.com/Lemn4t/mfkey_desktop_cli/releases/latest)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)[![C](https://img.shields.io/badge/C-00599C?style=for-the-badge&logo=c&logoColor=white)](<https://en.wikipedia.org/wiki/C_(programming_language)>)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey?style=for-the-badge)](https://github.com/Lemn4t/mfkey_desktop_cli/releases)

[![GitHub License](https://img.shields.io/github/license/Lemn4t/mfkey_desktop_cli?style=for-the-badge&color=orange)](LICENSE)

</div>

---

## 📥 Download

> **Ready-made builds are available on the releases page.:**
>
> ### ➡️ **[github.com/Lemn4t/mfkey_desktop_cli/releases](https://github.com/Lemn4t/mfkey_desktop_cli/releases)**

Download the binary for your platform, and you can run it immediately — you don't need to install any dependencies.

---

## ✨ Opportunities

- ⚡ **High—performance core in C** — Crypto-1 recovery algorithm Ported from [Proxmark3](https://github.com/RfidResearchGroup/proxmark3)
- 🦀 **Secure binding to Rust** — parsing, attack orchestration, and CLI
- 🎯 Support for **three types of attacks**:
  - `mfkey32` — key recovery from two intercepted authentications (Moebius /mfkey32v2)
  - `static_nested` — attacking static nested‑nonces
  - `static_encrypted` — attack on encrypted nonces
- 🤖 **`--auto` mode** — talk to the Flipper directly over USB: auto-detect the port, pull the `.mfkey32.log` / `.nested.log` files, run the attack and **upload recovered keys and candidate dictionaries back to the device** — fully hands-free
- 📄 **Automatic detection** of the input file format (Flipper Zero log format)
- 🛑 Interrupt by `Ctrl+C` with correct termination
- 💾 Saving found keys and candidate dictionaries

---

## 🤖 Automatic mode (`--auto`)

The `--auto` flag turns the tool into a one-click solution that works **directly with a connected Flipper Zero** over USB — no manual file copying required.

```bash
mfkey_desktop_cli --auto
```

If the port is not detected automatically (this can happen on Windows when USB metadata is missing), specify it manually:

```bash
mfkey_desktop_cli --auto --port COM3        # Windows
mfkey_desktop_cli --auto --port /dev/ttyACM0 # Linux
mfkey_desktop_cli --auto --port /dev/cu.usbmodemflip_XXXX1 # macOS
```

### What it does, step by step

1. **🔌 Auto-detects** the Flipper Zero serial port (or uses `--port`).
2. **🤝 Opens an RPC session** over USB-CDC (raises DTR/RTS — required on Windows).
3. **📂 Lists** `/ext/nfc` and finds **all** log files — both `.mfkey32.log` **and** `.nested.log` are processed in the same run; recovered keys from every file are merged together.
4. **⬇️ Downloads** the log files and runs the appropriate attack on each one.
5. **🔀 Smart dictionary merge:**
   - If `mf_classic_dict_user.nfc` **already exists** on the Flipper — it is downloaded, and **only new keys** are appended to the existing ones.
   - If the file **does not exist** — a fresh dictionary is created containing only the recovered keys.
6. **⬆️ Uploads** the merged keys file back to `/ext/nfc/assets/mf_classic_dict_user.nfc` (full overwrite, not an append). If nothing new was found, the upload is skipped.
7. **🗂️ Uploads candidate dictionaries** (`mf_classic_dict_<uid>.nfc`) to `/ext/nfc/assets/` as well. These are **always written fresh**: if a file with the same name already exists on the device, it is deleted and replaced with the new one (full overwrite, no merge).

> [!NOTE]
> Before running `--auto`, **close qFlipper, the Web Updater and any serial terminals** — they hold the COM/serial port exclusively and will prevent the tool from communicating with the device.

> [!NOTE]
> Candidate dictionaries are now uploaded to the device too. Depending on **how many** candidate files were generated and **how large** each one is, this step may take noticeably longer — uploads over USB-CDC can be slow when there are many or heavy files. This is expected; just let it finish.

> [!TIP]
> On Windows you can find the Flipper's COM port in **Device Manager → Ports (COM & LPT)**. A Flipper may expose more than one COM port — if the first one doesn't respond, try the next.

---

## 🚀 Using

### Single file (offline)

```bash
mfkey_desktop_cli <input_file>
```

Example:

```bash
mfkey_desktop_cli .nested.log
```

### Automatic mode (live device)

```bash
mfkey_desktop_cli --auto
mfkey_desktop_cli --auto --port COM3
```

> [!NOTE]
> **Linux / macOS:** before the first run, make the binary executable:
>
> ```bash
> chmod +x mfkey_desktop_cli
> ```
>
> On macOS you may also need to allow it in **System Settings → Privacy & Security** if Gatekeeper blocks it.
>
> On Linux, accessing the serial port may require adding your user to the `dialout` group:
>
> ```bash
> sudo usermod -aG dialout $USER
> ```
>
> (log out and back in for the change to take effect).

### Input file format

**Nested (format Flipper Zero):**

```
Sec 0 key B cuid da7d3c2e nt0 b07cef37 ks0 54a0efed par0 1001 nt1 224737c4 ks1 ce956841 par1 1000 dist 0
```

**MFKey32 (format Flipper Zero):**

```
Sec 0 key A cuid 801aa11c nt0 e58455e4 nr0 761ff4ec ar0 162122ec nt1 20782e85 nr1 ecb6f04f ar1 bf19891b
```

The program automatically detects the type of attack based on the contents of the string. The found keys are saved in `mf_classic_dict_user.nfc`.

---

## 🛠️ Source code build

You will need [Rust toolchain](https://rustup.rs/) and the C compiler (MSVC/GCC /Clang).

```bash
git clone https://github.com/Lemn4t/mfkey_desktop_cli.git
cd mfkey_desktop_cli
cargo build --release
```

The finished binary will appear in `target/release/`.

> [!NOTE]
> Protobuf definitions for the Flipper RPC protocol are compiled at build time with the pure-Rust [`protox`](https://crates.io/crates/protox) parser, so **`protoc` is not required** to build the project.

---

## 🧩 How it works

| Layer              | Language     | Responsibility                                         |
| ------------------ | ------------ | ------------------------------------------------------ |
| Core of Crypto‑1   | **C**        | Restoring the LFSR state, iterating through MSB tables |
| Parser and the CLI | **Rust**     | Nonce reading, attack selection, progress, withdrawal  |
| FFI bridge         | **Rust ↔ C** | Transfer of structures and callbacks between layers    |
| Flipper RPC (USB)  | **Rust**     | Serial transport, protobuf framing, Storage operations |

The attacks exploit known weaknesses of the Crypto‑1 cipher used in MIFARE Classic cards.

In `--auto` mode the tool speaks the Flipper Zero **Protobuf RPC** protocol over USB-CDC: it switches the CLI into RPC mode (`start_rpc_session`), then uses `Storage*` commands to list, read, write and delete files on the device.

---

## ⚖️ Disclaimer

> [!WARNING]
> This tool is intended **exclusively** for security research, training, and testing **your own** maps or maps that you have explicit permission to analyze.
>
> The author is not responsible for any misuse. Use it at your own risk and in accordance with the laws of your country.

---

## 🙏 Thanks

- [Proxmark3 / RfidResearchGroup](https://github.com/RfidResearchGroup/proxmark3)
- [mfkey / noproto](https://github.com/noproto/xero-firmware/tree/dev/applications/system/mfkey)
- [Flipper Zero Protobuf](https://github.com/flipperdevices/flipperzero-protobuf)

---

## 📄 License

The project is distributed under the **GPL-3.0** license. For more information, see the file [LICENSE](LICENSE).
