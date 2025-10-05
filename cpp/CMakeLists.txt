include(GNUInstallDirs)
find_package(Fcitx5Core REQUIRED)

# 來源與輸出
set(RUST_SRC_LIB ${CMAKE_SOURCE_DIR}/rust/target/release/libmpsemi_core.a)
set(RUST_BUILD_LIB ${CMAKE_CURRENT_BINARY_DIR}/libmpsemi_core.a)

# 單一規則：用 cargo 建 -> 複製到 build/cpp/
add_custom_command(
	OUTPUT ${RUST_BUILD_LIB}
	COMMAND cargo build --manifest-path ${CMAKE_SOURCE_DIR}/rust/Cargo.toml --release
	COMMAND ${CMAKE_COMMAND} -E copy_if_different ${RUST_SRC_LIB} ${RUST_BUILD_LIB}
	WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/rust
	COMMENT "正在構建Rust核心: ${RUST_LIB_NAME}\n 如果失敗，請先安裝 Rust 工具鏈 (https://www.rust-lang.org/tools/install)並依網頁指示安裝\n 若為第一次建置，請耐心等候，因為需要下載並編譯 Rust 相依套件\n Now building Rust core: ${RUST_LIB_NAME}.\n If it fails, please install Rust toolchain first (https://www.rust-lang.org/tools/install) and follow the instructions on the webpage.\n If this is the first build, please be patient as it needs to download and compile Rust dependencies."
	VERBATIM
)

# 明確的 target 供相依
add_custom_target(mpsemi_core ALL DEPENDS ${RUST_BUILD_LIB})

# 外掛本體，將 .a 當成來源以建立檔案級相依
add_library(mpsemi SHARED mpsemi_engine.cpp ${RUST_BUILD_LIB})
add_dependencies(mpsemi mpsemi_core)
target_link_libraries(mpsemi PRIVATE Fcitx5::Core ${RUST_BUILD_LIB})
set_target_properties(mpsemi PROPERTIES PREFIX "")

# 安裝
install(TARGETS mpsemi DESTINATION ${CMAKE_INSTALL_LIBDIR}/fcitx5)
install(FILES ${CMAKE_SOURCE_DIR}/share/addon/mpsemi-addon.conf
        DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/addon RENAME mpsemi.conf)
install(FILES ${CMAKE_SOURCE_DIR}/share/inputmethod/mpsemi.conf
        DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod)
