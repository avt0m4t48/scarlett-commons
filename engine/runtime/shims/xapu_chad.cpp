// xapu.dll and chad.dll: the console's audio processing unit (APU) and audio DSP command queues. No PC has either,
// so these give the title connections whose calls succeed, a zeroed shared buffer and an always-empty result queue:
// the audio the APU would render stays silent, and nothing the title submits is processed.
//
// Read from the executable's call sites (counts in docs/runtime_imports.md), statically, never run:
//   XApuConnect(out24*, out16*, handle*)      1 site; success is SUCCEEDED(hr). The first qword of the 16-byte
//                                             output is read later as the base of a CPU-visible buffer: per-stream
//                                             input at base + slot * 0x280 and output at base + 0x14000 +
//                                             slot * 0x1E00. The 24-byte output is never read.
//   XApuEnqueueCommand(handle, cmd*, 0)       4 sites; the return value is never read. cmd's first dword is a type
//                                             (1, 2 and 3 seen), the second a stream slot.
//   XApuGetResultCount(handle, 0)             1 site; 0 skips the loop that dequeues.
//   XApuDequeueResult(handle, out*)           1 site, called count times into a zeroed 0x50-byte local; a negative
//                                             HRESULT ends the loop.
//   XApuDisconnect(handle)                    1 site; the return value is never read.
//   XDspConnectWithMaximumStreamLimit(buffer, u32, u32, 0x80, 1, handle*)   1 site; success is a return of 0. The
//                                             buffer comes from the title's XMemAlloc and stays the title's.
//   XDspActivate(handle, ...4 more)           1 site; success is a return of 0, else the caller returns -1.
//   XDspSubmitCommand(a, b)                   1 site; success is a return of 0.
//   XDspDeactivate, XDspDisconnect            1 site each, tail calls: the caller returns what they return.
//
// HYPOTHESIS: the shared buffer is 0x14000 + 128 * 0x1E00 bytes. 128 slots is what fits 0x14000 at the 0x280 input
// stride, and the DSP connect passes 0x80 as its stream limit; no call site shows a bound. The 16-byte output is
// written as {base, size}; only the base is measured. The result queue stays empty because no result layout is
// known beyond the first two dwords, so XApuDequeueResult returns E_FAIL (a negative HRESULT, which the caller
// handles) without writing. Every command is accepted with S_OK and dropped.
#include <windows.h>
#include <intrin.h>

#include <cstdint>
#include <cstring>
#include <mutex>
#include <set>
#include <string>

#include "shims.h"
#include "startup_trace.h"

