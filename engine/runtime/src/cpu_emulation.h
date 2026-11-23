// Emulates the few Zen 2 instructions in the game's code that a desktop CPU may lack (docs/cpu_features.md).
//
// The executable is built for the console's AMD Zen 2 and uses some of its instructions without asking CPUID:
// SSE4a INSERTQ/EXTRQ, CLZERO and MONITORX/MWAITX (AMD only) and CLWB (Intel since Ice Lake). On a host without
// one, the CPU raises #UD (STATUS_ILLEGAL_INSTRUCTION) at that instruction. A vectored handler recognises these
// encodings at the faulting RIP, does the same work on the thread's CONTEXT and continues after the instruction.
// On a host that has the feature the CPU runs the instruction itself and the handler never sees it.
//
// What each one does, from the AMD64 Architecture Programmer's Manual (vol. 3 for CLZERO, MONITORX, MWAITX; vol. 4
// for INSERTQ, EXTRQ) and the Intel SDM vol. 2 for CLWB:
//   INSERTQ xmm1, xmm2, len, idx   F2 0F 78 /r ib ib   insert the low `len` bits of xmm2[63:0] into xmm1[63:0] at
//                                                       bit `idx` (len 0 means 64)
//   INSERTQ xmm1, xmm2             F2 0F 79 /r         same, len = xmm2[77:72], idx = xmm2[69:64]
//   EXTRQ   xmm1, len, idx         66 0F 78 /0 ib ib   xmm1[63:0] = (xmm1[63:0] >> idx), `len` bits, zero-extended
//   EXTRQ   xmm1, xmm2             66 0F 79 /r         same, len = xmm2[5:0], idx = xmm2[13:8]
//   CLZERO                         0F 01 FC            zero the 64-byte cache line that rAX points into
//   MONITORX                       0F 01 FA            arm an address monitor: a hint, nothing to do here
//   MWAITX                         0F 01 FB            wait for the monitor or a timer; returning at once is one
//                                                       of the exits the manual allows, so it is a pause
//   CLWB m8                        66 0F AE /6 (mem)   write a line back, keeping it cached: ordinary write-back
//                                                       memory is coherent, so there is nothing to do here
// INSERTQ and EXTRQ leave xmm1[127:64] undefined; the emulation keeps them, so code cannot tell the difference.
// A field with len + idx > 64 is undefined too; the emulation keeps the bits that fit.
#pragma once

#include <cstddef>
#include <cstdint>
#include <cstring>

