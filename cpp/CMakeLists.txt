include(GNUInstallDirs)
find_package(Fcitx5Core REQUIRED)

# 產出路徑與檔名
set(RUST_LIB ${CMAKE_SOURCE_DIR}/rust/target/release/libmpsemi_core.a)
set(RUST_STAMP ${CMAKE_CURRENT_BINARY_DIR}/mpsemi_core.stamp)

# 用 cargo 建 Rust，並把靜態庫複製到 build/cpp/
add_custom_command(
    OUTPUT ${RUST_STAMP}
    COMMAND cargo build --manifest-path ${CMAKE_SOURCE_DIR}/rust/Cargo.toml --release
	COMMAND ${CMAKE_COMMAND} -E touch ${RUST_STAMP}
	WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/rust
    COMMENT "正在構建Rust核心: ${RUST_LIB_NAME}\n 如果失敗，請先安裝 Rust 工具鏈 (https://www.rust-lang.org/tools/install)並依網頁指示安裝\n 若為第一次建置，請耐心等候，因為需要下載並編譯 Rust 相依套件\n Now building Rust core: ${RUST_LIB_NAME}.\n If it fails, please install Rust toolchain first (https://www.rust-lang.org/tools/install) and follow the instructions on the webpage.\n If this is the first build, please be patient as it needs to download and compile Rust dependencies."
    VERBATIM
)

# 將上面 OUTPUT 宣告成 target，供後續相依
add_custom_target(mpsemi_core ALL DEPENDS ${RUST_STAMP})

# 宣告已存在的靜態庫為 IMPORTED，並讓它依賴 build 動作
add_library(mpsemi_core_lib STATIC IMPORTED GLOBAL)
set_target_properties(mpsemi_core_lib PROPERTIES IMPORTED_LOCATION ${RUST_LIB})
add_dependencies(mpsemi_core_lib mpsemi_core)

# Fcitx5 外掛
add_library(mpsemi SHARED mpsemi_engine.cpp)
target_link_libraries(mpsemi PRIVATE Fcitx5::Core mpsemi_core_lib)
set_target_properties(mpsemi PROPERTIES PREFIX "")

# 安裝
install(TARGETS mpsemi DESTINATION ${CMAKE_INSTALL_LIBDIR}/fcitx5)
install(FILES ${CMAKE_SOURCE_DIR}/share/addon/mpsemi-addon.conf
        DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/addon RENAME mpsemi.conf)
install(FILES ${CMAKE_SOURCE_DIR}/share/inputmethod/mpsemi.conf
        DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod)