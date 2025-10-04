cmake_minimum_required(VERSION 3.16)
project(MPSEMI LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(GNUInstallDirs)
find_package(Fcitx5Core REQUIRED)

add_subdirectory(cpp)