namespace rt::cpu_emulation {

enum class Op { none, insertq_imm, insertq_reg, extrq_imm, extrq_reg, clzero, monitorx, mwaitx, clwb };

struct Decoded {
    Op op = Op::none;
    std::size_t length = 0;  // bytes, prefixes included
    int dst = 0;             // xmm register (INSERTQ, EXTRQ)
    int src = 0;             // xmm register (the register forms)
    uint8_t len = 0;         // field length and index of the immediate forms
    uint8_t idx = 0;
    bool addr32 = false;     // a 67 prefix: CLZERO uses EAX
};

// One of the encodings above at `code` (`avail` readable bytes), or Op::none.
inline Decoded decode(const uint8_t* code, std::size_t avail) {
    Decoded d;
    std::size_t i = 0;
    bool p66 = false, pf2 = false, pf3 = false;
    uint8_t rex = 0;
    for (; i < avail && i < 14; ++i) {  // legacy prefixes, then at most one REX right before the opcode
        const uint8_t b = code[i];
        if (b == 0x66) p66 = true;
        else if (b == 0xF2) pf2 = true;
        else if (b == 0xF3) pf3 = true;
        else if (b == 0x67) d.addr32 = true;
        else if (b == 0x2E || b == 0x3E || b == 0x26 || b == 0x36 || b == 0x64 || b == 0x65) {}
        else break;
        rex = 0;
    }
    if (i < avail && (code[i] & 0xF0) == 0x40) rex = code[i++];
    if (i + 2 > avail || code[i] != 0x0F) return {};
    const uint8_t op = code[i + 1];
    i += 2;
    if (op == 0x01) {  // CLZERO, MONITORX, MWAITX: fixed three-byte forms, no 66/F2/F3
        if (i >= avail || p66 || pf2 || pf3) return {};
        const uint8_t m = code[i++];
        d.op = m == 0xFC ? Op::clzero : m == 0xFA ? Op::monitorx : m == 0xFB ? Op::mwaitx : Op::none;
        d.length = i;
        return d.op == Op::none ? Decoded{} : d;
    }
    if (i >= avail) return {};
    const uint8_t modrm = code[i++];
    const int mod = modrm >> 6, reg = ((modrm >> 3) & 7) | ((rex & 4) << 1), rm = (modrm & 7) | ((rex & 1) << 3);
    if (op == 0x78 || op == 0x79) {  // SSE4a; F2 is INSERTQ, 66 is EXTRQ; both need a register operand
        if (mod != 3 || pf3 || p66 == pf2) return {};
        const bool imm = op == 0x78;
        if (imm) {
            if (i + 2 > avail) return {};
            if (p66 && (modrm & 0x38)) return {};  // EXTRQ's immediate form is /0
            d.len = code[i] & 0x3F;
            d.idx = code[i + 1] & 0x3F;
            i += 2;
        }
        d.op = pf2 ? (imm ? Op::insertq_imm : Op::insertq_reg) : (imm ? Op::extrq_imm : Op::extrq_reg);
        if (p66 && imm) d.dst = rm;  // EXTRQ xmm1, imm, imm names xmm1 in r/m
        else d.dst = reg, d.src = rm;
        d.length = i;
        return d;
    }
    if (op == 0xAE && p66 && !pf2 && !pf3 && mod != 3 && ((modrm >> 3) & 7) == 6) {  // CLWB m8
        if ((modrm & 7) == 4) {  // SIB
            if (i >= avail) return {};
            const uint8_t sib = code[i++];
            if (mod == 0 && (sib & 7) == 5) i += 4;
        } else if (mod == 0 && (modrm & 7) == 5) {
            i += 4;  // RIP-relative
        }
        i += mod == 1 ? 1 : mod == 2 ? 4 : 0;
        if (i > avail) return {};
        d.op = Op::clwb;
        d.length = i;
        return d;
    }
    return {};
}

inline uint64_t field_mask(unsigned len) { return len >= 64 ? ~uint64_t(0) : (uint64_t(1) << len) - 1; }

// INSERTQ on the low quadword: `len` (0 means 64) bits of `src` placed at bit `idx` of `dst`.
inline uint64_t insert_bits(uint64_t dst, uint64_t src, unsigned len, unsigned idx) {
    len &= 63;
    idx &= 63;
    const uint64_t mask = field_mask(len ? len : 64) << idx;
    return (dst & ~mask) | ((src << idx) & mask);
}

// EXTRQ on the low quadword: `len` (0 means 64) bits of `src` from bit `idx`, zero-extended.
inline uint64_t extract_bits(uint64_t src, unsigned len, unsigned idx) {
    len &= 63;
    idx &= 63;
    return (src >> idx) & field_mask(len ? len : 64);
}

}  // namespace rt::cpu_emulation

#ifdef _WIN32
#include <windows.h>
#include <intrin.h>

#include <atomic>
#include <cstdio>

