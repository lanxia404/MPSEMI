# MPSEMI — Fcitx5 中英混輸引擎（Rust + C++）

> 最小可用骨架。單一 preedit、候選列、空白鍵提交。後續自行替換為注音/拼音與英文解碼器。

## 特性

* Fcitx5 外掛（C++）+ 引擎核心（Rust）。
* 單一組字區（preedit），候選列由核心提供。
* 穩定 C-ABI 邊界，易於單元測試與後續擴充。

## 專案結構

```bash
MPSEMI/
├─ CMakeLists.txt
├─ cpp/
│  ├─ CMakeLists.txt
│  └─ mpsemi_engine.cpp
├─ rust/
│  ├─ Cargo.toml
│  └─ src/lib.rs
└─ share/
   ├─ addon/mpsemi-addon.conf
   └─ inputmethod/mpsemi.conf
```

## 需求

* 編譯工具：CMake ≥ 3.16、GCC/Clang、Rust stable、Cargo。
* Fcitx5 開發套件（含 `Fcitx5Core` 標頭）。
* 典型套件名：

  * Arch：`fcitx5 fcitx5-qt fcitx5-configtool fcitx5-devel`（視發行版包命名而定）
  * Debian/Ubuntu：`fcitx5`, `fcitx5-modules-dev`, `fcitx5-config-qt`（名稱可能不同）

## 建置與安裝

```bash
# 於專案根目錄
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr
cmake --build . -j
sudo cmake --install .
```

> 若連結器報「非 PIC」：
> 在 `rust/.cargo/config.toml` 新增
>
> ```ini
> [build]
> rustflags = ["-C", "relocation-model=pic"]
> ```

## 啟用與測試

1. 執行 `fcitx5-configtool`，加入「MPSEMI」輸入法。
2. 重啟輸入法：`fcitx5 -rd`
3. 任意輸入框中鍵入字元。空白鍵提交第一候選。

## 自訂核心（把骨架換成真混輸）

* Rust 端檔案：`rust/src/lib.rs`
* 目前暴露的 C-ABI：

  * `mpsemi_engine_new() / mpsemi_engine_free()`
  * `mpsemi_process_utf8(engine, const char* s)`：處理鍵序
  * `mpsemi_preedit(engine) -> char*`
  * `mpsemi_candidate_count(engine) -> u32`
  * `mpsemi_candidate_at(engine, i) -> char*`
  * `mpsemi_commit(engine) -> char*`
  * `mpsemi_free_cstr(char*)`
* 實作方向：

  * 將 `process()` 改為「分段：注音/拼音 + 英文」→ 各自產生候選 → 合併排序。
  * 在 C++ 端候選回調中呼叫 `mpsemi_commit()` 完成上屏與學習。

## 除錯

```bash
fcitx5 -rd             # 前景模式 + 偵錯輸出
fcitx5-diagnose        # 環境檢查
```

## 路線圖

* M1：骨架完成（已提供）。
* M2：注音/拼音 FSA + 英文拼寫器；基本詞頻排序。
* M3：雙語詞圖合併、個人化學習、Shift 強制 ASCII。
* M4：設定面板、詞庫導入/備份、性能剖析。

## 授權

* 預設 MIT。可自行更改，請同步加入 `LICENSE` 檔案。

---

## MPSEMI — Mixed Chinese/English IME for Fcitx5 (Rust + C++)

> Minimal runnable skeleton. Single preedit, candidate list, space to commit. Replace the Rust core with your real Bopomofo/Pinyin + English decoders.

## Features

* Fcitx5 plugin in C++. Engine core in Rust.
* Single preedit. Candidates supplied by the core.
* Stable C-ABI boundary for testing and iteration.

## Layout

```bash
MPSEMI/
├─ CMakeLists.txt
├─ cpp/
│  ├─ CMakeLists.txt
│  └─ mpsemi_engine.cpp
├─ rust/
│  ├─ Cargo.toml
│  └─ src/lib.rs
└─ share/
   ├─ addon/mpsemi-addon.conf
   └─ inputmethod/mpsemi.conf
```

## Prerequisites

* CMake ≥ 3.16, GCC/Clang, Rust stable, Cargo.
* Fcitx5 development headers (`Fcitx5Core`).
* Package names vary by distro.

## Build and Install

```bash
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr
cmake --build . -j
sudo cmake --install .
```

> If the linker complains about non-PIC, add:
>
> ```ini
> # rust/.cargo/config.toml
> [build]
> rustflags = ["-C", "relocation-model=pic"]
> ```

## Enable and Test

1. Open `fcitx5-configtool`, add “MPSEMI”.
2. Restart: `fcitx5 -rd`
3. Type in any text field. Press Space to commit the top candidate.

## Customize the Core

* File: `rust/src/lib.rs`
* C-ABI you can rely on:

  * `mpsemi_engine_new() / mpsemi_engine_free()`
  * `mpsemi_process_utf8(engine, const char* s)`
  * `mpsemi_preedit(engine) -> char*`
  * `mpsemi_candidate_count(engine) -> u32`
  * `mpsemi_candidate_at(engine, i) -> char*`
  * `mpsemi_commit(engine) -> char*`
  * `mpsemi_free_cstr(char*)`
* Implement segmentation (Zh/En), candidate generation, and merged ranking.

## Debug

```bash
fcitx5 -rd
fcitx5-diagnose
```

## Roadmap

* M1 skeleton.
* M2 Zh/En decoders + base ranking.
* M3 merged lattice, personalization, Shift-for-ASCII.
* M4 settings UI, lexicon import/backup, profiling.

## License

* MIT by default. Replace as needed.
