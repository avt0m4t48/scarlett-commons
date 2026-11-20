// Shim registry: reimplementations of console-only imports.
// Add a DLL by writing a shims/<dll>.cpp with a SHIM_TABLE and listing it in CMakeLists.txt.
#pragma once
#include <string>

namespace rt::shims {

struct Entry {
    const char* dll;   // lower-case, with .dll
    const char* name;  // export name, or "#ordinal"
    void* fn;
};

// Registers a static table at startup.
struct Table {
    Table(const Entry* entries, size_t count);
};

void* find(const std::string& dll_lower, const std::string& name);

// Makes a small thunk that logs "dll!name" once and returns 0 (in rax and xmm0).
void* make_unimplemented_thunk(const std::string& dll, const std::string& name);

// Prints every unimplemented import that has been called.
void report_called_unimplemented();

}  // namespace rt::shims

#define SHIM_TABLE(id, ...)                                                       \
    static const ::rt::shims::Entry id##_entries[] = {__VA_ARGS__};               \
    static ::rt::shims::Table id##_table(id##_entries, sizeof(id##_entries) / sizeof(id##_entries[0]))
