<div align="center">

# 🔑 Flipper Zero :: MFKey Desktop CLI

**MIFARE Classic Cross-platform CLI Key Recovery Tool for Flipper Zero**

[![Latest Release](https://img.shields.io/github/v/release/Lemn4t/mfkey_desktop_cli?style=for-the-badge&logo=github&color=blue)](https://github.com/Lemn4t/mfkey_desktop_cli/releases/latest)
[![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads/Lemn4t/mfkey_desktop_cli/latest/total?style=for-the-badge&logo=github&color=success)](https://github.com/Lemn4t/mfkey_desktop_cli/releases/latest)

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
- 📄 **Automatic detection** of the input file format (Flipper Zero log format)
- 🛑 Interrupt by `Ctrl+C` with correct termination
- 💾 Saving found keys and candidate dictionaries

---

## 🚀 Using

```bash
mfkey_desktop_cli <input_file>
```

Example:

```bash
mfkey_desktop_cli .nested.log
```

> [!NOTE]
> **Linux / macOS:** before the first run, make the binary executable:
>
> ```bash
> chmod +x mfkey_desktop_cli
> ```
>
> On macOS you may also need to allow it in **System Settings → Privacy & Security** if Gatekeeper blocks it.

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

---

## 🧩 How it works

| Layer              | Language     | Responsibility                                         |
| ------------------ | ------------ | ------------------------------------------------------ |
| Core of Crypto‑1   | **C**        | Restoring the LFSR state, iterating through MSB tables |
| Parser and the CLI | **Rust**     | Nonce reading, attack selection, progress, withdrawal  |
| FFI bridge         | **Rust ↔ C** | Transfer of structures and callbacks between layers    |

The attacks exploit known weaknesses of the Crypto‑1 cipher used in MIFARE Classic cards.

---

## ⚖️ Disclaimer

> [!WARNING]
> This tool is intended **exclusively** for security research, training, and testing **your own** maps or maps that you have explicit permission to analyze.
>
> The author is not responsible for any misuse. Use it at your own risk and in accordance with the laws of your country.

---

## 🙏 Thanks

- [Proxmark3 / RfidResearchGroup](https://github.com/RfidResearchGroup/proxmark3) — the original Crypto‑1 algorithm

---

## 📄 License

The project is distributed under the **GPL-3.0** license. For more information, see the file [LICENSE](LICENSE).