namespace rt::cpu_emulation {

struct Host {
    bool sse4a = false, monitorx = false, clzero = false, clwb = false;
};

inline Host detect_host() {
    Host h;
    int r[4];
    __cpuid(r, 0);
    const int max_leaf = r[0];
    if (max_leaf >= 7) {
        __cpuidex(r, 7, 0);
        h.clwb = (r[1] >> 24) & 1;  // EBX bit 24
    }
    __cpuid(r, 0x80000000);
    const unsigned max_ext = unsigned(r[0]);
    if (max_ext >= 0x80000001) {
        __cpuid(r, 0x80000001);
        h.sse4a = (r[2] >> 6) & 1;      // ECX bit 6
        h.monitorx = (r[2] >> 29) & 1;  // ECX bit 29
    }
    if (max_ext >= 0x80000008) {
        __cpuid(r, 0x80000008);
        h.clzero = r[1] & 1;  // EBX bit 0
    }
    return h;
}

inline const char* name(Op op) {
    switch (op) {
    case Op::insertq_imm: case Op::insertq_reg: return "INSERTQ (SSE4a)";
    case Op::extrq_imm: case Op::extrq_reg: return "EXTRQ (SSE4a)";
    case Op::clzero: return "CLZERO";
    case Op::monitorx: return "MONITORX";
    case Op::mwaitx: return "MWAITX";
    case Op::clwb: return "CLWB";
    default: return "none";
    }
}

// How many times each instruction was emulated; the two forms of INSERTQ and of EXTRQ share one count.
inline std::atomic<uint64_t>& count(Op op) {
    static std::atomic<uint64_t> counts[9];
    if (op == Op::insertq_reg) op = Op::insertq_imm;
    if (op == Op::extrq_reg) op = Op::extrq_imm;
    return counts[int(op)];
}

inline uint64_t& lo(CONTEXT* c, int r) { return c->FltSave.XmmRegisters[r].Low; }
inline uint64_t hi(const CONTEXT* c, int r) { return uint64_t(c->FltSave.XmmRegisters[r].High); }

// Apply one decoded instruction to `c` and step past it. False when the memory CLZERO zeroes is not writable, so
// the original exception stands.
inline bool apply(const Decoded& d, CONTEXT* c) {
    switch (d.op) {
    case Op::insertq_imm: lo(c, d.dst) = insert_bits(lo(c, d.dst), lo(c, d.src), d.len, d.idx); break;
    case Op::insertq_reg: {
        const uint64_t ctl = hi(c, d.src);
        lo(c, d.dst) = insert_bits(lo(c, d.dst), lo(c, d.src), unsigned(ctl >> 8) & 63, unsigned(ctl) & 63);
        break;
    }
    case Op::extrq_imm: lo(c, d.dst) = extract_bits(lo(c, d.dst), d.len, d.idx); break;
    case Op::extrq_reg: {
        const uint64_t ctl = lo(c, d.src);
        lo(c, d.dst) = extract_bits(lo(c, d.dst), unsigned(ctl) & 63, unsigned(ctl >> 8) & 63);
        break;
    }
    case Op::clzero: {
        const uint64_t addr = (d.addr32 ? (c->Rax & 0xFFFFFFFFu) : c->Rax) & ~uint64_t(63);
        __try {
            std::memset(reinterpret_cast<void*>(addr), 0, 64);
        } __except (EXCEPTION_EXECUTE_HANDLER) {
            return false;
        }
        break;
    }
    case Op::mwaitx: YieldProcessor(); break;
    case Op::monitorx: case Op::clwb: break;
    default: return false;
    }
    c->Rip += d.length;
    return true;
}

inline LONG CALLBACK handler(EXCEPTION_POINTERS* info) {
    if (info->ExceptionRecord->ExceptionCode != STATUS_ILLEGAL_INSTRUCTION) return EXCEPTION_CONTINUE_SEARCH;
    CONTEXT* c = info->ContextRecord;
    uint8_t code[15] = {};
    std::size_t avail = 0;
    __try {  // the faulting instruction was fetched, but read byte by byte in case it ends at a page boundary
        for (; avail < sizeof code; ++avail) code[avail] = reinterpret_cast<const volatile uint8_t*>(c->Rip)[avail];
    } __except (EXCEPTION_EXECUTE_HANDLER) {
    }
    const Decoded d = decode(code, avail);
    if (d.op == Op::none || !apply(d, c)) return EXCEPTION_CONTINUE_SEARCH;
    if (count(d.op).fetch_add(1, std::memory_order_relaxed) == 0)
        std::fprintf(stderr, "[rt] cpu: this CPU has no %s; emulating it (first at %p)\n", name(d.op),
                     reinterpret_cast<void*>(c->Rip - d.length));
    return EXCEPTION_CONTINUE_EXECUTION;
}

// Installs the handler first in the vectored chain, once. Returns false if Windows refused it.
inline bool install() {
    static void* const cookie = AddVectoredExceptionHandler(1, &handler);
    return cookie != nullptr;
}

}  // namespace rt::cpu_emulation
#endif
