# MPSEMI

## 專案簡介

MPSEMI 是一個以 Rust 為核心、透過 C/C++ 與 Fcitx 5 整合的輸入法引擎最小實作，示範如何將自訂文字處理流程包裝成 Fcitx 外掛。

## 功能特色

- 提供混合注音與英文輸入流程的原型，核心維護預編輯緩衝與候選列表。
- 透過穩定的 C ABI (`mpsemi_*`) 暴露引擎操作介面，方便跨語言整合。
- C++ 外掛利用 Fcitx5 API 取得按鍵事件、更新預編輯與候選窗。
- `share/` 內含 addon 與 input method 設定檔，可直接部署或打包。

## 專案結構

- `rust/`：輸入法核心（Rust 靜態函式庫與 C ABI）。
- `cpp/`：Fcitx5 外掛，連結 Rust 核心。
- `share/`：Fcitx addon/inputmethod 描述檔。
- `CMakeLists.txt`：頂層建置腳本，協調 C++ 與 Rust。
- `RENAME.md`：繁體中文與英文的額外說明。

## 系統需求

- Rust 工具鏈（支援 `edition = 2024`，穩定版需使用最新版本或改用 nightly）。
- CMake 3.16 以上。
- 支援 C++17 的編譯器（GCC 10+、Clang 12+ 等）。
- Fcitx 5 Core 開發套件（通常為 `fcitx5-devel` 或同名套件）。

## 建置流程

```bash
mkdir -p build
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=$HOME/.local
cmake --build build
```

以上指令會：

1. 透過 `cargo` 以 Release 模式編譯 `rust/` 的靜態函式庫。
2. 編譯 `cpp/mpsemi_engine.cpp` 並連結 `libmpsemi_core.a`。
3. 產生 `mpsemi` 共用函式庫。

若需單獨驗證 Rust 核心，可執行：

```bash
cargo check --manifest-path rust/Cargo.toml
cargo build --manifest-path rust/Cargo.toml --release
```

## 安裝與啟用

1. 安裝至自訂路徑（以下示範 `~/.local`）：

   ```bash
   cmake --install build --prefix $HOME/.local
   ```

2. 確認 Fcitx 5 搜尋路徑包含該 prefix（預設會包含 `~/.local`）。
3. 使用 `fcitx5-configtool` 在輸入法列表加入 `MPSEMI`，或直接編輯 `~/.config/fcitx5/profile`。
4. 切換至 MPSEMI 即可測試：輸入會累積在預編輯區，空白或 Enter 會提交候選詞。

## Rust FFI 介面

- `mpsemi_engine_new()` / `mpsemi_engine_free()`：建立與釋放引擎實例。
- `mpsemi_process_utf8(ptr, s)`：傳入 UTF-8 字串，更新狀態並回傳是否已處理。
- `mpsemi_preedit(ptr)`：取得預編輯字串（需以 `mpsemi_free_cstr` 釋放）。
- `mpsemi_candidate_count(ptr)`、`mpsemi_candidate_at(ptr, idx)`：查詢候選筆數與內容。
- `mpsemi_commit(ptr)`：取得要提交的字串並重置內部狀態。
- `mpsemi_free_cstr(s)`：釋放由核心回傳的字串。

## 開發建議

- Rust 端建議搭配 `cargo fmt`、`cargo check`、`cargo clippy` 維持程式品質。
- C++ 端可利用 `cmake --build build --target mpsemi` 快速增量建置。
- 需要偵錯時，可在 Rust 使用 `dbg!`，在 C++ 使用 `std::cerr`。
- 若新增 Fcitx metadata，請同步調整 `share/` 與對應的安裝路徑。

## 授權

本專案以 MIT License 發佈，詳見 [`LICENSE`](LICENSE)。

---

## MPSEMI (English)

## Overview

MPSEMI is a minimal input method engine with a Rust core linked into Fcitx 5 via C/C++. It showcases how to wrap custom text-processing logic as an Fcitx plug-in.

## Highlights

- Prototype of a mixed Zhuyin/English flow that keeps a preedit buffer and candidate list in Rust.
- Stable C ABI (`mpsemi_*`) makes the engine consumable from other languages.
- C++ plug-in demonstrates handling key events and Fcitx5 UI updates.
- Metadata in `share/` can be deployed or packaged directly.

## Repository Layout

- `rust/`: Rust core producing a static library with a C ABI.
- `cpp/`: Fcitx5 plug-in that links against the Rust core.
- `share/`: Addon and input method descriptors.
- `CMakeLists.txt`: Top-level build script coordinating both components.
- `RENAME.md`: Additional bilingual project notes.

## Requirements

- Rust toolchain with support for edition 2024 (use the latest stable or switch to nightly).
- CMake 3.16 or newer.
- A C++17-capable compiler (GCC 10+, Clang 12+, etc.).
- Fcitx 5 Core development headers/libraries (`fcitx5-devel` or similar).

## Build Instructions

```bash
mkdir -p build
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=$HOME/.local
cmake --build build
```

These commands:

1. Compile the Rust static library in `rust/` via `cargo` (Release mode).
2. Build `cpp/mpsemi_engine.cpp` and link against `libmpsemi_core.a`.
3. Produce the `mpsemi` shared library.

To verify the Rust core independently:

```bash
cargo check --manifest-path rust/Cargo.toml
cargo build --manifest-path rust/Cargo.toml --release
```

## Install & Enable

1. Install to a prefix (example uses `~/.local`):

   ```bash
   cmake --install build --prefix $HOME/.local
   ```

2. Ensure Fcitx 5 searches the chosen prefix (it includes `~/.local` by default).
3. Use `fcitx5-configtool` to add `MPSEMI`, or edit `~/.config/fcitx5/profile` manually.
4. Switch to MPSEMI to test: visible characters accumulate in preedit, Space/Enter commits the candidate.

## Rust FFI Surface

- `mpsemi_engine_new()` / `mpsemi_engine_free()`: create and destroy engine instances.
- `mpsemi_process_utf8(ptr, s)`: feed UTF-8 text, update state, return whether the event was consumed.
- `mpsemi_preedit(ptr)`: fetch the current preedit (release with `mpsemi_free_cstr`).
- `mpsemi_candidate_count(ptr)`, `mpsemi_candidate_at(ptr, idx)`: inspect candidate list size and entries.
- `mpsemi_commit(ptr)`: obtain the commit string and reset internal buffers.
- `mpsemi_free_cstr(s)`: release strings returned by the core.

## Development Tips

- Run `cargo fmt`, `cargo check`, and `cargo clippy` on Rust changes.
- Rebuild the C++ plug-in incrementally with `cmake --build build --target mpsemi`.
- Add debug output via `dbg!` in Rust or `std::cerr` in C++ as needed.
- Keep `share/` and install paths in sync when adding new Fcitx metadata.

## License

Distributed under the MIT License. See [`LICENSE`](LICENSE) for details.
