cmake_minimum_required(VERSION 3.21)

project(SederMediaSuiteDitQt VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

include(CTest)
include(GNUInstallDirs)

set(SEDER_QT_COMPONENTS Core Gui Quick QuickControls2 Widgets)
if(BUILD_TESTING)
    list(APPEND SEDER_QT_COMPONENTS Test)
endif()

find_package(Qt6 6.5 REQUIRED COMPONENTS ${SEDER_QT_COMPONENTS})

qt_standard_project_setup(REQUIRES 6.5)

set(RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/rust-target")
set(RUST_STATICLIB "${RUST_TARGET_DIR}/release/${CMAKE_STATIC_LIBRARY_PREFIX}seder_dit_tool${CMAKE_STATIC_LIBRARY_SUFFIX}")
set(RUST_MANIFEST "${CMAKE_CURRENT_SOURCE_DIR}/../Cargo.toml")
set(RUST_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/lib.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/ffi.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/media_core.rs"
)

add_custom_command(
    OUTPUT "${RUST_STATICLIB}"
    COMMAND cargo build --manifest-path "${RUST_MANIFEST}" --release --target-dir "${RUST_TARGET_DIR}"
    DEPENDS "${RUST_MANIFEST}" ${RUST_SOURCES}
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/.."
    COMMENT "Building SEDER DIT Rust core"
    VERBATIM
)

add_custom_target(seder_dit_rust_build DEPENDS "${RUST_STATICLIB}")
add_library(seder_dit_rust STATIC IMPORTED GLOBAL)
set_target_properties(seder_dit_rust PROPERTIES IMPORTED_LOCATION "${RUST_STATICLIB}")
add_dependencies(seder_dit_rust seder_dit_rust_build)

set(SEDER_APP_ICON_MACOS "${CMAKE_CURRENT_SOURCE_DIR}/../assets/icon.icns")
set(SEDER_APP_ICON_WINDOWS_RC "${CMAKE_CURRENT_SOURCE_DIR}/app.rc")

set(SEDER_APP_PLATFORM_SOURCES "")
if(APPLE)
    set_source_files_properties("${SEDER_APP_ICON_MACOS}" PROPERTIES
        MACOSX_PACKAGE_LOCATION "Resources")
    list(APPEND SEDER_APP_PLATFORM_SOURCES "${SEDER_APP_ICON_MACOS}")
endif()
if(WIN32)
    list(APPEND SEDER_APP_PLATFORM_SOURCES "${SEDER_APP_ICON_WINDOWS_RC}")
endif()

qt_add_executable(seder-media-suite-dit-qt
    MACOSX_BUNDLE
    WIN32
    cpp/AppController.cpp
    cpp/AppController.h
    cpp/DitCompareWorker.cpp
    cpp/DitCompareWorker.h
    cpp/DitFilterProxyModel.cpp
    cpp/DitFilterProxyModel.h
    cpp/DitResult.h
    cpp/DitResultModel.cpp
    cpp/DitResultModel.h
    cpp/SederFfi.cpp
    cpp/SederFfi.h
    cpp/ThemeController.cpp
    cpp/ThemeController.h
    cpp/main.cpp
    cpp/seder_ffi.h
    ${SEDER_APP_PLATFORM_SOURCES}
)

qt_add_qml_module(seder-media-suite-dit-qt
    URI SederDit
    VERSION 1.0
    QML_FILES
        qml/Main.qml
)

target_include_directories(seder-media-suite-dit-qt PRIVATE cpp)
target_link_libraries(seder-media-suite-dit-qt
    PRIVATE
        Qt6::Core
        Qt6::Gui
        Qt6::Quick
        Qt6::QuickControls2
        Qt6::Widgets
        seder_dit_rust
)

if(WIN32)
    # Rust std uses Nt* APIs which live in ntdll.dll; when consuming the Rust
    # staticlib from an MSVC-linked Qt executable we must link its import lib.
    target_link_libraries(seder-media-suite-dit-qt PRIVATE ntdll)
endif()

set_target_properties(seder-media-suite-dit-qt PROPERTIES
    MACOSX_BUNDLE_GUI_IDENTIFIER com.sederproductions.media-suite.dit-qt
    MACOSX_BUNDLE_BUNDLE_NAME "SEDER Media Suite DIT"
    MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION}
    MACOSX_BUNDLE_ICON_FILE icon.icns
)

install(TARGETS seder-media-suite-dit-qt
    BUNDLE DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)

qt_generate_deploy_qml_app_script(
    TARGET seder-media-suite-dit-qt
    OUTPUT_SCRIPT seder_deploy_script
    NO_UNSUPPORTED_PLATFORM_ERROR
)
install(SCRIPT ${seder_deploy_script})

if(BUILD_TESTING)
    qt_add_executable(dit_model_tests
        tests/dit_model_tests.cpp
        cpp/DitFilterProxyModel.cpp
        cpp/DitFilterProxyModel.h
        cpp/DitResult.h
        cpp/DitResultModel.cpp
        cpp/DitResultModel.h
        cpp/seder_ffi.h
    )
    target_include_directories(dit_model_tests PRIVATE cpp)
    target_link_libraries(dit_model_tests PRIVATE Qt6::Core Qt6::Test)
    add_test(NAME dit_model_tests COMMAND dit_model_tests)
endif()