namespace {

constexpr size_t kApuBufferSize = 0x14000 + 128 * 0x1E00;

struct ApuConnection {
    void* buffer;
};

struct DspConnection {
    int unused;
};

// gta6rt --trace FILE: the first call of each function and the first XApuEnqueueCommand of each command type, with
// the caller, so the first boot shows what the title sends without a line per call.
void note(const char* api, uint64_t value, const void* caller) {
    if (!rt::startup_trace::enabled()) return;
    static std::mutex mutex;
    static std::set<std::pair<std::string, uint64_t>> seen;
    {
        const std::lock_guard lock(mutex);
        if (!seen.insert({api, value}).second) return;
    }
    rt::startup_trace::event(
        "audio_queue", std::string("\"api\":") + rt::startup_trace::quote(api) +
                           ",\"value\":" + rt::startup_trace::quote(rt::startup_trace::hex(value)) + ",\"caller\":" +
                           rt::startup_trace::quote(rt::startup_trace::where(reinterpret_cast<uint64_t>(caller))));
}

HRESULT __stdcall XApuConnect(void* out24, void* out16, void** handle) {
    note("XApuConnect", 0, _ReturnAddress());
    if (!handle) return E_INVALIDARG;
    *handle = nullptr;
    void* buffer = VirtualAlloc(nullptr, kApuBufferSize, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);  // zeroed
    if (!buffer) return E_OUTOFMEMORY;
    if (out24) std::memset(out24, 0, 24);
    if (out16) {
        const uint64_t info[2] = {reinterpret_cast<uint64_t>(buffer), kApuBufferSize};
        std::memcpy(out16, info, sizeof info);
    }
    *handle = new ApuConnection{buffer};
    return S_OK;
}

HRESULT __stdcall XApuDisconnect(void* handle) {
    note("XApuDisconnect", 0, _ReturnAddress());
    auto* c = static_cast<ApuConnection*>(handle);
    if (!c) return E_INVALIDARG;
    VirtualFree(c->buffer, 0, MEM_RELEASE);
    delete c;
    return S_OK;
}

HRESULT __stdcall XApuEnqueueCommand(void* handle, const uint32_t* command, uint64_t flags) {
    note("XApuEnqueueCommand", command ? command[0] : ~0ull, _ReturnAddress());
    (void)flags;
    return handle && command ? S_OK : E_INVALIDARG;
}

uint32_t __stdcall XApuGetResultCount(void* handle, uint64_t flags) {
    note("XApuGetResultCount", 0, _ReturnAddress());
    (void)handle;
    (void)flags;
    return 0;
}

HRESULT __stdcall XApuDequeueResult(void* handle, void* result) {
    note("XApuDequeueResult", 0, _ReturnAddress());
    (void)handle;
    (void)result;
    return E_FAIL;  // the queue is always empty
}

HRESULT __stdcall XDspConnectWithMaximumStreamLimit(void* buffer, uint32_t a, uint32_t b, uint32_t max_streams,
                                                    uint32_t flags, void** handle) {
    note("XDspConnectWithMaximumStreamLimit", max_streams, _ReturnAddress());
    (void)buffer;
    (void)a;
    (void)b;
    (void)flags;
    if (!handle) return E_INVALIDARG;
    *handle = new DspConnection{};
    return S_OK;
}

HRESULT __stdcall XDspDisconnect(void* handle) {
    note("XDspDisconnect", 0, _ReturnAddress());
    delete static_cast<DspConnection*>(handle);
    return S_OK;
}

HRESULT __stdcall XDspActivate(void* handle, uint64_t, uint64_t, uint64_t, uint64_t) {
    note("XDspActivate", 0, _ReturnAddress());
    return handle ? S_OK : E_INVALIDARG;
}

HRESULT __stdcall XDspDeactivate(void* handle, uint64_t, uint64_t, uint64_t) {
    note("XDspDeactivate", 0, _ReturnAddress());
    (void)handle;
    return S_OK;
}

// Which of the two arguments is the connection is not known (the wrapper swaps them), so neither is checked.
HRESULT __stdcall XDspSubmitCommand(void*, void*) {
    note("XDspSubmitCommand", 0, _ReturnAddress());
    return S_OK;
}

}  // namespace

SHIM_TABLE(xapu,
    {"xapu.dll", "XApuConnect", reinterpret_cast<void*>(&XApuConnect)},
    {"xapu.dll", "XApuDequeueResult", reinterpret_cast<void*>(&XApuDequeueResult)},
    {"xapu.dll", "XApuDisconnect", reinterpret_cast<void*>(&XApuDisconnect)},
    {"xapu.dll", "XApuEnqueueCommand", reinterpret_cast<void*>(&XApuEnqueueCommand)},
    {"xapu.dll", "XApuGetResultCount", reinterpret_cast<void*>(&XApuGetResultCount)},
);

SHIM_TABLE(chad,
    {"chad.dll", "XDspActivate", reinterpret_cast<void*>(&XDspActivate)},
    {"chad.dll", "XDspConnectWithMaximumStreamLimit", reinterpret_cast<void*>(&XDspConnectWithMaximumStreamLimit)},
    {"chad.dll", "XDspDeactivate", reinterpret_cast<void*>(&XDspDeactivate)},
    {"chad.dll", "XDspDisconnect", reinterpret_cast<void*>(&XDspDisconnect)},
    {"chad.dll", "XDspSubmitCommand", reinterpret_cast<void*>(&XDspSubmitCommand)},
);
