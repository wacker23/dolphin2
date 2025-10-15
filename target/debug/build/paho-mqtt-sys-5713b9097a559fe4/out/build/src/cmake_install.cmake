# Install script for directory: /home/stladmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/paho-mqtt-sys-0.9.0/paho.mqtt.c/src

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Debug")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Install shared libraries without execute permission?
if(NOT DEFINED CMAKE_INSTALL_SO_NO_EXE)
  set(CMAKE_INSTALL_SO_NO_EXE "1")
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

# Set default install directory permissions.
if(NOT DEFINED CMAKE_OBJDUMP)
  set(CMAKE_OBJDUMP "/usr/bin/objdump")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out/build/src/libpaho-mqtt3c.a")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out/build/src/libpaho-mqtt3a.a")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE FILE FILES
    "/home/stladmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/paho-mqtt-sys-0.9.0/paho.mqtt.c/src/MQTTAsync.h"
    "/home/stladmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/paho-mqtt-sys-0.9.0/paho.mqtt.c/src/MQTTClient.h"
    "/home/stladmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/paho-mqtt-sys-0.9.0/paho.mqtt.c/src/MQTTClientPersistence.h"
    "/home/stladmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/paho-mqtt-sys-0.9.0/paho.mqtt.c/src/MQTTProperties.h"
    "/home/stladmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/paho-mqtt-sys-0.9.0/paho.mqtt.c/src/MQTTReasonCodes.h"
    "/home/stladmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/paho-mqtt-sys-0.9.0/paho.mqtt.c/src/MQTTSubscribeOpts.h"
    "/home/stladmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/paho-mqtt-sys-0.9.0/paho.mqtt.c/src/MQTTExportDeclarations.h"
    )
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out/build/src/libpaho-mqtt3cs.a")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out/build/src/libpaho-mqtt3as.a")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/eclipse-paho-mqtt-c/eclipse-paho-mqtt-cConfig.cmake")
    file(DIFFERENT _cmake_export_file_changed FILES
         "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/eclipse-paho-mqtt-c/eclipse-paho-mqtt-cConfig.cmake"
         "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out/build/src/CMakeFiles/Export/dd175520bdcfdcc5f75bc4f14a6d7fe8/eclipse-paho-mqtt-cConfig.cmake")
    if(_cmake_export_file_changed)
      file(GLOB _cmake_old_config_files "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/eclipse-paho-mqtt-c/eclipse-paho-mqtt-cConfig-*.cmake")
      if(_cmake_old_config_files)
        string(REPLACE ";" ", " _cmake_old_config_files_text "${_cmake_old_config_files}")
        message(STATUS "Old export file \"$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/lib/cmake/eclipse-paho-mqtt-c/eclipse-paho-mqtt-cConfig.cmake\" will be replaced.  Removing files [${_cmake_old_config_files_text}].")
        unset(_cmake_old_config_files_text)
        file(REMOVE ${_cmake_old_config_files})
      endif()
      unset(_cmake_old_config_files)
    endif()
    unset(_cmake_export_file_changed)
  endif()
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/eclipse-paho-mqtt-c" TYPE FILE FILES "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out/build/src/CMakeFiles/Export/dd175520bdcfdcc5f75bc4f14a6d7fe8/eclipse-paho-mqtt-cConfig.cmake")
  if(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/eclipse-paho-mqtt-c" TYPE FILE FILES "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out/build/src/CMakeFiles/Export/dd175520bdcfdcc5f75bc4f14a6d7fe8/eclipse-paho-mqtt-cConfig-debug.cmake")
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/eclipse-paho-mqtt-c" TYPE FILE FILES "/mnt/c/Users/Admin/Desktop/work/dolphin/target/debug/build/paho-mqtt-sys-5713b9097a559fe4/out/build/src/eclipse-paho-mqtt-cConfigVersion.cmake")
endif()

