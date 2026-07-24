# Security Policy

## 🎯 Scope & Intended Use

`mfkey_desktop_cli` is a **security research tool** for recovering MIFARE
Classic keys from Flipper Zero logs. It is intended **exclusively** for use on
**your own** cards or cards you have **explicit permission** to analyze, in
accordance with the laws of your country.

This security policy covers vulnerabilities in the **tool itself** (the Rust
CLI/RPC layer and the C Crypto-1 core), **not** the well-known cryptographic
weaknesses of the MIFARE Classic / Crypto-1 cipher, which the tool
intentionally exploits and which are out of scope.

---

## 🛡️ Supported Versions

Security fixes are applied to the latest release. Older releases are not
maintained — please update to the most recent version before reporting.

| Version        | Supported      |
| -------------- | -------------- |
| Latest release | ✅ Yes         |
| Older releases | ❌ No          |
| `dev` branch   | ✅ Best-effort |

You can always find the latest version on the
[Releases page](https://github.com/phntm-lab/mfkey_desktop_cli/releases).

---

## 📮 Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, report them privately using one of the following channels:

1. **GitHub Security Advisories (preferred):**
   Use the
   [_Report a vulnerability_](https://github.com/phntm-lab/mfkey_desktop_cli/security/advisories/new)
   button under the **Security** tab to open a private advisory.

2. **Private message:**
   Contact the maintainer directly via
   [Telegram](https://t.me/Lemn4t).

### What to include

To help us triage and fix the issue quickly, please include:

- A clear description of the vulnerability and its potential impact.
- The affected component (Rust CLI, parser, Flipper RPC/USB layer, or the
  C Crypto-1 core / FFI bridge).
- Affected version(s) and your platform (Windows / Linux / macOS).
- Step-by-step reproduction instructions or a proof-of-concept.
- Any relevant input samples (e.g. a malformed `.mfkey32.log` / `.nested.log`)
  — please sanitize sensitive data where possible.
- Any suggested mitigation or fix, if you have one.

---

## ⏱️ Response Process

- **Acknowledgement:** we aim to acknowledge your report within **72 hours**.
- **Assessment:** we will investigate and confirm the issue, then work on a
  fix and keep you informed of progress.
- **Disclosure:** once a fix is available, we will publish a new release and,
  with your consent, credit you in the advisory / release notes.

We kindly ask that you practice **responsible disclosure** and give us a
reasonable amount of time to address the issue before any public disclosure.

---

## ✅ Examples of In-Scope Vulnerabilities

- Memory-safety issues in the C core or at the Rust ↔ C FFI boundary
  (buffer overflows, use-after-free, out-of-bounds reads/writes).
- Crashes or undefined behavior triggered by crafted/malformed input log files.
- Path traversal or unsafe file handling when reading/writing local files or
  the on-device dictionary (`mf_classic_dict_user.nfc`).
- Issues in the Flipper RPC / serial transport that could corrupt device data
  unexpectedly (e.g. overwriting unintended files via `Storage*` commands).
- Dependency vulnerabilities that materially affect the tool.

## ⛔ Out of Scope

- The inherent cryptographic weaknesses of MIFARE Classic / Crypto-1 (these
  are the basis of the tool's functionality).
- Misuse of the tool against cards you do not own or have no permission to test.
- Issues requiring physical access to an already-compromised host machine.
- Vulnerabilities in third-party hardware/firmware (Flipper Zero itself,
  qFlipper, etc.) — report those to their respective projects.

---

Thank you for helping keep `mfkey_desktop_cli` and its users safe! 🙏
