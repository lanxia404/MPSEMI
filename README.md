# MPSEMI

MPSEMI 是一個以 Rust 為核心、透過 C/C++ 與 Fcitx 5 整合的輸入法引擎最小實作，示範如何將自訂文字處理流程包裝成 Fcitx 外掛。
MPSEMI is a minimal input method engine with a Rust core bridged into Fcitx 5 via C/C++, showing how to wrap custom text processing logic as an input method plug-in.

## 特色 Highlights

- 混合注音/英文輸入流程的原型，核心維護預編輯緩衝與候選列表並回傳給前端；Prototype for mixed Zhuyin/English typing that keeps a preedit buffer and candidate list in Rust.
- Rust 核心透過穩定的 C ABI (`mpsemi_*` 函式) 提供操作介面，便於其他語言呼叫；The Rust core exposes a stable C ABI (`mpsemi_*` functions) for easy interoperability.
- C++ 外掛使用 Fcitx5 API 處理按鍵、更新預編輯與候選窗，展示實際整合流程；The C++ plug-in uses the Fcitx5 API to handle key events and UI updates, serving as a reference implementation.
- `share/` 內含 addon 與 inputmethod 設定檔，方便安裝或打包；Metadata in `share/` is ready for installation or packaging.

## 專案結構 Repository Layout

- `rust/`：輸入法核心，使用 Rust 撰寫並輸出靜態函式庫與 C ABI；Rust core that builds a static library and C ABI.
- `cpp/`：Fcitx5 插件實作，連結 Rust 靜態庫並實作 `InputMethodEngine`；Fcitx5 plug-in linking against the Rust static library.
- `share/`：Fcitx addon 與 input method 的描述檔；Addon and input method description files used on install.
- `CMakeLists.txt`：頂層建置腳本，將 C++ 與 Rust 結合；Top-level CMake entry point combining both worlds.
- `RENAME.md`：繁體中文/英文專案說明；Bilingual project notes.

## 系統需求 Requirements

- Rust 工具鏈（支援 `edition = 2024`；若使用穩定版，請確保版本足夠新或切換 nightly）。Rust toolchain with edition 2024 support.
- CMake 3.16 以上。CMake ≥ 3.16.
- 支援 C++17 的編譯器（例如 GCC 10+ 或 Clang 12+）。A C++17-capable compiler (e.g., GCC 10+, Clang 12+).
- Fcitx 5 Core 開發套件（通常為 `fcitx5-devel` 或同名套件）。Fcitx 5 Core development files.

## 建置 Build

頂層 CMake 會驅動 Rust 與 C++ 的建置流程：

```bash
mkdir -p build
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=$HOME/.local
cmake --build build
```

上述指令會：

1. 以 Release 模式透過 `cargo` 編譯 `rust/` 內的靜態函式庫。
2. 編譯 `cpp/mpsemi_engine.cpp` 並連結前一步產生的 `libmpsemi_core.a`。
3. 產生 `mpsemi` 共享函式庫與對應輸出。

如需單獨驗證 Rust 核心，可執行：

```bash
cargo check --manifest-path rust/Cargo.toml
cargo build --manifest-path rust/Cargo.toml --release
```

## 安裝與啟用 Install & Enable

1. 執行安裝（此處示範安裝至使用者目錄）：

   ```bash
   cmake --install build --prefix $HOME/.local
   ```

該步驟會將 `mpsemi` 外掛安裝至 `${prefix}/lib/fcitx5`，並複製 `share/` 內的設定檔至相對應路徑。
2. 確認 Fcitx 5 能搜尋到自訂 prefix（預設會包含 `~/.local`）。
3. 啟動 `fcitx5-configtool`，在「輸入法」頁面新增 `MPSEMI`，或手動編輯 `~/.config/fcitx5/profile` 將 `mpsemi` 加入輸入法列表。
4. 切換至 MPSEMI 後即可測試：可視輸入字元會累積到預編輯區，空白或 Enter 會提交候選詞。

## Rust FFI 介面 Rust FFI Surface

Rust 核心透過下列函式與外部互動：

- `mpsemi_engine_new()` / `mpsemi_engine_free()`：建立或釋放引擎實例。
- `mpsemi_process_utf8(ptr, s)`：傳入 UTF-8 字串，更新預編輯與候選狀態並回傳是否消耗事件。
- `mpsemi_preedit(ptr)`：取得目前預編輯字串（呼叫端需以 `mpsemi_free_cstr` 釋放）。
- `mpsemi_candidate_count(ptr)` 與 `mpsemi_candidate_at(ptr, idx)`：查詢候選列表與內容。
- `mpsemi_commit(ptr)`：取得要送出的最終字串並重置內部狀態。
- `mpsemi_free_cstr(s)`：釋放任何從核心取得的字串。

## 開發流程 Development Tips

- 修改 Rust 程式碼後，先執行 `cargo fmt`、`cargo check` 或 `cargo clippy` 確保風格與靜態檢查通過。
- 修改 C++ 後，可使用 `cmake --build build --target mpsemi` 進行快速增量編譯。
- 若需要 debug 訊息，可在 Rust 或 C++ 中加入 `dbg!` / `std::cerr` 並重新建置。
- `share/` 內容會在 `cmake --install` 時自動複製，若新增其他 metadata，請同步更新安裝目標路徑。

## 授權 License

本專案以 MIT License 發佈，詳見 [`LICENSE`](LICENSE)。
The project is distributed under the MIT License. See [`LICENSE`](LICENSE) for details.
