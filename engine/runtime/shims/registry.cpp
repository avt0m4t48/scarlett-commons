#include <windows.h>

#include <cstdio>
#include <cstring>
#include <map>
#include <mutex>
#include <set>
#include <string>

#include "shims.h"

namespace rt::shims {
namespace {

std::map<std::string, void*>& table() {
    static std::map<std::string, void*> t;  // "dll!name"
    return t;
}

std::mutex g_called_mutex;
std::set<std::string> g_called;

// Called by the thunks with the import's "dll!name" string.
extern "C" uint64_t rt_unimplemented(const char* what) {
    std::lock_guard lock(g_called_mutex);
    if (g_called.insert(what).second) std::fprintf(stderr, "[rt] unimplemented: %s\n", what);
    return 0;
}

// Thunk layout (x86-64):
//   48 B9 <imm64>   mov rcx, what
//   48 B8 <imm64>   mov rax, rt_unimplemented
//   0F 57 C0        xorps xmm0, xmm0
//   FF E0           jmp rax              (tail call: returns straight to the game's caller)
constexpr size_t kThunkSize = 32;

uint8_t* g_pool = nullptr;
size_t g_pool_used = 0, g_pool_size = 0;
std::mutex g_pool_mutex;

}  // namespace

Table::Table(const Entry* entries, size_t count) {
    for (size_t i = 0; i < count; ++i) table()[std::string(entries[i].dll) + "!" + entries[i].name] = entries[i].fn;
}

void* find(const std::string& dll_lower, const std::string& name) {
    auto it = table().find(dll_lower + "!" + name);
    return it == table().end() ? nullptr : it->second;
}

void* make_unimplemented_thunk(const std::string& dll, const std::string& name) {
    std::lock_guard lock(g_pool_mutex);
    if (!g_pool || g_pool_used + kThunkSize > g_pool_size) {
        g_pool_size = 1 << 20;
        g_pool = static_cast<uint8_t*>(VirtualAlloc(nullptr, g_pool_size, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE));
        g_pool_used = 0;
        if (!g_pool) return nullptr;
    }
    const char* what = _strdup((dll + "!" + name).c_str());  // lives for the process
    uint8_t* t = g_pool + g_pool_used;
    g_pool_used += kThunkSize;
    uint64_t a = reinterpret_cast<uint64_t>(what), f = reinterpret_cast<uint64_t>(&rt_unimplemented);
    uint8_t code[kThunkSize] = {0x48, 0xB9};
    std::memcpy(code + 2, &a, 8);
    code[10] = 0x48; code[11] = 0xB8;
    std::memcpy(code + 12, &f, 8);
    code[20] = 0x0F; code[21] = 0x57; code[22] = 0xC0;
    code[23] = 0xFF; code[24] = 0xE0;
    std::memset(code + 25, 0xCC, kThunkSize - 25);
    std::memcpy(t, code, kThunkSize);
    FlushInstructionCache(GetCurrentProcess(), t, kThunkSize);
    return t;
}

void report_called_unimplemented() {
    std::lock_guard lock(g_called_mutex);
    for (auto& s : g_called) std::fprintf(stderr, "  called: %s\n", s.c_str());
}

}  // namespace rt::shims
