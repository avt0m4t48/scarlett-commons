cmake_minimum_required(VERSION 3.24)
project(gta6rt LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
  message(FATAL_ERROR "gta6rt must be built for x64 (run from an x64 Native Tools prompt or vcvars64.bat)")
endif()

add_executable(gta6rt
  src/main.cpp
  src/pe_loader.cpp
  shims/registry.cpp
  shims/xmem.cpp
  shims/d3d12_xs.cpp
)
target_compile_definitions(gta6rt PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX UNICODE _UNICODE)
target_compile_options(gta6rt PRIVATE /W4 /permissive- /utf-8)
# Static shim tables register themselves at startup; keep the linker from dropping their objects.
target_link_options(gta6rt PRIVATE /OPT:NOREF)
target_link_libraries(gta6rt PRIVATE psapi)
