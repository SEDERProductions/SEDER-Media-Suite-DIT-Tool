cmake_minimum_required(VERSION 3.21)

file(READ "${CMAKE_CURRENT_SOURCE_DIR}/../Cargo.toml" CARGO_TOML)
string(REGEX MATCH "\nversion = \"([^\"]+)\"" _ ${CARGO_TOML})
set(CARGO_VERSION "${CMAKE_MATCH_1}")
if(NOT CARGO_VERSION)
    set(CARGO_VERSION "0.0.0")
endif()
project(SederMediaSuiteDitQt VERSION ${CARGO_VERSION} LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

include(CTest)
include(GNUInstallDirs)

set(SEDER_QT_COMPONENTS Core Gui Quick QuickControls2 Widgets)
if(BUILD_TESTING)
    list(APPEND SEDER_QT_COMPONENTS Test)
endif()

find_package(Qt6 6.4 REQUIRED COMPONENTS ${SEDER_QT_COMPONENTS})

qt_standard_project_setup(REQUIRES 6.4)

set(RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/rust-target")
set(RUST_MANIFEST "${CMAKE_CURRENT_SOURCE_DIR}/../Cargo.toml")
set(RUST_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/lib.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/ffi.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/offload/mod.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/offload/engine.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/offload/hash.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/offload/media.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/offload/template.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/offload/volume.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/../src/report.rs"
)

function(seder_config_to_cargo_profile config out_profile out_release_flag)
    if(config STREQUAL "Debug")
        set(profile "debug")
        set(release_flag "")
    else()
        set(profile "release")
        set(release_flag "--release")
    endif()

    set(${out_profile} "${profile}" PARENT_SCOPE)
    set(${out_release_flag} "${release_flag}" PARENT_SCOPE)
endfunction()

set(SEDER_RUST_TARGET "" CACHE STRING "Rust target triple for cross-compilation (e.g. x86_64-apple-darwin)")

set(_seder_cargo_extra_args "")
if(SEDER_RUST_TARGET)
    list(APPEND _seder_cargo_extra_args "--target" "${SEDER_RUST_TARGET}")
endif()

set(_seder_known_configs Debug Release RelWithDebInfo MinSizeRel)
set(_seder_rust_outputs)
set(_seder_seen_rust_outputs)

foreach(_cfg IN LISTS _seder_known_configs)
    seder_config_to_cargo_profile("${_cfg}" _cargo_profile _cargo_release_flag)
    if(SEDER_RUST_TARGET)
        set(_rust_staticlib_${_cfg} "${RUST_TARGET_DIR}/${SEDER_RUST_TARGET}/${_cargo_profile}/${CMAKE_STATIC_LIBRARY_PREFIX}seder_dit_tool${CMAKE_STATIC_LIBRARY_SUFFIX}")
    else()
        set(_rust_staticlib_${_cfg} "${RUST_TARGET_DIR}/${_cargo_profile}/${CMAKE_STATIC_LIBRARY_PREFIX}seder_dit_tool${CMAKE_STATIC_LIBRARY_SUFFIX}")
    endif()

    # Only add custom_command for unique output paths (Release/RelWithDebInfo/MinSizeRel all map to Cargo release profile)
    if(NOT "${_rust_staticlib_${_cfg}}" IN_LIST _seder_seen_rust_outputs)
        add_custom_command(
            OUTPUT "${_rust_staticlib_${_cfg}}"
            COMMAND ${CMAKE_COMMAND} -E echo "Building SEDER DIT Rust core for CMake config ${_cfg} using Cargo profile ${_cargo_profile}"
            COMMAND cargo build --manifest-path "${RUST_MANIFEST}" ${_cargo_release_flag} --target-dir "${RUST_TARGET_DIR}" ${_seder_cargo_extra_args}
            DEPENDS "${RUST_MANIFEST}" ${RUST_SOURCES}
            WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/.."
            COMMENT "Building SEDER DIT Rust core (${_cfg})"
            VERBATIM
        )
        list(APPEND _seder_seen_rust_outputs "${_rust_staticlib_${_cfg}}")
    endif()

    list(APPEND _seder_rust_outputs "${_rust_staticlib_${_cfg}}")
endforeach()

if(CMAKE_CONFIGURATION_TYPES)
    set(_seder_active_configs ${CMAKE_CONFIGURATION_TYPES})
else()
    if(CMAKE_BUILD_TYPE)
        set(_seder_active_configs ${CMAKE_BUILD_TYPE})
    else()
        set(_seder_active_configs Debug)
    endif()
endif()

foreach(_cfg IN LISTS _seder_active_configs)
    seder_config_to_cargo_profile("${_cfg}" _cargo_profile _cargo_release_flag)
    message(STATUS "CMake config '${_cfg}' maps to Cargo profile '${_cargo_profile}'")
endforeach()

add_custom_target(seder_dit_rust_build DEPENDS ${_seder_rust_outputs})
add_library(seder_dit_rust STATIC IMPORTED GLOBAL)
set_target_properties(seder_dit_rust PROPERTIES
    IMPORTED_LOCATION_DEBUG "${_rust_staticlib_Debug}"
    IMPORTED_LOCATION_RELEASE "${_rust_staticlib_Release}"
    IMPORTED_LOCATION_RELWITHDEBINFO "${_rust_staticlib_RelWithDebInfo}"
    IMPORTED_LOCATION_MINSIZEREL "${_rust_staticlib_MinSizeRel}"
)
add_dependencies(seder_dit_rust seder_dit_rust_build)

set(SEDER_APP_ICON_MACOS "${CMAKE_CURRENT_SOURCE_DIR}/../assets/icon.icns")
set(SEDER_APP_ICON_WINDOWS "${CMAKE_CURRENT_SOURCE_DIR}/../assets/icon.ico")
set(SEDER_APP_ICON_WINDOWS_RC "${CMAKE_CURRENT_SOURCE_DIR}/app.rc")

set(SEDER_APP_PLATFORM_SOURCES "")
if(APPLE AND EXISTS "${SEDER_APP_ICON_MACOS}")
    set_source_files_properties("${SEDER_APP_ICON_MACOS}" PROPERTIES
        MACOSX_PACKAGE_LOCATION "Resources")
    list(APPEND SEDER_APP_PLATFORM_SOURCES "${SEDER_APP_ICON_MACOS}")
endif()
if(WIN32 AND EXISTS "${SEDER_APP_ICON_WINDOWS}")
    list(APPEND SEDER_APP_PLATFORM_SOURCES "${SEDER_APP_ICON_WINDOWS_RC}")
endif()

qt_add_executable(seder-media-suite-dit-qt
    MACOSX_BUNDLE
    WIN32
    cpp/AppController.cpp
    cpp/AppController.h
    cpp/DestinationItem.cpp
    cpp/DestinationItem.h
    cpp/DestinationListModel.cpp
    cpp/DestinationListModel.h
    cpp/DitOffloadWorker.cpp
    cpp/DitOffloadWorker.h
    cpp/SettingsStore.cpp
    cpp/SettingsStore.h
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
        qml/MetaLabel.qml
        qml/FieldLabel.qml
        qml/DenseTextField.qml
        qml/QuietButton.qml
        qml/StyledComboBox.qml
        qml/StyledCheckBox.qml
        qml/StyledProgressBar.qml
        qml/PathPicker.qml
        qml/AboutDialog.qml
        qml/PreferencesDialog.qml
)

target_include_directories(seder-media-suite-dit-qt PRIVATE cpp)
target_compile_definitions(seder-media-suite-dit-qt PRIVATE
    SEDER_DIT_VERSION="${CARGO_VERSION}"
)
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

if(Qt6_VERSION VERSION_GREATER_EQUAL "6.5.0")
    qt_generate_deploy_qml_app_script(
        TARGET seder-media-suite-dit-qt
        OUTPUT_SCRIPT seder_deploy_script
        NO_UNSUPPORTED_PLATFORM_ERROR
    )
else()
    qt_generate_deploy_qml_app_script(
        TARGET seder-media-suite-dit-qt
        FILENAME_VARIABLE seder_deploy_script
        NO_UNSUPPORTED_PLATFORM_ERROR
    )
endif()
install(SCRIPT ${seder_deploy_script})

if(BUILD_TESTING)
    qt_add_executable(dit_model_tests
        tests/dit_model_tests.cpp
        cpp/DestinationItem.cpp
        cpp/DestinationItem.h
        cpp/DestinationListModel.cpp
        cpp/DestinationListModel.h
        cpp/seder_ffi.h
    )
    target_include_directories(dit_model_tests PRIVATE cpp)
    target_link_libraries(dit_model_tests PRIVATE Qt6::Core Qt6::Test)
    add_test(NAME dit_model_tests COMMAND dit_model_tests)
endif()
